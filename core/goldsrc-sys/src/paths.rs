use std::path::PathBuf;

/// Centralized framework constants for path resolution.
pub const FRAMEWORK_NAME: &str = "goldsrc";
pub const DEFAULT_MOD_DIR: &str = "cstrike";
pub const ADDONS_DIR_NAME: &str = "addons";
pub const PLUGINS_DIR_NAME: &str = "plugins";
pub const BIN_DIR_NAME: &str = "bin";
pub const CONFIGS_DIR_NAME: &str = "configs";
pub const LOGS_DIR_NAME: &str = "logs";

/// Centralized helper for resolving directory paths across HLDS environments.
pub struct PathResolver;

impl PathResolver {
    /// Returns possible plugin directory paths in order of preference.
    pub fn plugin_dirs() -> Vec<PathBuf> {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));
        let mut dirs = Vec::new();

        if let Some(ref base) = exe_dir {
            dirs.push(
                base.join(DEFAULT_MOD_DIR)
                    .join(ADDONS_DIR_NAME)
                    .join(FRAMEWORK_NAME)
                    .join(PLUGINS_DIR_NAME),
            );
            dirs.push(
                base.join(ADDONS_DIR_NAME)
                    .join(FRAMEWORK_NAME)
                    .join(PLUGINS_DIR_NAME),
            );
        }

        dirs.push(
            PathBuf::from(DEFAULT_MOD_DIR)
                .join(ADDONS_DIR_NAME)
                .join(FRAMEWORK_NAME)
                .join(PLUGINS_DIR_NAME),
        );
        dirs.push(
            PathBuf::from(ADDONS_DIR_NAME)
                .join(FRAMEWORK_NAME)
                .join(PLUGINS_DIR_NAME),
        );
        dirs
    }

    /// Returns the first existing plugin directory, or the primary default path.
    pub fn existing_plugin_dir() -> PathBuf {
        for dir in Self::plugin_dirs() {
            if dir.exists() {
                return dir;
            }
        }
        PathBuf::from(DEFAULT_MOD_DIR)
            .join(ADDONS_DIR_NAME)
            .join(FRAMEWORK_NAME)
            .join(PLUGINS_DIR_NAME)
    }

    /// Returns possible config directory paths in order of preference.
    pub fn config_dirs() -> Vec<PathBuf> {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));
        let mut dirs = Vec::new();

        if let Some(ref base) = exe_dir {
            dirs.push(
                base.join(DEFAULT_MOD_DIR)
                    .join(ADDONS_DIR_NAME)
                    .join(FRAMEWORK_NAME)
                    .join(CONFIGS_DIR_NAME),
            );
            dirs.push(
                base.join(ADDONS_DIR_NAME)
                    .join(FRAMEWORK_NAME)
                    .join(CONFIGS_DIR_NAME),
            );
        }

        dirs.push(
            PathBuf::from(DEFAULT_MOD_DIR)
                .join(ADDONS_DIR_NAME)
                .join(FRAMEWORK_NAME)
                .join(CONFIGS_DIR_NAME),
        );
        dirs.push(
            PathBuf::from(ADDONS_DIR_NAME)
                .join(FRAMEWORK_NAME)
                .join(CONFIGS_DIR_NAME),
        );
        dirs
    }

    /// Returns the first existing config directory, or the primary default path.
    pub fn existing_config_dir() -> PathBuf {
        for dir in Self::config_dirs() {
            if dir.exists() {
                return dir;
            }
        }
        PathBuf::from(DEFAULT_MOD_DIR)
            .join(ADDONS_DIR_NAME)
            .join(FRAMEWORK_NAME)
            .join(CONFIGS_DIR_NAME)
    }

    /// Returns the path to goldsrc.toml.
    pub fn main_config_path() -> PathBuf {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        if let Some(ref base) = exe_dir {
            let path = base
                .join(DEFAULT_MOD_DIR)
                .join(ADDONS_DIR_NAME)
                .join(FRAMEWORK_NAME)
                .join("goldsrc.toml");
            if path.exists() {
                return path;
            }
        }

        PathBuf::from(DEFAULT_MOD_DIR)
            .join(ADDONS_DIR_NAME)
            .join(FRAMEWORK_NAME)
            .join("goldsrc.toml")
    }

    /// Returns the path to debug.log.
    pub fn debug_log_path() -> PathBuf {
        PathBuf::from(DEFAULT_MOD_DIR)
            .join(ADDONS_DIR_NAME)
            .join(FRAMEWORK_NAME)
            .join("debug.log")
    }
}
