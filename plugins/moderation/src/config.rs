//! Moderation configuration model derived into TOML file format and CVAR registrations.

use goldsrc::{ConfigModel, CvarFlags};

#[derive(Debug, Clone, PartialEq, ConfigModel)]
#[config(cvar_prefix = "grs_mod_")]
pub struct ModerationConfig {
    /// Default duration in minutes for player bans (0 = permanent)
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 0..=525600)]
    pub default_ban_mins: i32,

    /// Default damage applied by grs_slap command
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 0.0..=100.0)]
    pub default_slap_damage: f32,

    /// Maximum duration in seconds allowed for grs_freeze inspection
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 5..=600)]
    pub max_freeze_secs: i32,

    /// Broadcast disciplinary actions to global server chat
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER)]
    pub notify_chat: bool,

    /// Persist disciplinary audit logs into storage
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER)]
    pub log_audit: bool,
}

impl Default for ModerationConfig {
    fn default() -> Self {
        Self {
            default_ban_mins: 60,
            default_slap_damage: 0.0,
            max_freeze_secs: 120,
            notify_chat: true,
            log_audit: true,
        }
    }
}
