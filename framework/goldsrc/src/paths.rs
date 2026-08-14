use std::path::{Path, PathBuf};

/// Centralized framework constants for path resolution.
pub const FRAMEWORK_NAME: &str = "goldsrc";
/// Default mod directory (HLDS mod folder name).
pub const DEFAULT_MOD_DIR: &str = "cstrike";
/// HLDS addons directory name.
pub const ADDONS_DIR_NAME: &str = "addons";
/// Plugins sub-directory name.
pub const PLUGINS_DIR_NAME: &str = "plugins";
/// Binaries sub-directory name.
pub const BIN_DIR_NAME: &str = "bin";
/// Configs sub-directory name.
pub const CONFIGS_DIR_NAME: &str = "configs";
/// Logs sub-directory name.
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

    /// Normalizes a path to a consistent, human-readable string with forward
    /// slashes (`/`) on all platforms.
    ///
    /// - Resolves `.` and `..` components lexically (without touching the
    ///   filesystem), so the path does not need to exist.
    /// - Converts all backslashes to forward slashes (important on Windows).
    /// - Does **not** make the path absolute; relative paths stay relative.
    ///
    /// # Example
    /// ```ignore
    /// let p = PathBuf::from(r"cstrike\addons\goldsrc\.\plugins");
    /// assert_eq!(PathResolver::normalize(&p), "cstrike/addons/goldsrc/plugins");
    /// ```
    pub fn normalize(path: &Path) -> String {
        // Resolve . and .. without hitting the filesystem.
        let mut components: Vec<std::ffi::OsString> = Vec::new();
        for comp in path.components() {
            match comp {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    // Only pop if there is a normal component to go up from;
                    // never pop a prefix (drive letter) or root component.
                    if matches!(
                        components
                            .last()
                            .and_then(|c| { std::path::Path::new(c).components().next() }),
                        Some(std::path::Component::Normal(_))
                    ) {
                        components.pop();
                    }
                }
                other => components.push(other.as_os_str().to_os_string()),
            }
        }

        // Re-join and convert separators to forward slashes.
        let joined = components
            .iter()
            .enumerate()
            .fold(String::new(), |mut acc, (i, c)| {
                if i > 0 {
                    acc.push('/');
                }
                acc.push_str(&c.to_string_lossy());
                acc
            });

        // On Windows, backslashes may still appear inside individual components
        // (e.g. drive prefix). Replace any remaining ones.
        joined.replace('\\', "/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_forward_slashes_unchanged() {
        let p = PathBuf::from("cstrike/addons/goldsrc/plugins");
        assert_eq!(
            PathResolver::normalize(&p),
            "cstrike/addons/goldsrc/plugins"
        );
    }

    #[test]
    fn normalize_backslashes_converted() {
        // On Windows PathBuf may store backslashes; we always want forward slashes.
        let p = PathBuf::from(r"cstrike\addons\goldsrc\plugins");
        assert_eq!(
            PathResolver::normalize(&p),
            "cstrike/addons/goldsrc/plugins"
        );
    }

    #[test]
    fn normalize_strips_cur_dir() {
        let p = PathBuf::from("cstrike/./addons/./goldsrc");
        assert_eq!(PathResolver::normalize(&p), "cstrike/addons/goldsrc");
    }

    #[test]
    fn normalize_resolves_parent_dir() {
        let p = PathBuf::from("cstrike/addons/../goldsrc");
        assert_eq!(PathResolver::normalize(&p), "cstrike/goldsrc");
    }

    #[test]
    fn normalize_mixed_separators() {
        let p = PathBuf::from(r"cstrike\addons/goldsrc\..\goldsrc\plugins");
        assert_eq!(
            PathResolver::normalize(&p),
            "cstrike/addons/goldsrc/plugins"
        );
    }
}
