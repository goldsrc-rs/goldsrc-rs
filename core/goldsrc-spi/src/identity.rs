//! Strongly-typed vendor-agnostic player identity, GUID, and SteamID representations.

use std::fmt;

/// Computes a deterministic 128-bit hash using MurmurHash3 (x64, 128-bit variant).
///
/// Guaranteed to be 100% deterministic and endianness-safe across all targets (x86, ARM, WASM).
pub fn murmur3_128(data: &[u8], seed: u64) -> [u8; 16] {
    let mut h1: u64 = seed;
    let mut h2: u64 = seed;

    const C1: u64 = 0x87c3_7b91_1142_53d5;
    const C2: u64 = 0x4cf5_ad43_2745_937f;

    let n_blocks = data.len() / 16;
    for i in 0..n_blocks {
        let chunk = &data[i * 16..(i + 1) * 16];
        let mut k1 = u64::from_le_bytes(chunk[0..8].try_into().expect("valid slice"));
        let mut k2 = u64::from_le_bytes(chunk[8..16].try_into().expect("valid slice"));

        k1 = k1.wrapping_mul(C1);
        k1 = k1.rotate_left(31);
        k1 = k1.wrapping_mul(C2);
        h1 ^= k1;

        h1 = h1.rotate_left(27);
        h1 = h1.wrapping_add(h2);
        h1 = h1.wrapping_mul(5).wrapping_add(0x52dc_e729);

        k2 = k2.wrapping_mul(C2);
        k2 = k2.rotate_left(33);
        k2 = k2.wrapping_mul(C1);
        h2 ^= k2;

        h2 = h2.rotate_left(31);
        h2 = h2.wrapping_add(h1);
        h2 = h2.wrapping_mul(5).wrapping_add(0x3849_5ab5);
    }

    let tail = &data[n_blocks * 16..];
    let mut k1: u64 = 0;
    let mut k2: u64 = 0;

    let len = tail.len();
    if len > 8 {
        for (i, &b) in tail[8..].iter().enumerate() {
            k2 ^= (b as u64) << (i * 8);
        }
        k2 = k2.wrapping_mul(C2);
        k2 = k2.rotate_left(33);
        k2 = k2.wrapping_mul(C1);
        h2 ^= k2;
    }
    if len > 0 {
        let first_len = len.min(8);
        for (i, &b) in tail[..first_len].iter().enumerate() {
            k1 ^= (b as u64) << (i * 8);
        }
        k1 = k1.wrapping_mul(C1);
        k1 = k1.rotate_left(31);
        k1 = k1.wrapping_mul(C2);
        h1 ^= k1;
    }

    let total_len = data.len() as u64;
    h1 ^= total_len;
    h2 ^= total_len;

    h1 = h1.wrapping_add(h2);
    h2 = h2.wrapping_add(h1);

    let fmix = |mut k: u64| -> u64 {
        k ^= k >> 33;
        k = k.wrapping_mul(0xff51_afd7_ed55_8ccd);
        k ^= k >> 33;
        k = k.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        k ^= k >> 33;
        k
    };

    h1 = fmix(h1);
    h2 = fmix(h2);

    h1 = h1.wrapping_add(h2);
    h2 = h2.wrapping_add(h1);

    let mut out = [0u8; 16];
    out[0..8].copy_from_slice(&h1.to_le_bytes());
    out[8..16].copy_from_slice(&h2.to_le_bytes());
    out
}

/// Canonical, vendor-agnostic 128-bit player GUID.
///
/// Deterministically generated from verified authentication sources and suitable
/// for persistent database indexing, ranking systems, and ACL rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerGuid(pub [u8; 16]);

impl PlayerGuid {
    /// Nil (all-zeros) GUID constant.
    pub const NIL: Self = Self([0u8; 16]);

    /// Creates a GUID from raw 16 bytes.
    #[inline(always)]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Returns a reference to the raw 16 bytes.
    #[inline(always)]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Computes deterministic GUID for a SteamID (steam64).
    pub fn for_steam(steam64: u64) -> Self {
        let mut buf = [0u8; 14 + 8];
        buf[0..14].copy_from_slice(b"goldsrc:steam:");
        buf[14..22].copy_from_slice(&steam64.to_le_bytes());
        Self(murmur3_128(&buf, 0x1337_c001))
    }

    /// Computes deterministic GUID for an AI Bot by name.
    pub fn for_bot(name: &str) -> Self {
        let mut data = Vec::with_capacity(12 + name.len());
        data.extend_from_slice(b"goldsrc:bot:");
        data.extend_from_slice(name.as_bytes());
        Self(murmur3_128(&data, 0x1337_c001))
    }

