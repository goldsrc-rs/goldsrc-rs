//! Map manager configuration model derived into TOML file format and CVAR registrations.

use goldsrc::{ConfigModel, CvarFlags};

#[derive(Debug, Clone, PartialEq, ConfigModel)]
#[config(cvar_prefix = "grs_map_")]
pub struct MapManagerConfig {
    /// Minutes before map end to automatically trigger end-of-map vote
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 0.5..=30.0)]
    pub vote_trigger_mins: f32,

    /// Duration in seconds for interactive map vote polling
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 5..=120)]
    pub vote_duration_secs: i32,

    /// Number of recently played maps blocked from rotation and nominations
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 0..=20)]
    pub block_recent_count: i32,

    /// Comma-separated default pool of maps in rotation
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER)]
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
