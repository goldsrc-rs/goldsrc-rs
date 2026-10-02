//! Administration configuration model derived into TOML file format and CVAR registrations.

use goldsrc::ConfigModel;

#[derive(Debug, Clone, PartialEq, ConfigModel)]
pub struct AdministrationConfig {
    #[cvar(
        name = "grs_adm_default_match_cfg",
        flags = "ARCHIVE|SERVER",
        description = "Default configuration file executed for competitive matches"
    )]
    pub default_match_cfg: String,

    #[cvar(
        name = "grs_adm_default_warmup_cfg",
        flags = "ARCHIVE|SERVER",
        description = "Default configuration file executed during pre-match warmup"
    )]
    pub default_warmup_cfg: String,

    #[cvar(
        name = "grs_adm_restart_delay_secs",
        flags = "ARCHIVE|SERVER",
        description = "Default countdown delay in seconds for round restarts"
    )]
    pub restart_delay_secs: i32,

    #[cvar(
        name = "grs_adm_max_audit_fetch",
        flags = "ARCHIVE|SERVER",
        description = "Maximum number of audit log records returned by grs_audit"
    )]
    pub max_audit_fetch: i32,

    #[cvar(
        name = "grs_adm_notify_actions",
        flags = "ARCHIVE|SERVER",
        description = "Broadcast technical administrative actions to all players (1 = yes, 0 = no)"
    )]
    pub notify_actions: i32,
}

impl Default for AdministrationConfig {
    fn default() -> Self {
        Self {
            default_match_cfg: "clanwar.cfg".to_string(),
            default_warmup_cfg: "warmup.cfg".to_string(),
            restart_delay_secs: 1,
            max_audit_fetch: 20,
            notify_actions: 1,
        }
    }
}