    /// Computes deterministic GUID for a LAN client by IP address.
    pub fn for_lan(ip: std::net::IpAddr) -> Self {
        let ip_str = ip.to_string();
        let mut data = Vec::with_capacity(12 + ip_str.len());
        data.extend_from_slice(b"goldsrc:lan:");
        data.extend_from_slice(ip_str.as_bytes());
        Self(murmur3_128(&data, 0x1337_c001))
    }

    /// Computes deterministic GUID for an HLTV proxy.
    pub fn for_hltv() -> Self {
        Self(murmur3_128(b"goldsrc:hltv", 0x1337_c001))
    }

    /// Computes deterministic GUID for any custom external provider.
    pub fn for_custom(provider: &str, raw_id: &str) -> Self {
        let mut data = Vec::with_capacity(provider.len() + 1 + raw_id.len());
        data.extend_from_slice(provider.as_bytes());
        data.push(b':');
        data.extend_from_slice(raw_id.as_bytes());
        Self(murmur3_128(&data, 0x1337_c001))
    }

    /// Returns standard 32-character lowercase hex string representation.
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(32);
        for byte in &self.0 {
            use std::fmt::Write;
            let _ = write!(s, "{:02x}", byte);
        }
        s
    }

    /// Returns standard 8-4-4-4-12 UUID string representation.
    pub fn to_uuid_string(&self) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.0[0],
            self.0[1],
            self.0[2],
            self.0[3],
            self.0[4],
            self.0[5],
            self.0[6],
            self.0[7],
            self.0[8],
            self.0[9],
            self.0[10],
            self.0[11],
            self.0[12],
            self.0[13],
            self.0[14],
            self.0[15]
        )
    }

    /// Parses a 32-character hex or UUID string into a `PlayerGuid`.
    pub fn from_hex(s: &str) -> Option<Self> {
        let clean = s.replace('-', "");
        if clean.len() != 32 {
            return None;
        }
        let mut bytes = [0u8; 16];
        for i in 0..16 {
            bytes[i] = u8::from_str_radix(&clean[i * 2..i * 2 + 2], 16).ok()?;
        }
        Some(Self(bytes))
    }
}

impl fmt::Display for PlayerGuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_uuid_string())
    }
}

/// Strongly-typed 8-byte POD representation of a SteamID.
///
/// Converts between Steam2 (`STEAM_0:Y:Z`), Steam3 (`[U:1:AccID]`), and Steam64
/// with zero heap allocations during parsing and bitwise conversions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SteamId(pub u64);

impl SteamId {
    /// Base offset for individual Steam accounts in 64-bit format.
    pub const INDIVIDUAL_BASE: u64 = 0x0110_0001_0000_0000; // 76561197960265728

    /// Creates a `SteamId` directly from a 64-bit Steam ID.
    #[inline(always)]
    pub const fn from_steam64(id: u64) -> Self {
        Self(id)
    }

    /// Creates a `SteamId` from a 32-bit account ID (Individual / Public universe).
    #[inline(always)]
    pub const fn from_account_id(account_id: u32) -> Self {
        Self(Self::INDIVIDUAL_BASE | (account_id as u64))
    }

    /// Returns the full 64-bit Steam ID.
    #[inline(always)]
    pub const fn steam64(&self) -> u64 {
        self.0
    }

    /// Returns the 32-bit Account ID portion.
    #[inline(always)]
    pub const fn account_id(&self) -> u32 {
        (self.0 & 0xFFFF_FFFF) as u32
    }

    /// Returns the Auth Server bit (Y in `STEAM_0:Y:Z`, either 0 or 1).
    #[inline(always)]
    pub const fn auth_server(&self) -> u32 {
        self.account_id() & 1
    }

    /// Returns the Auth ID number (Z in `STEAM_0:Y:Z`).
    #[inline(always)]
    pub const fn auth_id(&self) -> u32 {
        self.account_id() >> 1
    }

    /// Formats this SteamID into classic Steam2 format (e.g. `STEAM_0:1:12345678`).
    pub fn to_steam2(&self) -> String {
        format!("STEAM_0:{}:{}", self.auth_server(), self.auth_id())
    }

    /// Formats this SteamID into modern Steam3 format (e.g. `[U:1:24691356]`).
    pub fn to_steam3(&self) -> String {
        format!("[U:1:{}]", self.account_id())
    }

