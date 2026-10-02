//! Chat director configuration model derived into TOML file format and CVAR registrations.

use goldsrc::ConfigModel;

#[derive(Debug, Clone, PartialEq, ConfigModel)]
pub struct ChatDirectorConfig {
    #[cvar(
        name = "grs_chat_flood_interval",
        flags = "ARCHIVE|SERVER",
        description = "Minimum interval in seconds between messages to prevent spam"
    )]
    pub flood_interval: f32,

    #[cvar(
        name = "grs_chat_broadcast_interval",
        flags = "ARCHIVE|SERVER",
        description = "Interval in seconds between rotating informational broadcast messages"
    )]
    pub broadcast_interval: f32,

    #[cvar(
        name = "grs_chat_enable_staff_channel",
        flags = "ARCHIVE|SERVER",
        description = "Enable say_team @ prefix routing to online staff (1 = yes, 0 = no)"
    )]
    pub enable_staff_channel: i32,

    #[cvar(
        name = "grs_chat_enable_dhud_banners",
        flags = "ARCHIVE|SERVER",
        description = "Render rotating announcements in Director HUD banners (1 = yes, 0 = no)"
    )]
    pub enable_dhud_banners: i32,
}

impl Default for ChatDirectorConfig {
    fn default() -> Self {
        Self {
            flood_interval: 0.75,
            broadcast_interval: 60.0,
            enable_staff_channel: 1,
            enable_dhud_banners: 1,
        }
    }
}
