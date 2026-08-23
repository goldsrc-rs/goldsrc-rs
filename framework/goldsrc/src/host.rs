use crate::{GoldSrcConfig, paths::PathResolver};
use goldsrc_api::consts::BackendType;
use goldsrc_wasm_host::PluginManager;
use goldsrc_wasm_host::error::HostError;

pub struct HostRuntime {
    manager: PluginManager,
    engine: std::sync::Arc<dyn goldsrc_api::Engine>,
}

use std::sync::{Mutex, OnceLock};

static RUNTIME: OnceLock<Mutex<HostRuntime>> = OnceLock::new();

impl HostRuntime {
    /// Initialize the host runtime, logger, configuration and hot reload watchers.
    ///
    /// `engine` is the backend's [`goldsrc_api::Engine`] bridge — it gives
    /// WASM plugins access to the real game state. Call once at backend init.
    pub fn init(
        backend: BackendType,
        print_cb: fn(&str),
        engine: std::sync::Arc<dyn goldsrc_api::Engine>,
    ) -> Result<(), HostError> {
        let backend_name = match backend {
            BackendType::Metamod => "Metamod",
            BackendType::Standalone => "Standalone",
        };
        goldsrc_wasm_host::set_print_callback(print_cb);

        let mut manager = PluginManager::new(engine.clone())
            .map_err(|e| HostError::Manager(format!("[GoldSrc.rs {backend_name}] {e}")))?;

        let sys_config = GoldSrcConfig::load_or_create(backend);

        // Initialise unified logger
        let logs_dir = std::path::PathBuf::from(&sys_config.core.logs_dir);
        crate::logging::init_with_dir(
            sys_config.logging.clone(),
            Some(logs_dir),
            backend,
            Some(print_cb),
        );

        // Initial startup banner stating active backend and version
        log::info!(
            target: "core",
            "GoldSrc.rs v{} initialized (Backend: {})",
            env!("CARGO_PKG_VERSION"),
            backend_name
        );

        let main_cfg_path = PathResolver::main_config_path(backend);
        log::info!(
            target: "core",
            "Config loaded from: \"{}\"",
            PathResolver::normalize(&main_cfg_path)
        );

        let plugin_dir = std::path::PathBuf::from(&sys_config.core.plugins_dir);
        let config_dir = std::path::PathBuf::from(&sys_config.core.configs_dir);

        log::info!(
            target: "core",
            "Plugin dir: \"{}\"",
            PathResolver::normalize(&plugin_dir)
        );
        log::info!(
            target: "core",
            "Config dir: \"{}\"",
            PathResolver::normalize(&config_dir)
        );

        if sys_config.wasm.hot_reload
            && let Err(e) = manager.enable_hot_reload(&plugin_dir)
        {
            log::warn!(target: "wasm", "Failed to enable hot reload on {:?}: {e}", plugin_dir);
        }
        if sys_config.wasm.config_watcher
            && let Err(e) = manager.enable_config_watcher(&config_dir)
        {
            log::warn!(target: "wasm", "Failed to enable config watcher on {:?}: {e}", config_dir);
        }

        // Auto-load all .wasm plugins in plugin_dir
        if let Ok(entries) = std::fs::read_dir(&plugin_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_default();
                    match manager.load_plugin(&path) {
                        Ok(_) => log::info!(
                            target: "wasm",
                            "Loaded plugin: \"{}\"",
                            file_name
                        ),
                        Err(e) => log::error!(
                            target: "wasm",
                            "Failed to load \"{}\": {e}",
                            file_name
                        ),
                    }
                }
            }
        }

        let runtime = Self { manager, engine };
        let _ = RUNTIME.set(Mutex::new(runtime));
        Ok(())
    }

    /// Returns a clone of the Engine reference if initialized.
    pub fn engine() -> Option<std::sync::Arc<dyn goldsrc_api::Engine>> {
        RUNTIME
            .get()
            .and_then(|lock| lock.lock().ok().map(|g| g.engine.clone()))
    }

    /// Run `f` with exclusive access to the `PluginManager`, if initialized.
    pub fn with_manager<R>(f: impl FnOnce(Option<&mut PluginManager>) -> R) -> R {
        if let Some(lock) = RUNTIME.get() {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            f(Some(&mut guard.manager))
        } else {
            f(None)
        }
    }

    /// Tick plugins frame event.
    pub fn on_server_frame() {
        Self::with_manager(|m| {
            if let Some(manager) = m {
                manager.on_server_frame();
            }
        });
    }
}