    /// Parses a raw string into a `SteamId` without heap allocation during parsing.
    ///
    /// Accepts:
    /// - Steam2: `STEAM_X:Y:Z` or `VALVE_X:Y:Z`
    /// - Steam3: `[U:1:Z]`
    /// - Steam64 or numeric Account ID: `7656119...` or `123456`
    pub fn parse(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }

        // 1. Steam2: STEAM_X:Y:Z or VALVE_X:Y:Z
        let rest = if let Some(stripped) = trimmed.strip_prefix("STEAM_") {
            stripped
        } else {
            trimmed.strip_prefix("VALVE_").unwrap_or_default()
        };

        if !rest.is_empty() {
            let mut parts = rest.split(':');
            let _universe = parts.next()?;
            let auth_server: u32 = parts.next()?.parse().ok()?;
            let auth_id: u32 = parts.next()?.parse().ok()?;
            if parts.next().is_some() || auth_server > 1 {
                return None;
            }
            let account_id = auth_id.checked_mul(2)?.checked_add(auth_server)?;
            return Some(Self::from_account_id(account_id));
        }

        // 2. Steam3: [U:1:Z]
        if let Some(stripped) = trimmed.strip_prefix("[U:1:")
            && let Some(z_str) = stripped.strip_suffix(']')
        {
            let account_id: u32 = z_str.parse().ok()?;
            return Some(Self::from_account_id(account_id));
        }

        // 3. Decimal digits
        if trimmed.chars().all(|c| c.is_ascii_digit())
            && let Ok(num) = trimmed.parse::<u64>()
        {
            if num >= Self::INDIVIDUAL_BASE {
                return Some(Self(num));
            } else if num <= u32::MAX as u64 {
                return Some(Self::from_account_id(num as u32));
            }
        }

        None
    }
}

impl fmt::Display for SteamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_steam2())
    }
}

/// Concrete identity provider source of an authenticated client.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AuthIdentity {
    /// External authenticated identity (Steam, Discord, Reunion, Custom token).
    External {
        provider: Box<str>,
        raw_id: Box<str>,
    },
    /// Local simulated AI client.
    Bot { name: Box<str> },
    /// Local Area Network client.
    Lan { client_ip: std::net::IpAddr },
    /// HLTV broadcast proxy.
    Hltv,
}

impl AuthIdentity {
    /// Creates a Steam external identity.
    pub fn steam(steam_id: SteamId) -> Self {
        Self::External {
            provider: "steam".into(),
            raw_id: steam_id.to_steam2().into_boxed_str(),
        }
    }

    /// Creates an AI bot identity.
    pub fn bot(name: impl Into<Box<str>>) -> Self {
        Self::Bot { name: name.into() }
    }

    /// Creates a LAN client identity.
    pub fn lan(ip: std::net::IpAddr) -> Self {
        Self::Lan { client_ip: ip }
    }

    /// Creates an HLTV identity.
    pub fn hltv() -> Self {
        Self::Hltv
    }

    /// Creates a custom provider identity.
    pub fn external(provider: impl Into<Box<str>>, raw_id: impl Into<Box<str>>) -> Self {
        Self::External {
            provider: provider.into(),
            raw_id: raw_id.into(),
        }
    }

    /// Computes the canonical deterministic `PlayerGuid` for this identity.
    pub fn to_guid(&self) -> PlayerGuid {
        match self {
            Self::External { provider, raw_id } => {
                if provider.as_ref() == "steam"
                    && let Some(id) = SteamId::parse(raw_id)
                {
                    PlayerGuid::for_steam(id.steam64())
                } else {
                    PlayerGuid::for_custom(provider, raw_id)
                }
            }
            Self::Bot { name } => PlayerGuid::for_bot(name),
            Self::Lan { client_ip } => PlayerGuid::for_lan(*client_ip),
            Self::Hltv => PlayerGuid::for_hltv(),
        }
    }

    /// Returns the provider name (e.g. "steam", "engine:bot", "engine:lan", "engine:hltv").
    pub fn provider(&self) -> &str {
        match self {
            Self::External { provider, .. } => provider,
            Self::Bot { .. } => "engine:bot",
            Self::Lan { .. } => "engine:lan",
            Self::Hltv => "engine:hltv",
        }
    }

    /// Returns the parsed `SteamId` if this identity is from the Steam provider.
    pub fn steam_id(&self) -> Option<SteamId> {
        match self {
            Self::External { provider, raw_id } if provider.as_ref() == "steam" => {
                SteamId::parse(raw_id)
            }
            _ => None,
        }
    }
}

