//! Host runtime orchestrator for GoldSrc.rs backends.

use crate::logging::LogTarget;
use crate::{paths::PathResolver, GoldSrcConfig};
use goldsrc_wasm_host::PluginManager;

pub struct HostRuntime {
    manager: PluginManager,
}

thread_local! {
    static RUNTIME: std::cell::RefCell<Option<HostRuntime>> = const {
        std::cell::RefCell::new(None)
    };
}

impl HostRuntime {
    /// Initialize the host runtime, logger, configuration and hot reload watchers.
    ///
    /// Stores the runtime in a thread-local (the GoldSrc engine is single-threaded,
    /// so all hooks run on the same thread). Call once at backend init.
    pub fn init(backend_name: &str, print_cb: fn(&str)) -> Result<(), String> {
        goldsrc_wasm_host::set_print_callback(print_cb);

        let mut manager = PluginManager::new().map_err(|e| {
            format!("[GoldSrc.rs {backend_name}] Failed to init PluginManager: {e}")
        })?;

        let sys_config = GoldSrcConfig::load_or_create();

        // Initialise unified logger
        crate::logging::init(sys_config.logging.clone(), Some(print_cb));

        let main_cfg_path = PathResolver::main_config_path();
        crate::gslog_info!(
            LogTarget::Core,
            "[{}] Config loaded from: {}",
            backend_name,
            PathResolver::normalize(&main_cfg_path)
        );

        let plugin_dir = std::path::PathBuf::from(&sys_config.core.plugins_dir);
        let config_dir = std::path::PathBuf::from(&sys_config.core.configs_dir);

        crate::gslog_info!(
            LogTarget::Core,
            "[{}] Plugin dir: {}",
            backend_name,
            PathResolver::normalize(&plugin_dir)
        );
        crate::gslog_info!(
            LogTarget::Core,
            "[{}] Config dir: {}",
            backend_name,
            PathResolver::normalize(&config_dir)
        );

        if sys_config.wasm.hot_reload {
            let _ = manager.enable_hot_reload(&plugin_dir);
        }
        if sys_config.wasm.config_watcher {
            let _ = manager.enable_config_watcher(&config_dir);
        }

        // Auto-load all .wasm plugins in plugin_dir
        if let Ok(entries) = std::fs::read_dir(&plugin_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                    match manager.load_plugin(&path) {
                        Ok(_) => crate::gslog_info!(
                            LogTarget::Wasm,
                            "[{}] Loaded plugin: {:?}",
                            backend_name,
                            path.file_name().unwrap_or_default()
                        ),
                        Err(e) => crate::gslog_error!(
                            LogTarget::Wasm,
                            "[{}] Failed to load {:?}: {e}",
                            backend_name,
                            path.file_name().unwrap_or_default()
                        ),
                    }
                }
            }
        }

        let runtime = Self { manager };
        RUNTIME.with(|cell| {
            *cell.borrow_mut() = Some(runtime);
        });
        Ok(())
    }

    /// Run `f` with exclusive access to the `PluginManager`, if initialized.
    ///
    /// The engine is single-threaded, so `RefCell` gives safe interior
    /// mutability without the aliasing UB of a `static mut`.
    pub fn with_manager<R>(f: impl FnOnce(Option<&mut PluginManager>) -> R) -> R {
        RUNTIME.with(|cell| f(cell.borrow_mut().as_mut().map(|r| &mut r.manager)))
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
