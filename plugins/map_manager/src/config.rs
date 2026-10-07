//! Map manager configuration model derived into TOML file format and CVAR registrations.

use goldsrc::ConfigModel;

#[derive(Debug, Clone, PartialEq, ConfigModel)]
pub struct MapManagerConfig {
    #[cvar(
        name = "grs_map_vote_trigger_mins",
        flags = "ARCHIVE|SERVER",
        description = "Minutes before map end to automatically trigger end-of-map vote"
    )]
    pub vote_trigger_mins: f32,

    #[cvar(
        name = "grs_map_vote_duration_secs",
        flags = "ARCHIVE|SERVER",
        description = "Duration in seconds for interactive map vote polling"
    )]
    pub vote_duration_secs: i32,

    #[cvar(
        name = "grs_map_block_recent_count",
        flags = "ARCHIVE|SERVER",
        description = "Number of recently played maps blocked from rotation and nominations"
    )]
    pub block_recent_count: i32,

    #[cvar(
        name = "grs_map_default_pool",
        flags = "ARCHIVE|SERVER",
        description = "Comma-separated default pool of maps in rotation"
    )]
    pub default_pool: String,
}

impl Default for MapManagerConfig {
    fn default() -> Self {
        Self {
            vote_trigger_mins: 2.5,
            vote_duration_secs: 15,
            block_recent_count: 3,
            default_pool: "de_dust2,de_inferno,de_nuke,de_train,de_mirage,cs_assault".to_string(),
        }
    }
}
