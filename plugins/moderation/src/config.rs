//! Moderation configuration model derived into TOML file format and CVAR registrations.

use goldsrc::ConfigModel;

#[derive(Debug, Clone, PartialEq, ConfigModel)]
pub struct ModerationConfig {
    #[cvar(
        name = "grs_mod_default_ban_mins",
        flags = "ARCHIVE|SERVER",
        description = "Default duration in minutes for player bans (0 = permanent)"
    )]
    pub default_ban_mins: i32,

    #[cvar(
        name = "grs_mod_default_slap_damage",
        flags = "ARCHIVE|SERVER",
        description = "Default damage applied by grs_slap command"
    )]
    pub default_slap_damage: f32,

    #[cvar(
        name = "grs_mod_max_freeze_secs",
        flags = "ARCHIVE|SERVER",
        description = "Maximum duration in seconds allowed for grs_freeze inspection"
    )]
    pub max_freeze_secs: i32,

    #[cvar(
        name = "grs_mod_notify_chat",
        flags = "ARCHIVE|SERVER",
        description = "Broadcast disciplinary actions to global server chat (1 = yes, 0 = no)"
    )]
    pub notify_chat: i32,

    #[cvar(
        name = "grs_mod_log_audit",
        flags = "ARCHIVE|SERVER",
        description = "Persist disciplinary audit logs into storage (1 = yes, 0 = no)"
    )]
    pub log_audit: i32,
}

impl Default for ModerationConfig {
    fn default() -> Self {
        Self {
            default_ban_mins: 60,
            default_slap_damage: 0.0,
            max_freeze_secs: 120,
            notify_chat: 1,
            log_audit: 1,
        }
    }
}
