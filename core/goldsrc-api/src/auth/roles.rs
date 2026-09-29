//! Type-safe role capability aliases and namespace constants.

/// Standard root capability namespace containers.
pub mod namespaces {
    /// Engine-level operational capabilities (`engine:*`).
    pub const ENGINE: &str = "engine";
    /// Internal ECS stage and system capabilities (`system:*`).
    pub const SYSTEM: &str = "system";
    /// Chat channel interception and broadcast capabilities (`chat:*`).
    pub const CHAT: &str = "chat";
    /// Menu presentation and slot action capabilities (`menu:*`).
    pub const MENU: &str = "menu";
    /// Gameplay modifiers and player intervention capabilities (`gameplay:*`).
    pub const GAMEPLAY: &str = "gameplay";
    /// Bundle-scoped capability container prefix (`bundle:<id>:*`).
    pub const BUNDLE: &str = "bundle";

    /// Checks if a string identifier matches one of the canonical root namespaces.
    #[inline]
    pub fn is_root_namespace(ns: &str) -> bool {
        matches!(ns, ENGINE | SYSTEM | CHAT | MENU | GAMEPLAY | BUNDLE)
    }
}

pub use namespaces::is_root_namespace;

/// Predefined composite capability sets for standard administrative roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AdminCaps;

impl AdminCaps {
    /// Full administrative access wildcard.
    pub const ALL: &'static str = "admin:*";
    /// Authority to grant capabilities to other players.
    pub const GRANT: &'static str = "admin.grant";
    /// Authority to instantly slay players.
    pub const SLAY: &'static str = "admin.slay";
    /// Authority to teleport players across map coordinates.
    pub const TELEPORT: &'static str = "admin.teleport";
    /// Authority to slap players with damage/shake.
    pub const SLAP: &'static str = "admin.slap";
    /// Authority to ban players from the server.
    pub const BAN: &'static str = "admin.ban";
    /// Authority to kick players from the server.
    pub const KICK: &'static str = "admin.kick";
    /// Authority to start server votes.
    pub const VOTE: &'static str = "admin.vote";
    /// Authority to modify server CVARs.
    pub const CVAR: &'static str = "admin.cvar";
    /// Authority to inspect host hardware and diagnostics telemetry.
    pub const SYSINFO: &'static str = "admin.sysinfo";
    /// Access to private admin chat channel.
    pub const CHAT: &'static str = "chat:channel(admin)";

    /// Returns standard administrator default capabilities.
    pub fn default_caps() -> &'static [&'static str] {
        &[
            Self::GRANT,
            Self::SLAY,
            Self::TELEPORT,
            Self::SLAP,
            Self::BAN,
            Self::KICK,
            Self::VOTE,
            Self::CVAR,
            Self::SYSINFO,
            Self::CHAT,
        ]
    }
}

/// Predefined composite capability sets for VIP players.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VipCaps;

impl VipCaps {
    /// Full VIP access wildcard.
    pub const ALL: &'static str = "vip:*";
    /// Authority to use VIP health recovery.
    pub const HEAL: &'static str = "gameplay:heal";
    /// Authority to receive VIP bonus armor.
    pub const ARMOR: &'static str = "gameplay:armor";
    /// Access to dedicated VIP menu.
    pub const MENU: &'static str = "menu:scope(vip)";
    /// Access to VIP chat channel.
    pub const CHAT: &'static str = "chat:channel(vip)";

    /// Returns standard VIP default capabilities.
    pub fn default_caps() -> &'static [&'static str] {
        &[Self::HEAL, Self::ARMOR, Self::MENU, Self::CHAT]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_capabilities() {
        assert_eq!(AdminCaps::ALL, "admin:*");
        assert_eq!(AdminCaps::CHAT, "chat:channel(admin)");
        assert_eq!(VipCaps::MENU, "menu:scope(vip)");
        assert_eq!(AdminCaps::default_caps().len(), 10);
        assert_eq!(VipCaps::default_caps().len(), 4);
        assert!(namespaces::is_root_namespace("chat"));
        assert!(namespaces::is_root_namespace("gameplay"));
        assert!(!namespaces::is_root_namespace("admin"));
    }
}
