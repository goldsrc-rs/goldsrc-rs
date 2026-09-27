//! Global constants for the GoldSrc engine and framework.

/// Maximum number of players supported by the GoldSrc engine.
pub const MAX_PLAYERS: u16 = 32;

/// Maximum number of entity edicts in GoldSrc engine.
pub const MAX_EDICTS: u16 = 2048;

/// Maximum payload size in bytes for a single user network message.
pub const MAX_USER_MSG_DATA_LEN: usize = 192;

/// Maximum payload size for SayText user messages (192 - 12 bytes header/sender/NUL).
pub const MAX_SAYTEXT_PAYLOAD_LEN: usize = 180;

/// Safe payload limit for single-chunk chat messages (180 - 5 bytes safety margin).
pub const SAFE_SAYTEXT_LIMIT: usize = MAX_SAYTEXT_PAYLOAD_LEN - 5;

/// Standard engine interface version (`DLL_FUNCTIONS`).
pub const ENGINE_INTERFACE_VERSION: i32 = 140;

/// Standard NEW_DLL_FUNCTIONS interface version.
pub const NEW_DLL_INTERFACE_VERSION: i32 = 1;

/// Client print destination: Console (HLSDK `print_console = 0`).
pub const PRINT_CONSOLE: i32 = 0;

/// Client print destination: Center message (HLSDK `print_center = 1`).
pub const PRINT_CENTER: i32 = 1;

/// Client print destination: Chat (HLSDK `print_chat = 2`).
pub const PRINT_CHAT: i32 = 2;

/// Client print destination: Notify / developer print (HLSDK `print_notify = 1`).
pub const PRINT_NOTIFY: i32 = 1;

/// HUD / TextMsg print destination: Notify / developer print (HLSDK `HUD_PRINTNOTIFY = 1`).
pub const HUD_PRINTNOTIFY: i32 = 1;

/// HUD / TextMsg print destination: Console (HLSDK `HUD_PRINTCONSOLE = 2`).
pub const HUD_PRINTCONSOLE: i32 = 2;

/// HUD / TextMsg print destination: Chat (HLSDK `HUD_PRINTTALK = 3`).
pub const HUD_PRINTCHAT: i32 = 3;

/// HUD / TextMsg print destination: Center message (HLSDK `HUD_PRINTCENTER = 4`).
pub const HUD_PRINTCENTER: i32 = 4;

/// HUD / TextMsg print destination: Radio chat (HLSDK `HUD_PRINTRADIO = 5`).
pub const HUD_PRINTRADIO: i32 = 5;
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
// Entity Flags (edict->v.flags)
// ----------------------------------------------------------------------------

/// Entity flag indicating this edict is a connected client (`FL_CLIENT` = `1 << 3`).
pub const FL_CLIENT: i32 = 1 << 3;

/// Entity flag indicating this edict is a simulated bot/fake client (`FL_FAKECLIENT` = `1 << 13`).
pub const FL_FAKECLIENT: i32 = 1 << 13;

/// Entity flag indicating this edict is an HLTV spectator proxy (`FL_PROXY` = `1 << 10`).
pub const FL_PROXY: i32 = 1 << 10;

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

/// Standard backend binaries library directory name (`lib/`, replacing legacy `bin/`).
pub const LIB_DIR_NAME: &str = "lib";

/// Standard bundles directory name.
pub const BUNDLES_DIR_NAME: &str = "bundles";

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

/// Fallback plugin requires string if none are specified.
pub const DEFAULT_PLUGIN_REQUIRES: &str = "none";

// ----------------------------------------------------------------------------
// Sandbox Permissions Constants
// ----------------------------------------------------------------------------

pub mod permissions {
    /// Grants all permissions without restriction.
    pub const ALL: &str = "*";

    /// Grants all cvar capabilities.
    pub const CVAR_ALL: &str = "cvar:*";
    /// Allows reading engine cvars.
    pub const CVAR_GET: &str = "cvar:get";
    /// Allows modifying engine cvars.
    pub const CVAR_SET: &str = "cvar:set";

    /// Grants all filesystem and storage capabilities.
    pub const FS_ALL: &str = "fs:*";
    /// Allows reading files from storage.
    pub const FS_READ: &str = "fs:read";
    /// Allows writing files or persistent data to storage.
    pub const FS_WRITE: &str = "fs:write";
    /// Allows accessing shared storage buckets across plugins.
    pub const STORAGE_SHARED: &str = "storage:shared";

    /// Allows creating new world entities.
    pub const ENTITY_CREATE: &str = "entity:create";
    /// Allows removing or deleting entities from the world.
    pub const ENTITY_REMOVE: &str = "entity:remove";

    /// Allows broadcasting chat messages to all players on the server.
    pub const CHAT_BROADCAST: &str = "chat:broadcast";

    /// Allows issuing raw console commands to the server engine.
    pub const SERVER_COMMAND: &str = "server:command";
}

// ----------------------------------------------------------------------------
// Standard Logging & Tracing Subsystem Targets
// ----------------------------------------------------------------------------

/// Canonical subsystem logging targets used across the GoldSrc.rs framework.
pub mod log_targets {
    /// Framework core (init, lifecycle, configuration).
    pub const CORE: &str = "core";
    /// Authentication, capabilities, and permissions.
    pub const AUTH: &str = "auth";
    /// Persistence, SQLite, and KV storage engine.
    pub const STORAGE: &str = "storage";
    /// Declarative reactive rules engine.
    pub const RULES: &str = "rules";
    /// Menu presentation and user sessions.
    pub const MENU: &str = "menu";
    /// Internationalization dictionary compiler and placeholder expansion.
    pub const I18N: &str = "i18n";
    /// Entity Component System (ECS) world, stages, and systems.
    pub const ECS: &str = "ecs";
    /// WASM host runtime and component manager.
    pub const WASM: &str = "wasm";
    /// GameDLL proxy layer (standalone backend).
    pub const PROXY: &str = "proxy";
    /// Metamod engine interface bridge and precache manager.
    pub const ENGINE: &str = "engine";
    /// ReHLDS/ReGameDLL interface bindings.
    pub const REAPI: &str = "reapi";
    /// Filesystem watcher service and hot-reload debouncer.
    pub const WATCHER: &str = "watcher";
    /// Engine and plugin event bus.
    pub const EVENTS: &str = "events";
    /// Individual guest WASM plugins.
    pub const PLUGIN: &str = "plugin";
}
