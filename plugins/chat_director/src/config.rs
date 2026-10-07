//! Chat director configuration model derived into TOML file format and CVAR registrations.

use goldsrc::{ConfigModel, CvarFlags};

#[derive(Debug, Clone, PartialEq, ConfigModel)]
#[config(cvar_prefix = "grs_chat_")]
pub struct ChatDirectorConfig {
    /// Minimum interval in seconds between messages to prevent spam
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 0.1..=10.0)]
    pub flood_interval: f32,

    /// Interval in seconds between rotating informational broadcast messages
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 5.0..=600.0)]
    pub broadcast_interval: f32,

    /// Enable say_team @ prefix routing to online staff
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER)]
    pub enable_staff_channel: bool,

    /// Render rotating announcements in Director HUD banners
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER)]
    pub enable_dhud_banners: bool,
}

impl Default for ChatDirectorConfig {
    fn default() -> Self {
        Self {
            flood_interval: 0.75,
            broadcast_interval: 60.0,
            enable_staff_channel: true,
            enable_dhud_banners: true,
        }
    }
}
