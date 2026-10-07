//! Menu frontend configuration model derived into TOML file format and CVAR registrations.

use goldsrc::{ConfigModel, CvarFlags};

#[derive(Debug, Clone, PartialEq, ConfigModel)]
#[config(cvar_prefix = "grs_menu_")]
pub struct MenuFrontendConfig {
    /// Header title displayed atop the main server menu
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER)]
    pub title: String,

    /// Maximum number of selectable items per menu page (1..8)
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 1..=8)]
    pub page_size: i32,

    /// Minimum debounce interval in milliseconds between button presses
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 50..=1000)]
    pub debounce_ms: i32,

    /// Auto-close timeout in seconds for idle menus
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 5..=300)]
    pub timeout_secs: i32,
}

impl Default for MenuFrontendConfig {
    fn default() -> Self {
        Self {
            title: "Главное Меню Сервера".to_string(),
            page_size: 7,
            debounce_ms: 150,
            timeout_secs: 30,
        }
    }
}
