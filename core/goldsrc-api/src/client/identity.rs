//! Strongly-typed player identity, network address, and SteamID representations.

use std::fmt;

/// Strongly typed SteamID or player authentication identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SteamId {
    /// Classic Steam2 format, e.g. `STEAM_0:1:12345678`.
    Steam2(String),
    /// Modern Steam3 format, e.g. `[U:1:24691356]`.
    Steam3(String),
    /// Dedicated server loopback or HLTV client.
    Server,
    /// Automated bot or local fakeclient.
    Bot,
    /// LAN client without Steam authentication.
    Lan,
    /// Pending authentication during connection handshake.
    #[default]
    Pending,
    /// Custom or unrecognized authentication string (e.g. non-standard emulators).
    Custom(String),
}

impl SteamId {
    /// Parses a raw auth ID string from the engine into a strongly-typed `SteamId`.
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("STEAM_ID_PENDING") {
            Self::Pending
        } else if trimmed.eq_ignore_ascii_case("BOT") || trimmed.eq_ignore_ascii_case("BOT_CLIENT")
        {
            Self::Bot
        } else if trimmed.eq_ignore_ascii_case("STEAM_ID_LAN")
            || trimmed.eq_ignore_ascii_case("VALVE_ID_LAN")
        {
            Self::Lan
        } else if trimmed.eq_ignore_ascii_case("server") || trimmed.eq_ignore_ascii_case("HLTV") {
            Self::Server
        } else if trimmed.starts_with("STEAM_") || trimmed.starts_with("VALVE_") {
            Self::Steam2(trimmed.to_string())
        } else if trimmed.starts_with("[U:") {
            Self::Steam3(trimmed.to_string())
        } else {
            Self::Custom(trimmed.to_string())
        }
    }

    /// Returns the canonical string representation of this SteamID.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Steam2(s) | Self::Steam3(s) | Self::Custom(s) => s.as_str(),
            Self::Server => "HLTV",
            Self::Bot => "BOT",
            Self::Lan => "STEAM_ID_LAN",
            Self::Pending => "STEAM_ID_PENDING",
        }
    }

    /// Returns `true` if authentication is completed and represents an authenticated account.
    pub fn is_authenticated(&self) -> bool {
        matches!(self, Self::Steam2(_) | Self::Steam3(_))
    }
}

impl fmt::Display for SteamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for SteamId {
    fn from(s: &str) -> Self {
        Self::parse(s)
    }
}

impl From<String> for SteamId {
    fn from(s: String) -> Self {
        Self::parse(&s)
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
    /// Parsed Steam ID / Auth ID (`pfnGetPlayerAuthId`).
    pub steam_id: SteamId,
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
    /// Returns the Steam ID string representation or "STEAM_ID_PENDING".
    #[inline]
    pub fn auth_id(&self) -> &str {
        self.steam_id.as_str()
    }

    /// Returns the IP string or fallback to localhost if unavailable.
    #[inline]
    pub fn ip_str(&self) -> &str {
        self.ip.as_deref().unwrap_or("127.0.0.1")
    }
}
