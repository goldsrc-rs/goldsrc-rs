//! Global constants for the GoldSrc engine and framework.

pub use crate::engine::{
    ENGINE_INTERFACE_VERSION, HUD_PRINTCENTER, HUD_PRINTCHAT, HUD_PRINTCONSOLE, HUD_PRINTNOTIFY,
    HUD_PRINTRADIO, MAX_EDICTS, MAX_PLAYERS, MAX_USER_MSG_DATA_LEN, NEW_DLL_INTERFACE_VERSION,
    PRINT_CENTER, PRINT_CHAT, PRINT_CONSOLE, PRINT_NOTIFY,
};
pub use crate::hud::{
    DRC_CMD_MESSAGE, HUD_COORD_CENTER, MAX_HUD_CHANNELS, SVC_DIRECTOR, SVC_TEMPENTITY,
    TE_TEXTMESSAGE,
};
pub use crate::menu::{
    DEFAULT_ITEMS_PER_PAGE, MAX_MENU_SLOTS, MAX_SHOW_MENU_CHUNK_SIZE, MENU_KEY_ALL, MENU_SLOT_BACK,
    MENU_SLOT_EXIT, MENU_SLOT_NEXT,
};

/// Type of backend hosting the framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Metamod plugin backend (paths use `addons/`)
    Metamod,
    /// Standalone GameDLL proxy backend (paths use mod root)
    Standalone,
}

// ----------------------------------------------------------------------------
// File and Directory Names
// ----------------------------------------------------------------------------

/// Default name of the framework configuration file.
pub const DEFAULT_CONFIG_FILE_NAME: &str = "goldsrc.toml";

/// Default mod directory (e.g., "cstrike", "valve").
pub const DEFAULT_MOD_DIR: &str = "cstrike";

/// Standard Metamod addons directory name.
pub const ADDONS_DIR_NAME: &str = "addons";

/// Framework base directory name.
pub const FRAMEWORK_NAME: &str = "goldsrc";

/// Standard plugins directory name.
pub const PLUGINS_DIR_NAME: &str = "plugins";

/// Standard configs directory name.
pub const CONFIGS_DIR_NAME: &str = "configs";

/// Standard logs directory name.
pub const LOGS_DIR_NAME: &str = "logs";

/// Standard data directory name.
pub const DATA_DIR_NAME: &str = "data";

/// Standard localization dictionaries directory name within data.
pub const LANG_DIR_NAME: &str = "lang";

/// Standard database directory name within data.
pub const DB_DIR_NAME: &str = "db";

/// Default SQLite database filename.
pub const DEFAULT_DB_FILE_NAME: &str = "goldsrc.db";

/// Standard hosts directory name.
pub const HOSTS_DIR_NAME: &str = "hosts";

/// Standard WebAssembly binary file extension.
pub const WASM_EXT: &str = ".wasm";

// ----------------------------------------------------------------------------
// Plugin Metadata Fallback Constants
// ----------------------------------------------------------------------------

/// Fallback plugin display name if not specified.
pub const DEFAULT_PLUGIN_NAME: &str = "Unknown";

/// Fallback plugin version string if not specified.
pub const DEFAULT_PLUGIN_VERSION: &str = "0.0.0";

/// Fallback plugin author string if not specified.
pub const DEFAULT_PLUGIN_AUTHOR: &str = "Unknown";

/// Fallback plugin description string if not specified.
pub const DEFAULT_PLUGIN_DESCRIPTION: &str = "No description provided";

/// Fallback plugin license string if not specified.
pub const DEFAULT_PLUGIN_LICENSE: &str = "Not Stated";

/// Fallback plugin website or repository URL if not specified.
pub const DEFAULT_PLUGIN_URL: &str = "N/A";

/// Fallback plugin registered systems string if none are registered.
pub const DEFAULT_PLUGIN_SYSTEMS: &str = "none";

/// Fallback plugin require string if none are specified.
pub const DEFAULT_PLUGIN_REQUIRE: &str = "none";
