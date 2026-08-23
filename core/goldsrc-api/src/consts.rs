//! Global constants for the GoldSrc engine and framework.

/// Maximum number of players supported by the GoldSrc engine.
pub const MAX_PLAYERS: u16 = 32;

/// Maximum number of entity edicts in GoldSrc engine.
pub const MAX_EDICTS: u16 = 2048;

/// Standard engine interface version (`DLL_FUNCTIONS`).
pub const ENGINE_INTERFACE_VERSION: i32 = 140;

/// Standard NEW_DLL_FUNCTIONS interface version.
pub const NEW_DLL_INTERFACE_VERSION: i32 = 1;

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

/// Default name of the framework log file.
pub const DEFAULT_LOG_FILE_NAME: &str = "goldsrc.log";

/// Default name of the debug log file (used by proxy backend).
pub const DEBUG_LOG_FILE_NAME: &str = "debug.log";

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

/// Standard hosts directory name.
pub const HOSTS_DIR_NAME: &str = "hosts";

/// Standard WebAssembly binary file extension.
pub const WASM_EXT: &str = ".wasm";

// ----------------------------------------------------------------------------
// Network Protocol & User Messages
// ----------------------------------------------------------------------------

/// Maximum payload length for a single GoldSrc UserMessage buffer in bytes.
pub const MAX_USER_MSG_DATA_LEN: usize = 192;

/// Safe payload chunk size for `ShowMenu` multipart network messages.
pub const MAX_SHOW_MENU_CHUNK_SIZE: usize = 150;

/// Network message opcode for temporary entities (`SVC_TEMPENTITY`).
pub const SVC_TEMPENTITY: i32 = 23;

/// TempEntity sub-type for screen text messages (`TE_TEXTMESSAGE`).
pub const TE_TEXTMESSAGE: u8 = 29;

/// Network message opcode for Director HUD messages (`SVC_DIRECTOR`).
pub const SVC_DIRECTOR: i32 = 10;

/// Director command sub-opcode for screen text messages (`DRC_CMD_MESSAGE`).
pub const DRC_CMD_MESSAGE: u8 = 2;

// ----------------------------------------------------------------------------
// UI & Menu Engine Constants
// ----------------------------------------------------------------------------

/// Standard maximum number of menu slots per page (1..=10).
pub const MAX_MENU_SLOTS: u8 = 10;

/// Default items per page in paginated menus (slots 1..=7).
pub const DEFAULT_ITEMS_PER_PAGE: usize = 7;

/// Menu slot index for navigating to the previous page (Slot 8).
pub const MENU_SLOT_BACK: u8 = 8;

/// Menu slot index for navigating to the next page (Slot 9).
pub const MENU_SLOT_NEXT: u8 = 9;

/// Menu slot index for exiting or closing the menu (Slot 0 -> 10).
pub const MENU_SLOT_EXIT: u8 = 10;

/// Bitmask representing all 10 menu keys active (`1..=10`).
pub const MENU_KEY_ALL: u16 = 0x3FF;

// ----------------------------------------------------------------------------
// Engine Console & Client Print Types
// ----------------------------------------------------------------------------

/// Client print destination: Notify.
pub const PRINT_NOTIFY: i32 = 1;

/// Client print destination: Console.
pub const PRINT_CONSOLE: i32 = 2;

/// Client print destination: Chat.
pub const PRINT_CHAT: i32 = 3;

/// Client print destination: Center message.
pub const PRINT_CENTER: i32 = 4;
