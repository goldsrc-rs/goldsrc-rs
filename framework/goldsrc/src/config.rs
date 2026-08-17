use crate::logging::LogConfig;
use crate::paths::PathResolver;
use goldsrc_api::consts::BackendType;
use serde::{Deserialize, Serialize};
use std::fs;

/// The `[core]` section of `goldsrc.toml` — filesystem layout paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Mod directory relative to the game root (e.g. `cstrike`).
    pub mod_dir: String,
    /// Directory holding WASM plugins.
    pub plugins_dir: String,
    /// Directory holding per-plugin configs.
    pub configs_dir: String,
    /// Directory holding log output.
    pub logs_dir: String,
}

impl CoreConfig {
    pub fn new(backend: BackendType) -> Self {
        let fw_dir = PathResolver::framework_dir(backend);
        Self {
            mod_dir: goldsrc_api::consts::DEFAULT_MOD_DIR.to_string(),
            plugins_dir: PathResolver::normalize(&fw_dir.join(crate::paths::PLUGINS_DIR_NAME)),
            configs_dir: PathResolver::normalize(&fw_dir.join(crate::paths::CONFIGS_DIR_NAME)),
            logs_dir: PathResolver::normalize(&fw_dir.join(crate::paths::LOGS_DIR_NAME)),
        }
    }
}

/// The `[wasm]` section of `goldsrc.toml` — WASM runtime behaviour.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WasmConfig {
    /// Watch the plugins dir and auto-reload changed `.wasm` files.
    #[serde(default)]
    pub hot_reload: bool,
    /// Watch the configs dir for `.toml` changes.
    #[serde(default)]
    pub config_watcher: bool,
}

/// Top-level `goldsrc.toml` configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldSrcConfig {
    /// Filesystem layout paths.
    pub core: CoreConfig,
    /// WASM runtime behaviour.
    #[serde(default)]
    pub wasm: WasmConfig,
    /// Logger settings.
    #[serde(default)]
    pub logging: LogConfig,
}

impl GoldSrcConfig {
    /// Loads `goldsrc.toml` from the resolved base path or creates a default one if missing.
    ///
    /// If an existing configuration file fails to parse, a backup (`.bak`) is created
    /// before generating a fresh default configuration to prevent data loss.
    pub fn load_or_create(backend: BackendType) -> Self {
        let config_path = PathResolver::main_config_path(backend);
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str::<Self>(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        log::warn!(
                            target: "core",
                            "Failed to parse existing configuration file '{}': {}. Creating backup...",
                            config_path.display(),
                            e
                        );
                        let bak_path = config_path.with_extension("toml.bak");
                        if let Err(copy_err) = fs::copy(&config_path, &bak_path) {
                            log::error!(
                                target: "core",
                                "Failed to create backup config at '{}': {}",
                                bak_path.display(),
                                copy_err
                            );
                        } else {
                            log::info!(
                                target: "core",
                                "Existing configuration backed up to '{}'",
                                bak_path.display()
                            );
                        }
                    }
                },
                Err(e) => {
                    log::error!(
                        target: "core",
                        "Failed to read configuration file '{}': {}",
                        config_path.display(),
                        e
                    );
                }
            }
        }

        let config = Self {
            core: CoreConfig::new(backend),
            wasm: WasmConfig::default(),
            logging: LogConfig::default(),
        };

        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(toml_str) = toml::to_string_pretty(&config) {
            let _ = fs::write(&config_path, toml_str);
        }
        config
    }
}
