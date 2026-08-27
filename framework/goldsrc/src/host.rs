use crate::{HostConfig, paths::PathResolver};
use goldsrc_api::consts::BackendType;
use goldsrc_wasm_host::PluginManager;
use goldsrc_wasm_host::error::HostError;

pub struct HostRuntime {
    manager: PluginManager,
    engine: std::sync::Arc<dyn goldsrc_api::Engine>,
    pub plugins_config: crate::plugins_config::PluginsConfig,
    pub paused_plugins: std::collections::HashMap<String, bool>,
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

        if sys_config.watcher.enabled
            && let Err(e) = manager.enable_hot_reload(&plugin_dir)
        {
            log::warn!(target: "wasm", "Failed to enable hot reload on {:?}: {e}", plugin_dir);
        }
        if sys_config.watcher.watch_configs
            && let Err(e) = manager.enable_config_watcher(&config_dir)
        {
            log::warn!(target: "wasm", "Failed to enable config watcher on {:?}: {e}", config_dir);
        }

        // Load plugins.toml configuration if present
        let plugins_config_path = config_dir.join("plugins.toml");
        let mut plugins_config = if plugins_config_path.is_file() {
            match std::fs::read_to_string(&plugins_config_path) {
                Ok(content) => match crate::plugins_config::PluginsConfig::parse(&content) {
                    Ok(cfg) => {
                        log::info!(target: "wasm", "Loaded plugin orchestration config from {:?}", plugins_config_path);
                        cfg
                    }
                    Err(e) => {
                        log::warn!(target: "wasm", "Failed to parse {:?}: {e}, using default discovery", plugins_config_path);
                        crate::plugins_config::PluginsConfig::default()
                    }
                },
                Err(e) => {
                    log::warn!(target: "wasm", "Failed to read {:?}: {e}", plugins_config_path);
                    crate::plugins_config::PluginsConfig::default()
                }
            }
        } else {
            crate::plugins_config::PluginsConfig::default()
        };

        // Recursive helper to discover all .wasm plugins in directory tree
        fn discover_wasm_plugins(
            dir: &std::path::Path,
            base_dir: &std::path::Path,
            out: &mut Vec<(String, std::path::PathBuf)>,
        ) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        discover_wasm_plugins(&path, base_dir, out);
                    } else if path.is_file()
                        && path.extension().and_then(|s| s.to_str()) == Some("wasm")
                    {
                        let rel_path = path.strip_prefix(base_dir).unwrap_or(&path);
                        let rel_name = rel_path
                            .with_extension("")
                            .to_string_lossy()
                            .replace('\\', "/");
                        out.push((rel_name, path));
                    }
                }
            }
        }

        let mut discovered = Vec::new();
        discover_wasm_plugins(&plugin_dir, &plugin_dir, &mut discovered);

        // Sort by priority from plugins_config (higher priority loads earlier)
        discovered.sort_by_key(|(name, _)| {
            let priority = plugins_config
                .plugins
                .iter()
                .find(|p| p.name == *name)
                .map(|p| p.priority)
                .unwrap_or(100);
            std::cmp::Reverse(priority)
        });

        for (name, path) in discovered {
            if !plugins_config.is_plugin_enabled(&name) {
                log::info!(target: "wasm", "Plugin \"{}\" disabled by configuration, skipping", name);
                continue;
            }

            match manager.load_plugin(&path) {
                Ok(_) => log::info!(
                    target: "wasm",
                    "Loaded plugin: \"{}\"",
                    name
                ),
                Err(e) => log::error!(
                    target: "wasm",
                    "Failed to load \"{}\": {e}",
                    name
                ),
            }
        }

        let mut paused_plugins = std::collections::HashMap::new();

        // Evaluate startup reactive rules from plugins.toml
        if !plugins_config.rules.is_empty() {
            let registry = crate::rules::create_default_server_rule_registry();
            let rules: Vec<goldsrc_api::rules::Rule> = plugins_config
                .rules
                .iter()
                .map(|r| goldsrc_api::rules::Rule::new(&r.name, r.when.clone(), r.action.clone()))
                .collect();
            let rule_engine = goldsrc_api::rules::RuleEngine::new(registry, rules);

            let mut ctx = crate::rules::ServerRuleContext {
                map_name: "",
                player_count: 0,
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

        // Sync paused states to manager
        for (plugin_name, is_paused) in &paused_plugins {
            let _ = manager.pause_plugin(plugin_name, *is_paused);
        }

        manager.recalculate_dependency_states();
        for info in manager.get_plugins_info() {
            if let goldsrc_wasm_host::PluginStatus::Blocked { reason } = &info.status {
                log::warn!(
                    target: "wasm",
                    "Plugin '{}' BLOCKED: {}",
                    info.name,
                    reason
                );
            }
        }

        let runtime = Self {
            manager,
            engine,
            plugins_config,
            paused_plugins,
        };
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

    /// Triggers reactive rule engine re-evaluation for current game state.
    pub fn evaluate_rules(map_name: &str, player_count: usize) {
        if let Some(lock) = RUNTIME.get() {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            if guard.plugins_config.rules.is_empty() {
                return;
            }

            let HostRuntime {
                ref mut manager,
                ref engine,
                ref mut plugins_config,
                ref mut paused_plugins,
            } = *guard;

            {
                let registry = crate::rules::create_default_server_rule_registry();
                let rules: Vec<goldsrc_api::rules::Rule> = plugins_config
                    .rules
                    .iter()
                    .map(|r| {
                        goldsrc_api::rules::Rule::new(&r.name, r.when.clone(), r.action.clone())
                    })
                    .collect();
                let rule_engine = goldsrc_api::rules::RuleEngine::new(registry, rules);

                let mut ctx = crate::rules::ServerRuleContext {
                    map_name,
                    player_count,
                    engine: engine.as_ref(),
                    plugins_config,
                    paused_plugins,
                    execution_log: Vec::new(),
                };

                let results = rule_engine.evaluate_and_execute(&mut ctx);
                for (rule_name, res) in results {
                    match res {
                        Ok(_) => {
                            log::info!(target: "rules", "Executed reactive rule '{}'", rule_name)
                        }
                        Err(errors) => log::warn!(
                            target: "rules",
                            "Failed to execute rule '{}': {:?}",
                            rule_name,
                            errors
                        ),
                    }
                }
            }

            // Sync paused states to manager
            for (plugin_name, is_paused) in &*paused_plugins {
                let _ = manager.pause_plugin(plugin_name, *is_paused);
            }
            manager.recalculate_dependency_states();
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
