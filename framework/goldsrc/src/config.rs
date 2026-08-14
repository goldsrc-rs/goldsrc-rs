use crate::logging::LogConfig;
use crate::paths::{PathResolver, ADDONS_DIR_NAME, DEFAULT_MOD_DIR, FRAMEWORK_NAME};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    #[serde(default = "default_mod_dir")]
    pub mod_dir: String,
    #[serde(default = "default_plugins_dir")]
    pub plugins_dir: String,
    #[serde(default = "default_configs_dir")]
    pub configs_dir: String,
    #[serde(default = "default_logs_dir")]
    pub logs_dir: String,
}

fn default_mod_dir() -> String {
    DEFAULT_MOD_DIR.to_string()
}
fn default_plugins_dir() -> String {
    format!(
        "{}/{}/{}/plugins",
        DEFAULT_MOD_DIR, ADDONS_DIR_NAME, FRAMEWORK_NAME
    )
}
fn default_configs_dir() -> String {
    format!(
        "{}/{}/{}/configs",
        DEFAULT_MOD_DIR, ADDONS_DIR_NAME, FRAMEWORK_NAME
    )
}
fn default_logs_dir() -> String {
    format!(
        "{}/{}/{}/logs",
        DEFAULT_MOD_DIR, ADDONS_DIR_NAME, FRAMEWORK_NAME
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    #[serde(default = "default_true")]
    pub hot_reload: bool,
    #[serde(default = "default_true")]
    pub config_watcher: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoldSrcConfig {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub wasm: WasmConfig,
    #[serde(default)]
    pub logging: LogConfig,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            mod_dir: default_mod_dir(),
            plugins_dir: default_plugins_dir(),
            configs_dir: default_configs_dir(),
            logs_dir: default_logs_dir(),
        }
    }
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            hot_reload: true,
            config_watcher: true,
        }
    }
}

impl GoldSrcConfig {
    /// Loads `goldsrc.toml` from the resolved base path or creates a default one if missing.
    pub fn load_or_create() -> Self {
        let config_path = PathResolver::main_config_path();
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str::<Self>(&content) {
                    return config;
                }
            }
        }

        let config = Self::default();
        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(toml_str) = toml::to_string_pretty(&config) {
            let _ = fs::write(&config_path, toml_str);
        }
        config
    }
}
