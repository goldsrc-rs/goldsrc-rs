use crate::{HostConfig, paths::PathResolver};
use goldsrc_api::StorageProvider;
use goldsrc_api::consts::BackendType;
use goldsrc_wasm_host::PluginManager;
use goldsrc_wasm_host::error::HostError;

pub struct HostRuntime {
    manager: PluginManager,
    engine: std::sync::Arc<dyn goldsrc_api::Engine>,
    pub storage: std::sync::Arc<crate::storage::SqliteStorageEngine>,
    pub plugins_config: crate::plugins_config::PluginsConfig,
    pub paused_plugins: std::collections::HashMap<String, bool>,
    pub current_map: String,
}

use std::sync::{Mutex, OnceLock};

static RUNTIME: OnceLock<Mutex<HostRuntime>> = OnceLock::new();

impl HostRuntime {
    /// Initialize the host runtime, logger, configuration, storage, i18n and hot reload watchers.
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

        goldsrc_wasm_host::set_storage_callbacks(
            |bucket, key| HostRuntime::storage().and_then(|s| s.get(bucket, key).ok().flatten()),
            |bucket, key, val| {
                HostRuntime::storage()
                    .map(|s| s.set(bucket, key, val).is_ok())
                    .unwrap_or(false)
            },
            |bucket, key| {
                HostRuntime::storage()
                    .map(|s| s.delete(bucket, key).unwrap_or(false))
                    .unwrap_or(false)
            },
            |bucket, key, delta| {
                HostRuntime::storage()
                    .and_then(|s| s.fetch_add(bucket, key, delta).ok())
                    .unwrap_or(0)
            },
        );

        goldsrc_wasm_host::set_translate_callback(|caller, dict, lang, key| {
            crate::i18n::I18nEngine::translate_with_caller(caller, dict, lang, key, &[], &[])
        });

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

        // 1. Initialize SQLite WAL Storage Engine in data/db/goldsrc.db
        let db_path = crate::paths::PathResolver::db_path(backend);
        let storage = match crate::storage::SqliteStorageEngine::open(&db_path) {
            Ok(s) => {
                log::info!(
                    target: "storage",
                    "SQLite WAL Storage Engine initialized at \"{}\"",
                    PathResolver::normalize(&db_path)
                );
                s
            }
            Err(e) => {
                log::error!(
                    target: "storage",
                    "Failed to initialize SQLite Storage Engine at \"{}\": {e}",
                    PathResolver::normalize(&db_path)
                );
                return Err(HostError::Manager(format!("Storage init failed: {e}")));
            }
        };

        // 2. Initialize i18n dictionaries from data/lang/*.toml
        let lang_dir = crate::paths::PathResolver::lang_dir(backend);
        if !lang_dir.exists() {
            let _ = std::fs::create_dir_all(&lang_dir);
        }
        let sample_lang_file = lang_dir.join("test_i18n.toml");
        if !sample_lang_file.exists() {
            let default_template = include_str!("../../../resources/lang/test_i18n.toml");
            let _ = std::fs::write(&sample_lang_file, default_template);
        }
        let lang_count = crate::i18n::I18nEngine::load_dir(&lang_dir);
        log::info!(
            target: "i18n",
            "Loaded {lang_count} localization entries from \"{}\"",
            PathResolver::normalize(&lang_dir)
        );

        // Try to enable hot-reload watcher if enabled in config
        let existing_plugin_dir = crate::paths::PathResolver::existing_plugin_dir(backend);
        let mut watcher_enabled = false;
        if existing_plugin_dir.exists() {
            if let Err(e) = manager.enable_hot_reload(&existing_plugin_dir) {
                log::warn!(target: "wasm", "Failed to enable hot-reload on {:?}: {e}", existing_plugin_dir);
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
                        && let Ok(rel) = path.strip_prefix(base_dir)
                    {
                        let rel_str = rel.with_extension("").to_string_lossy().replace('\\', "/");
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
            let is_enabled = plugins_config.is_plugin_enabled(&rel_name);
            match manager.load_plugin(&path) {
                Ok(plugin_name) => {
                    log::info!(
                        target: "wasm",
                        "Loaded plugin '{}' from \"{}\"",
                        rel_name,
                        PathResolver::normalize(&path)
                    );
                    if !is_enabled {
                        let _ = manager.pause_plugin(&plugin_name, true);
                    }
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
            storage,
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

    /// Returns a reference to the shared SQLite Storage Engine.
    pub fn storage() -> Option<std::sync::Arc<crate::storage::SqliteStorageEngine>> {
        RUNTIME
            .get()
            .and_then(|lock| lock.lock().ok().map(|g| g.storage.clone()))
    }

    /// Clears temporary rule pause overrides and flushes storage on map change.
    pub fn on_map_change() {
        if let Some(lock) = RUNTIME.get() {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            let _ = guard.storage.flush();
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

        // 1. Reload plugins.toml dynamically to pick up any changes made by the server administrator
        let config_dir = crate::paths::PathResolver::existing_config_dir(
            goldsrc_api::consts::BackendType::Metamod,
        );
        let plugins_config_path = config_dir.join("plugins.toml");
        let fresh_config =
            crate::plugins_config::PluginsConfig::load_or_create(&plugins_config_path);

        // 2. Snapshot engine, rules, config, and paused states under short lock
        let (engine, mut plugins_config, mut paused_plugins, effective_map) = {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            guard.plugins_config = fresh_config;

            let resolved_map = if !map_name.is_empty() {
                map_name.to_string()
            } else if !guard.current_map.is_empty() {
                guard.current_map.clone()
            } else {
                guard.engine.cvar_get_string("mapname").unwrap_or_default()
            };

            if !resolved_map.is_empty() {
                guard.current_map = resolved_map.clone();
            }

            (
                guard.engine.clone(),
                guard.plugins_config.clone(),
                guard.paused_plugins.clone(),
                resolved_map,
            )
        };

        log::info!(
            target: "rules",
            "Evaluating {} rules for map: '{}', players: {}",
            plugins_config.rules.len(),
            effective_map,
            player_count
        );

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
                map_name: &effective_map,
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

            // 3.1. Apply group and individual enabled/disabled base state from plugins_config
            let plugins_info = guard.manager.get_plugins_info();
            for info in plugins_info {
                if let Some(is_paused) = paused_plugins.get(&info.name) {
                    // Reactive rule has explicit override for this plugin
                    let reason = if *is_paused {
                        Some("reactive rule".to_string())
                    } else {
                        None
                    };
                    let _ = guard
                        .manager
                        .pause_plugin_with_reason(&info.name, *is_paused, reason);
                } else {
                    // Fall back to declarative enabled/disabled status in plugins.toml
                    let disabled_reason = plugins_config.plugin_disabled_reason(&info.name);
                    let is_paused = disabled_reason.is_some();
                    let _ = guard.manager.pause_plugin_with_reason(
                        &info.name,
                        is_paused,
                        disabled_reason,
                    );
                }
            }

            guard.plugins_config = plugins_config;
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