/// Authenticated client identity paired with its canonical deterministic GUID.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AuthSubject {
    pub identity: AuthIdentity,
    pub guid: PlayerGuid,
}

impl AuthSubject {
    /// Creates a new `AuthSubject` computing the deterministic `PlayerGuid` from `identity`.
    pub fn new(identity: AuthIdentity) -> Self {
        let guid = identity.to_guid();
        Self { identity, guid }
    }

    /// Creates an authenticated Steam subject.
    pub fn steam(steam_id: SteamId) -> Self {
        Self::new(AuthIdentity::steam(steam_id))
    }

    /// Creates an authenticated Bot subject.
    pub fn bot(name: impl Into<Box<str>>) -> Self {
        Self::new(AuthIdentity::bot(name))
    }

    /// Creates an authenticated LAN subject.
    pub fn lan(ip: std::net::IpAddr) -> Self {
        Self::new(AuthIdentity::lan(ip))
    }

    /// Creates an authenticated HLTV subject.
    pub fn hltv() -> Self {
        Self::new(AuthIdentity::hltv())
    }

    /// Creates an authenticated external provider subject.
    pub fn external(provider: impl Into<Box<str>>, raw_id: impl Into<Box<str>>) -> Self {
        Self::new(AuthIdentity::external(provider, raw_id))
    }
}

/// Authentication lifecycle state of a client connection.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AuthState {
    /// Handshake or external credential verification is in progress.
    #[default]
    Pending,
    /// Client is authenticated with a verified identity and deterministic GUID.
    Authenticated(AuthSubject),
}

impl AuthState {
    /// Returns `true` if authentication is completed.
    #[inline(always)]
    pub fn is_authenticated(&self) -> bool {
        matches!(self, Self::Authenticated(_))
    }

    /// Returns the canonical `PlayerGuid` if authenticated.
    pub fn guid(&self) -> Option<PlayerGuid> {
        match self {
            Self::Authenticated(subject) => Some(subject.guid),
            Self::Pending => None,
        }
    }

    /// Returns the `SteamId` if authenticated via Steam provider.
    pub fn steam_id(&self) -> Option<SteamId> {
        match self {
            Self::Authenticated(subject) => subject.identity.steam_id(),
            Self::Pending => None,
        }
    }

    /// Returns the authenticated subject reference, if authenticated.
    pub fn subject(&self) -> Option<&AuthSubject> {
        match self {
            Self::Authenticated(subject) => Some(subject),
            Self::Pending => None,
        }
    }
}

/// Comprehensive network and authentication identity of a player.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerIdentity {
    /// Client slot index (1..=32).
    pub slot: i32,
    /// Engine user ID (`pfnGetPlayerUserId`), monotonic and unique per server connection.
    pub user_id: u32,
    /// Raw engine auth string (e.g. "STEAM_0:1:12345", "BOT", "STEAM_ID_PENDING").
    pub raw_auth_id: String,
    /// Authentication lifecycle state.
    pub auth_state: AuthState,
    /// Client IP address string (e.g. "192.168.1.10" or "127.0.0.1"), parsed from userinfo.
    pub ip: Option<String>,
    /// Client ping latency in milliseconds, if available from `pfnGetPlayerStats`.
    pub ping: i32,
    /// Packet loss percentage (0..=100), if available from `pfnGetPlayerStats`.
    pub packet_loss: i32,
    /// Whether this client is a simulated AI bot (`FL_FAKECLIENT`).
    pub is_bot: bool,
    /// Whether this client is an HLTV spectator proxy (`FL_PROXY`).
    pub is_hltv: bool,
}

impl PlayerIdentity {
    /// Returns the raw Auth ID string representation.
    #[inline]
    pub fn auth_id(&self) -> &str {
        &self.raw_auth_id
    }

    /// Returns the canonical GUID if authenticated.
    #[inline]
    pub fn guid(&self) -> Option<PlayerGuid> {
        self.auth_state.guid()
    }

    /// Returns the parsed SteamId if authenticated via Steam.
    #[inline]
    pub fn steam_id(&self) -> Option<SteamId> {
        self.auth_state.steam_id()
    }

    /// Returns the IP string or fallback to localhost if unavailable.
    #[inline]
    pub fn ip_str(&self) -> &str {
        self.ip.as_deref().unwrap_or("127.0.0.1")
    }

    /// Returns `true` if authentication is completed.
    #[inline]
    pub fn is_authenticated(&self) -> bool {
        self.auth_state.is_authenticated()
    }
}

