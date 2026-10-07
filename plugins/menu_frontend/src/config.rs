//! Menu frontend configuration model derived into TOML file format and CVAR registrations.

use goldsrc::ConfigModel;

#[derive(Debug, Clone, PartialEq, ConfigModel)]
pub struct MenuFrontendConfig {
    #[cvar(
        name = "grs_menu_title",
        flags = "ARCHIVE|SERVER",
        description = "Header title displayed atop the main server menu"
    )]
    pub title: String,

    #[cvar(
        name = "grs_menu_page_size",
        flags = "ARCHIVE|SERVER",
        description = "Maximum number of selectable items per menu page (1..8)"
    )]
    pub page_size: i32,

    #[cvar(
        name = "grs_menu_debounce_ms",
        flags = "ARCHIVE|SERVER",
        description = "Minimum debounce interval in milliseconds between button presses"
    )]
    pub debounce_ms: i32,

    #[cvar(
        name = "grs_menu_timeout_secs",
        flags = "ARCHIVE|SERVER",
        description = "Auto-close timeout in seconds for idle menus"
    )]
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
