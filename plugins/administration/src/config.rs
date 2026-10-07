//! Administration configuration model derived into TOML file format and CVAR registrations.

use goldsrc::{ConfigModel, CvarFlags};

#[derive(Debug, Clone, PartialEq, ConfigModel)]
#[config(cvar_prefix = "grs_adm_")]
pub struct AdministrationConfig {
    /// Default configuration file executed for competitive matches
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER)]
    pub default_match_cfg: String,

    /// Default configuration file executed during pre-match warmup
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER)]
    pub default_warmup_cfg: String,

    /// Default countdown delay in seconds for round restarts
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 0..=60)]
    pub restart_delay_secs: i32,

    /// Maximum number of audit log records returned by grs_audit
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 1..=1000)]
    pub max_audit_fetch: i32,

    /// Broadcast technical administrative actions to all players
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER)]
    pub notify_actions: bool,
}

impl Default for AdministrationConfig {
    fn default() -> Self {
        Self {
            default_match_cfg: "clanwar.cfg".to_string(),
            default_warmup_cfg: "warmup.cfg".to_string(),
            restart_delay_secs: 1,
            max_audit_fetch: 20,
            notify_actions: true,
        }
    }
}
