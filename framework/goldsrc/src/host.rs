use crate::{HostConfig, paths::PathResolver};
use goldsrc_api::consts::BackendType;
use goldsrc_wasm_host::PluginManager;
use goldsrc_wasm_host::error::HostError;

pub struct HostRuntime {
    manager: PluginManager,
    engine: std::sync::Arc<dyn goldsrc_api::Engine>,
    pub plugins_config: crate::plugins_config::PluginsConfig,
    pub paused_plugins: std::collections::HashMap<String, bool>,
    pub current_map: String,
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
        goldsrc_wasm_host::set_show_menu_callback(|_player_idx, _keys_mask, _timeout, _text| {});

        let mut manager = PluginManager::new(engine.clone())
            .map_err(|e| HostError::Manager(format!("[GoldSrc.rs {backend_name}] {e}")))?;
        manager.set_plugin_dirs(crate::paths::PathResolver::plugin_dirs(backend));

        let sys_config = HostConfig::load_or_create(backend);

        // Initialise unified logger
        let logs_dir = crate::paths::PathResolver::existing_log_dir(backend);
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

        // Try to enable hot-reload watcher if enabled in config
        let plugins_dirs = crate::paths::PathResolver::plugin_dirs(backend);
        let mut watcher_enabled = false;
        for dir in &plugins_dirs {
            if let Err(e) = manager.enable_hot_reload(dir) {
                log::warn!(target: "wasm", "Failed to enable hot-reload on {:?}: {e}", dir);
            } else {
                watcher_enabled = true;
            }
        }
        if watcher_enabled {
            log::info!(target: "wasm", "Hot-reload watcher is ACTIVE for plugins");
        }

        let config_dir = crate::paths::PathResolver::existing_config_dir(backend);
        if let Err(e) = manager.enable_config_watcher(&config_dir) {
            log::warn!(target: "wasm", "Failed to enable config watcher on {:?}: {e}", config_dir);
        }

        // Load or create plugins.toml configuration template
        let plugins_config_path = config_dir.join("plugins.toml");
        let plugins_config =
            crate::plugins_config::PluginsConfig::load_or_create(&plugins_config_path);
        log::info!(
            target: "wasm",
            "Plugins orchestration config loaded from: \"{}\"",
            crate::paths::PathResolver::normalize(&plugins_config_path)
        );

