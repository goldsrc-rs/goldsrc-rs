//! Global constants for the GoldSrc engine and framework.

/// Maximum number of players supported by the GoldSrc engine.
pub const MAX_PLAYERS: u16 = 32;

/// Maximum number of entity edicts in GoldSrc engine.
pub const MAX_EDICTS: u16 = 2048;

/// Type of backend hosting the framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Metamod plugin backend (paths use `addons/`)
    Metamod,
    /// Standalone GameDLL proxy backend (paths use mod root)
    Standalone,
}

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

/// Framework base name.
pub const FRAMEWORK_NAME: &str = "goldsrc";