/// Strongly-typed generational token identifying a specific client connection.
///
/// Immune to Slot Recycling Hazards: if player Alice in slot 1 disconnects and player Bob
/// connects to slot 1, Bob will have a higher generation count, invalidating Alice's tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerSessionToken {
    /// Slot index of the client (1..=32).
    pub slot: i32,
    /// Monotonically increasing connection generation counter.
    pub generation: u64,
    /// Engine user ID (`pfnGetPlayerUserId`) if known.
    pub user_id: u32,
}

impl PlayerSessionToken {
    /// Creates a new session token.
    #[inline(always)]
    pub const fn new(slot: i32, generation: u64, user_id: u32) -> Self {
        Self {
            slot,
            generation,
            user_id,
        }
    }

    /// Slot index of the player (1..=32).
    #[inline(always)]
    pub const fn slot(&self) -> i32 {
        self.slot
    }

    /// Generation counter for the connection.
    #[inline(always)]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Engine user ID if assigned.
    #[inline(always)]
    pub const fn user_id(&self) -> u32 {
        self.user_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steam_id_parse_and_conversions() {
        let steam2 = "STEAM_0:1:12345678";
        let id = SteamId::parse(steam2).expect("valid Steam2");
        assert_eq!(id.auth_server(), 1);
        assert_eq!(id.auth_id(), 12345678);
        assert_eq!(id.account_id(), 24691357);
        assert_eq!(id.to_steam2(), "STEAM_0:1:12345678");
        assert_eq!(id.to_steam3(), "[U:1:24691357]");
        assert_eq!(id.steam64(), 76561197960265728 + 24691357);

        let id3 = SteamId::parse("[U:1:24691357]").expect("valid Steam3");
        assert_eq!(id3, id);

        let id64 = SteamId::parse(&format!("{}", id.steam64())).expect("valid Steam64");
        assert_eq!(id64, id);

        let id_acc = SteamId::parse("24691357").expect("valid account ID");
        assert_eq!(id_acc, id);

        let id_valve = SteamId::parse("VALVE_0:0:98765").expect("valid VALVE_ prefix");
        assert_eq!(id_valve.auth_server(), 0);
        assert_eq!(id_valve.auth_id(), 98765);
        assert_eq!(id_valve.account_id(), 197530);

        assert!(SteamId::parse("").is_none());
        assert!(SteamId::parse("BOT").is_none());
        assert!(SteamId::parse("STEAM_ID_PENDING").is_none());
        assert!(SteamId::parse("STEAM_ID_LAN").is_none());
        assert!(SteamId::parse("random_garbage").is_none());
    }

    #[test]
    fn test_player_guid_deterministic() {
        let steam_id = SteamId::from_account_id(42);
        let guid1 = PlayerGuid::for_steam(steam_id.steam64());
        let guid2 = PlayerGuid::for_steam(steam_id.steam64());
        assert_eq!(guid1, guid2);
        assert_ne!(guid1, PlayerGuid::NIL);

        let bot1 = PlayerGuid::for_bot("Expert Bot");
        let bot2 = PlayerGuid::for_bot("Expert Bot");
        let bot3 = PlayerGuid::for_bot("Noob Bot");
        assert_eq!(bot1, bot2);
        assert_ne!(bot1, bot3);

        let ip: std::net::IpAddr = "192.168.1.100".parse().unwrap();
        let lan1 = PlayerGuid::for_lan(ip);
        let lan2 = PlayerGuid::for_lan(ip);
        assert_eq!(lan1, lan2);

        let hex = guid1.to_hex();
        assert_eq!(hex.len(), 32);
        let parsed_hex = PlayerGuid::from_hex(&hex).expect("valid hex");
        assert_eq!(parsed_hex, guid1);

        let uuid_str = guid1.to_uuid_string();
        assert_eq!(uuid_str.len(), 36);
        let parsed_uuid = PlayerGuid::from_hex(&uuid_str).expect("valid uuid hex");
        assert_eq!(parsed_uuid, guid1);
    }

    #[test]
    fn test_auth_state_lifecycle() {
        let mut state = AuthState::Pending;
        assert!(!state.is_authenticated());
        assert_eq!(state.guid(), None);
        assert_eq!(state.steam_id(), None);

        let steam_id = SteamId::from_account_id(12345);
        state = AuthState::Authenticated(AuthSubject::steam(steam_id));
        assert!(state.is_authenticated());
        assert_eq!(state.steam_id(), Some(steam_id));
        assert_eq!(
            state.guid(),
            Some(PlayerGuid::for_steam(steam_id.steam64()))
        );
    }
}