        // Recursive helper to discover all .wasm plugins in directory tree
        fn discover_wasm_plugins(
            dir: &std::path::Path,
            base_dir: &std::path::Path,
            out: &mut Vec<(String, std::path::PathBuf)>,
        ) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        discover_wasm_plugins(&path, base_dir, out);
                    } else if path.extension().is_some_and(|ext| ext == "wasm")
                        && let Ok(rel_path) = path.strip_prefix(base_dir)
                    {
                        let rel_str = rel_path
                            .with_extension("")
                            .to_string_lossy()
                            .replace('\\', "/");
                        out.push((rel_str, path));
                    }
                }
            }
        }

        let mut discovered_plugins = Vec::new();
        let plugin_dir = crate::paths::PathResolver::existing_plugin_dir(backend);
        discover_wasm_plugins(&plugin_dir, &plugin_dir, &mut discovered_plugins);

        // Sort discovered plugins by priority from plugins.toml (higher priority loads first)
        discovered_plugins.sort_by_key(|(name, _)| {
            let priority = plugins_config
                .plugins
                .iter()
                .find(|p| p.name == *name)
                .map(|p| p.priority)
                .unwrap_or(100);
            std::cmp::Reverse(priority)
        });

        // Load plugins based on plugins.toml activation status
        for (rel_name, path) in discovered_plugins {
            if !plugins_config.is_plugin_enabled(&rel_name) {
                log::info!(target: "wasm", "Skipping disabled plugin: '{}'", rel_name);
                continue;
            }

            match manager.load_plugin(&path) {
                Ok(index) => {
                    log::info!(
                        target: "wasm",
                        "Loaded plugin [{index}] '{}' from \"{}\"",
                        rel_name,
                        PathResolver::normalize(&path)
                    );
                }
                Err(e) => {
                    log::error!(
                        target: "wasm",
                        "Failed to load plugin '{}' (\"{}\"): {e}",
                        rel_name,
                        PathResolver::normalize(&path)
                    );
                }
            }
        }

        let paused_plugins = std::collections::HashMap::new();

        let runtime = Self {
            manager,
            engine,
            plugins_config,
            paused_plugins,
            current_map: String::new(),
        };
        let _ = RUNTIME.set(Mutex::new(runtime));

        // Evaluate initial rules (e.g. initial pause/cvar states)
        Self::evaluate_rules("", 0);

        Ok(())
    }

    /// Returns a clone of the Engine reference if initialized.
    pub fn engine() -> Option<std::sync::Arc<dyn goldsrc_api::Engine>> {
        RUNTIME
            .get()
            .and_then(|lock| lock.lock().ok().map(|g| g.engine.clone()))
    }

    /// Returns the currently active map name.
    pub fn current_map() -> String {
        RUNTIME
            .get()
            .and_then(|lock| lock.lock().ok().map(|g| g.current_map.clone()))
            .unwrap_or_default()
    }

    /// Sets the active map name.
    pub fn set_current_map(map_name: &str) {
        if let Some(lock) = RUNTIME.get() {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            guard.current_map = map_name.to_string();
        }
    }

    /// Clears temporary rule pause overrides on map change.
    pub fn on_map_change() {
        if let Some(lock) = RUNTIME.get() {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            guard.paused_plugins.clear();
            guard.current_map.clear();
        }
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

    /// Triggers reactive rule engine re-evaluation for current game state.
    /// Drops the `HostRuntime` mutex during rule action execution to avoid deadlocks.
    pub fn evaluate_rules(map_name: &str, player_count: usize) {
        let Some(lock) = RUNTIME.get() else {
            return;
        };

        // 1. Snapshot engine, rules, config, and paused states under short lock
        let (engine, mut plugins_config, mut paused_plugins) = {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            if guard.plugins_config.rules.is_empty() {
                return;
            }
            if !map_name.is_empty() {
                guard.current_map = map_name.to_string();
            }
            (
                guard.engine.clone(),
                guard.plugins_config.clone(),
                guard.paused_plugins.clone(),
            )
        };

        // 2. Evaluate rules and execute actions OUTSIDE of the HostRuntime lock
        {
            let registry = crate::rules::create_default_server_rule_registry();
            let rules: Vec<goldsrc_api::rules::Rule> = plugins_config
                .rules
                .iter()
                .map(|r| goldsrc_api::rules::Rule::new(&r.name, r.when.clone(), r.action.clone()))
                .collect();
            let rule_engine = goldsrc_api::rules::RuleEngine::new(registry, rules);

            let mut ctx = crate::rules::ServerRuleContext {
                map_name,
                player_count,
                engine: engine.as_ref(),
                plugins_config: &mut plugins_config,
                paused_plugins: &mut paused_plugins,
                execution_log: Vec::new(),
            };

            let results = rule_engine.evaluate_and_execute(&mut ctx);
            for (rule_name, res) in results {
                match res {
                    Ok(_) => log::info!(target: "rules", "Executed reactive rule '{}'", rule_name),
                    Err(errors) => log::warn!(
                        target: "rules",
                        "Failed to execute rule '{}': {:?}",
                        rule_name,
                        errors
                    ),
                }
            }
        }

        // 3. Re-acquire lock to commit updated paused states and recalculate dependencies
        {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            guard.paused_plugins = paused_plugins.clone();
            guard.plugins_config = plugins_config;

            for (plugin_name, is_paused) in &paused_plugins {
                let _ = guard.manager.pause_plugin(plugin_name, *is_paused);
            }
            guard.manager.recalculate_dependency_states();
        }
    }

    /// Tick plugins frame event.
    pub fn on_server_frame() {
        Self::with_manager(|m| {
            if let Some(manager) = m {
                manager.on_server_frame();
            }
        });
        crate::logging::flush();
    }
}
