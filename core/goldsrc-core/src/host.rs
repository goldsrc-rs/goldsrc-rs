use crate::{HostConfig, paths::PathResolver};
use goldsrc_api::StorageProvider;
use goldsrc_api::consts::{BackendType, log_targets};
use goldsrc_host_wasm::PluginManager;
use goldsrc_host_wasm::error::HostError;

pub struct HostRuntime {
    pub backend: BackendType,
    manager: PluginManager,
    engine: std::sync::Arc<dyn goldsrc_api::Engine>,
    pub storage: std::sync::Arc<crate::storage::SqliteStorageEngine>,
    pub plugins_config: crate::plugins_config::PluginsConfig,
    pub paused_plugins: std::collections::HashMap<String, bool>,
    pub current_map: String,
    pub rule_orchestrator: crate::rules::RuleOrchestrator,
    pub watcher_service: crate::watcher::WatcherService,
}

use std::sync::{Mutex, OnceLock};
use std::time::Instant;

static RUNTIME: OnceLock<Mutex<HostRuntime>> = OnceLock::new();
static ENGINE_INSTANCE: OnceLock<std::sync::Arc<dyn goldsrc_api::Engine>> = OnceLock::new();
static PLAYER_LANG_OVERRIDES: std::sync::LazyLock<
    std::sync::RwLock<std::collections::HashMap<i32, String>>,
> = std::sync::LazyLock::new(|| std::sync::RwLock::new(std::collections::HashMap::new()));

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
        goldsrc_host_wasm::set_print_callback(print_cb);
        goldsrc_host_wasm::set_show_menu_callback(|_player_idx, _keys_mask, _timeout, _text| {});

        goldsrc_host_wasm::set_storage_callbacks(
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

        let _ = ENGINE_INSTANCE.set(engine.clone());

        goldsrc_host_wasm::set_translate_callback(|caller, dict, lang, key| {
            crate::i18n::I18nService::translate_with_caller(caller, dict, lang, key, &[], &[])
        });

        goldsrc_host_wasm::set_format_placeholders_callback(|player_idx, text| {
            let player = if player_idx > 0 {
                goldsrc_api::Player::new(player_idx)
            } else {
                goldsrc_api::Player::new(0)
            };
            crate::placeholders::format_placeholders(text, player)
        });

        goldsrc_api::client::player::set_player_resolver_hook(|index| {
            if let Some(engine) = HostRuntime::engine() {
                engine.player_handle(index)
            } else {
                None
            }
        });
        goldsrc_api::client::player::set_player_name_hook(|index| {
            if let Some(engine) = HostRuntime::engine() {
                engine.player_name(index)
            } else {
                None
            }
        });
        goldsrc_api::client::player::set_player_team_hook(|index| {
            if let Some(engine) = HostRuntime::engine() {
                engine.player_team(index)
            } else {
                0
            }
        });
        goldsrc_api::client::player::set_player_lang_hook(|index| {
            if let Some(override_lang) = HostRuntime::get_player_language_override(index) {
                Some(override_lang)
            } else if let Some(engine) = HostRuntime::engine() {
                engine.player_lang(index)
            } else {
                None
            }
        });
        goldsrc_api::client::player::set_native_print_hook(|player_index, target, message| {
            if let Some(engine) = HostRuntime::engine() {
                crate::net::NetworkMessageDispatcher::dispatch_player_print(
                    engine.as_ref(),
                    player_index,
                    target,
                    message,
                );
            }
        });
        goldsrc_api::client::player::set_open_menu_hook(|player_idx, menu| {
            if let Some(engine) = HostRuntime::engine() {
                let current_time = HostRuntime::current_time();
                if let Ok(mut mgr) = crate::menu::menu_manager().lock() {
                    mgr.open_menu(player_idx, menu.clone(), engine.as_ref(), current_time);
                }
            }
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
            target: log_targets::CORE,
            "GoldSrc.rs v{} initialized (Backend: {})",
            env!("CARGO_PKG_VERSION"),
            backend_name
        );

        let main_cfg_path = PathResolver::main_config_path(backend);
        log::info!(
            target: log_targets::CORE,
            "Config loaded from: \"{}\"",
            PathResolver::normalize(&main_cfg_path)
        );

        // 1. Initialize SQLite WAL Storage Engine in data/db/goldsrc.db
        let db_path = crate::paths::PathResolver::db_path(backend);
        let storage = match crate::storage::SqliteStorageEngine::open(&db_path) {
            Ok(s) => {
                log::info!(
                    target: log_targets::STORAGE,
                    "SQLite WAL Storage Engine initialized at \"{}\"",
                    PathResolver::normalize(&db_path)
                );
                s
            }
            Err(e) => {
                log::error!(
                    target: log_targets::STORAGE,
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
        let common_lang_file = lang_dir.join("common.toml");
        if !common_lang_file.exists() {
            let default_common = include_str!("../../../resources/lang/common.toml");
            let _ = std::fs::write(&common_lang_file, default_common);
        }
        let sample_lang_file = lang_dir.join("test_i18n.toml");
        if !sample_lang_file.exists() {
            let default_template = include_str!("../../../resources/lang/test_i18n.toml");
            let _ = std::fs::write(&sample_lang_file, default_template);
        }
        let lang_count = crate::i18n::I18nService::load_dir(&lang_dir);
        log::info!(
            target: log_targets::I18N,
            "Loaded {lang_count} localization entries from \"{}\"",
            PathResolver::normalize(&lang_dir)
        );

        // 3. Initialize watcher service and register default watchers
        let mut watcher_service = crate::watcher::WatcherService::new();

        let existing_plugin_dir = crate::paths::PathResolver::existing_plugin_dir(backend);
        if let Err(e) = watcher_service.register(crate::watcher::WatcherSpec::directory(
            "core:plugins",
            &existing_plugin_dir,
            crate::watcher::WatcherFilter::Extension("wasm"),
            true,
        )) {
            log::warn!(
                target: log_targets::CORE,
                "Failed to register plugin watcher on \"{}\": {e}",
                PathResolver::normalize(&existing_plugin_dir)
            );
        } else {
            log::info!(
                target: log_targets::CORE,
                "Watcher registered: 'core:plugins' on \"{}\"",
                PathResolver::normalize(&existing_plugin_dir)
            );
        }

        let config_dir = crate::paths::PathResolver::existing_config_dir(backend);
        let plugins_config_path = config_dir.join("plugins.toml");
        if let Err(e) = watcher_service.register(crate::watcher::WatcherSpec::file(
            "core:configs",
            &plugins_config_path,
        )) {
            log::warn!(
                target: log_targets::CORE,
                "Failed to register config watcher on \"{}\": {e}",
                PathResolver::normalize(&plugins_config_path)
            );
        } else {
            log::info!(
                target: log_targets::CORE,
                "Watcher registered: 'core:configs' on \"{}\"",
                PathResolver::normalize(&plugins_config_path)
            );
        }

        if let Err(e) = watcher_service.register(crate::watcher::WatcherSpec::directory(
            "i18n:dicts",
            &lang_dir,
            crate::watcher::WatcherFilter::Extension("toml"),
            true,
        )) {
            log::warn!(
                target: log_targets::CORE,
                "Failed to register i18n watcher on \"{}\": {e}",
                PathResolver::normalize(&lang_dir)
            );
        } else {
            log::info!(
                target: log_targets::CORE,
                "Watcher registered: 'i18n:dicts' on \"{}\"",
                PathResolver::normalize(&lang_dir)
            );
        }

        // Load or create plugins.toml configuration template
        let plugins_config_path = config_dir.join("plugins.toml");
        let plugins_config =
            crate::plugins_config::PluginsConfig::load_or_create(&plugins_config_path);
        log::info!(
            target: log_targets::WASM,
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

        // Resolve plugin load order deterministically using PhasedDag:
        // Tier (Core -> Service -> Gameplay -> Addon -> Analytics) -> Topological Dependencies (`requires`) -> Discovery Order
        let mut dag = goldsrc_api::dag::PhasedDag::<
            goldsrc_api::dag::PluginTier,
            String,
            std::path::PathBuf,
        >::new();
        for (rel_name, path) in &discovered_plugins {
            let base_name = rel_name
                .rsplit_once('/')
                .map(|(_, b)| b)
                .unwrap_or(rel_name);
            let entry = plugins_config
                .plugins
                .iter()
                .find(|p| p.name == *rel_name || p.name == base_name);
            let tier = entry.map(|p| p.tier).unwrap_or_default();
            let mut builder = dag.add(rel_name.clone(), path.clone()).phase(tier);
            if let Some(e) = entry {
                for req in &e.requires {
                    let target_rel = discovered_plugins
                        .iter()
                        .find(|(d_name, _)| {
                            d_name == req
                                || d_name.rsplit_once('/').map(|(_, b)| b).unwrap_or(d_name) == req
                        })
                        .map(|(d_name, _)| d_name.clone())
                        .unwrap_or_else(|| req.clone());
                    builder = builder.after(target_rel);
                }
            }
            builder.register();
        }

        let sorted_plugins: Vec<(String, std::path::PathBuf)> = match dag.resolve() {
            Ok(resolved) => resolved.into_iter().map(|n| (n.id, n.data)).collect(),
            Err(e) => {
                log::error!(
                    target: log_targets::WASM,
                    "Plugin topological resolution encountered conflict: {e}. Falling back to default discovery order."
                );
                discovered_plugins
            }
        };

        // Load plugins based on plugins.toml activation status
        for (rel_name, path) in sorted_plugins {
            let is_enabled = plugins_config.is_plugin_enabled(&rel_name);
            match manager.load_plugin(&path) {
                Ok(plugin_name) => {
                    log::info!(
                        target: log_targets::WASM,
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
                        target: log_targets::WASM,
                        "Failed to load plugin '{}' (\"{}\"): {e}",
                        rel_name,
                        PathResolver::normalize(&path)
                    );
                }
            }
        }

        let paused_plugins = std::collections::HashMap::new();
        let mut rule_orchestrator = crate::rules::RuleOrchestrator::new();
        let initial_rules = plugins_config.rules.iter().map(|r| r.to_rule()).collect();
        rule_orchestrator.set_rules(initial_rules);

        let runtime = Self {
            backend,
            manager,
            engine,
            storage,
            plugins_config,
            paused_plugins,
            current_map: String::new(),
            rule_orchestrator,
            watcher_service,
        };
        let _ = RUNTIME.set(Mutex::new(runtime));

        // Evaluate initial rules (e.g. initial pause/cvar states) across all scopes
        Self::evaluate_rules("", 0);

        Ok(())
    }

    /// Returns a clone of the Engine reference if initialized.
    pub fn engine() -> Option<std::sync::Arc<dyn goldsrc_api::Engine>> {
        ENGINE_INSTANCE.get().cloned()
    }

    /// Returns monotonic server host uptime in seconds (f32).
    pub fn current_time() -> f32 {
        static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        START_TIME
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_secs_f32()
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

    /// Sets a deliberate administrator manual pause state override for a plugin.
    pub fn set_manual_pause_override(plugin_name: &str, is_paused: bool) {
        if let Some(lock) = RUNTIME.get() {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            guard
                .rule_orchestrator
                .set_manual_override(plugin_name, is_paused);
        }
    }

    /// Removes a deliberate administrator manual pause state override for a plugin.
    pub fn remove_manual_pause_override(plugin_name: &str) -> Option<bool> {
        if let Some(lock) = RUNTIME.get() {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            guard.rule_orchestrator.remove_manual_override(plugin_name)
        } else {
            None
        }
    }

    /// Clears all deliberate administrator manual pause state overrides.
    pub fn clear_manual_pause_overrides() {
        if let Some(lock) = RUNTIME.get() {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            guard.rule_orchestrator.clear_manual_overrides();
        }
    }

    /// Clears temporary rule pause overrides and flushes storage on map change.
    pub fn on_map_change() {
        if let Some(lock) = RUNTIME.get() {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            let _ = guard.storage.flush();
            guard.paused_plugins.clear();
            guard.current_map.clear();
            guard.rule_orchestrator.on_map_change();
        }
        crate::logging::flush();
    }

    /// Run `f` with exclusive access to the `PluginManager`, if initialized.
    /// Protects against re-entrant mutex deadlock if called recursively on the same thread.
    pub fn with_manager<R>(f: impl FnOnce(Option<&mut PluginManager>) -> R) -> R {
        thread_local! {
            static IN_MANAGER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
        }

        if IN_MANAGER.get() {
            log::warn!(
                target: log_targets::CORE,
                "Re-entrant call to HostRuntime::with_manager detected and suppressed to prevent deadlock"
            );
            return f(None);
        }

        if let Some(lock) = RUNTIME.get() {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            IN_MANAGER.set(true);
            struct ResetGuard;
            impl Drop for ResetGuard {
                fn drop(&mut self) {
                    IN_MANAGER.set(false);
                }
            }
            let _reset = ResetGuard;
            f(Some(&mut guard.manager))
        } else {
            f(None)
        }
    }

    /// Run `f` with exclusive access to the host runtime's [`WatcherService`], if initialized.
    pub fn with_watcher_service<R>(
        f: impl FnOnce(Option<&mut crate::watcher::WatcherService>) -> R,
    ) -> R {
        if let Some(lock) = RUNTIME.get() {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            f(Some(&mut guard.watcher_service))
        } else {
            f(None)
        }
    }

    /// Returns the session language override for player if one was explicitly set.
    pub fn get_player_language_override(index: i32) -> Option<String> {
        PLAYER_LANG_OVERRIDES.read().ok()?.get(&index).cloned()
    }

    /// Sets the session language override for player.
    pub fn set_player_language_override(index: i32, lang: &str) {
        if let Ok(mut lock) = PLAYER_LANG_OVERRIDES.write() {
            lock.insert(index, lang.to_lowercase());
        }
    }

    /// Triggers reactive rule engine re-evaluation for current game state across all scopes.
    pub fn evaluate_rules(map_name: &str, player_count: usize) {
        Self::evaluate_rules_scoped(goldsrc_api::rules::RuleScope::All, map_name, player_count);
    }

    /// Triggers reactive rule engine re-evaluation for current game state under the specified [`RuleScope`].
    /// Drops the `HostRuntime` mutex during rule action execution to avoid deadlocks.
    pub fn evaluate_rules_scoped(
        scope: goldsrc_api::rules::RuleScope,
        map_name: &str,
        player_count: usize,
    ) {
        let Some(lock) = RUNTIME.get() else {
            return;
        };

        let backend = lock
            .lock()
            .map(|g| g.backend)
            .unwrap_or(BackendType::Metamod);

        // 1. Reload plugins.toml dynamically to pick up any changes made by the server administrator
        let config_dir = crate::paths::PathResolver::existing_config_dir(backend);
        let plugins_config_path = config_dir.join("plugins.toml");
        let fresh_config =
            crate::plugins_config::PluginsConfig::load_or_create(&plugins_config_path);

        // 2. Evaluate rules directly on the active RuleOrchestrator under lock
        let results = {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            guard.plugins_config = fresh_config;

            let rules: Vec<goldsrc_api::rules::Rule> = guard
                .plugins_config
                .rules
                .iter()
                .map(|r| r.to_rule())
                .collect();
            guard.rule_orchestrator.set_rules(rules);

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

            let effective_map = guard.current_map.clone();
            let manual_overrides = guard.rule_orchestrator.manual_overrides().clone();
            let engine = guard.engine.clone();

            let rt = &mut *guard;
            let results = {
                let mut ctx = crate::rules::ServerRuleContext {
                    map_name: &effective_map,
                    player_count,
                    engine: engine.as_ref(),
                    plugins_config: &mut rt.plugins_config,
                    paused_plugins: &mut rt.paused_plugins,
                    manual_overrides: &manual_overrides,
                    execution_log: Vec::new(),
                };
                rt.rule_orchestrator.evaluate_scope(&scope, &mut ctx)
            };

            // Synchronize plugin states with WASM manager
            crate::plugins::PluginOrchestrator::sync_plugin_states(
                &mut rt.manager,
                &rt.plugins_config,
                &rt.paused_plugins,
                &manual_overrides,
            );

            results
        };

        for (rule_name, res) in results {
            match res {
                Ok(_) => {
                    log::info!(target: log_targets::RULES, "Executed reactive rule '{}'", rule_name)
                }
                Err(errors) => log::warn!(
                    target: log_targets::RULES,
                    "Failed to execute rule '{}': {:?}",
                    rule_name,
                    errors
                ),
            }
        }
    }

    /// Tick plugins frame event and drain debounced watcher events.
    pub fn on_server_frame() {
        let events = if let Some(lock) = RUNTIME.get() {
            let mut guard = lock.lock().unwrap_or_else(|e| e.into_inner());
            guard.watcher_service.drain_events()
        } else {
            Vec::new()
        };

        for event in events {
            match event.watcher_id.as_str() {
                "core:plugins" => {
                    log::info!(
                        target: log_targets::WASM,
                        "Detected change in plugin file \"{}\", reloading...",
                        crate::paths::PathResolver::normalize(&event.path)
                    );
                    let path = &event.path;
                    if !path.exists() {
                        Self::with_manager(|m| {
                            if let Some(manager) = m {
                                manager.unload_plugin_by_path(path);
                            }
                        });
                        continue;
                    }

                    // 1. Snapshot engine pointers under brief lock
                    let engines = Self::with_manager(|m| {
                        m.map(|manager| (manager.wasm_engine(), manager.engine_ops()))
                    });

                    let Some((wasm_engine, engine_ops)) = engines else {
                        continue;
                    };

                    // 2. Heavy Cranelift compilation & component instantiation OUTSIDE lock (drop(guard))!
                    match goldsrc_host_wasm::manager::loader::instantiate_plugin(
                        &wasm_engine,
                        &engine_ops,
                        path,
                    ) {
                        Ok(new_plugin) => {
                            // 3. Short lock re-acquisition to commit reloaded plugin and swap instances
                            Self::with_manager(|m| {
                                if let Some(manager) = m {
                                    let _ = manager.commit_reloaded_plugin(new_plugin, path);
                                }
                            });
                        }
                        Err(e) => {
                            log::warn!(
                                target: log_targets::WASM,
                                "Hot-reload instantiation of \"{}\" failed (previous version kept active): {e}",
                                crate::paths::PathResolver::normalize(path)
                            );
                        }
                    }
                }
                "core:configs" => {
                    let path = &event.path;
                    let file_name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default();
                    if file_name.eq_ignore_ascii_case("plugins.toml") {
                        log::info!(
                            target: log_targets::WASM,
                            "Hot-reloaded plugins orchestration config from \"{}\"",
                            crate::paths::PathResolver::normalize(path)
                        );
                        let data = path.to_string_lossy().as_bytes().to_vec();
                        Self::with_manager(|m| {
                            if let Some(manager) = m {
                                manager.call_on_event("config_changed", &data);
                            }
                        });
                        Self::evaluate_rules("", 0);
                    }
                }
                "i18n:dicts" => {
                    let path = &event.path;
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                        && let Ok(count) = crate::i18n::I18nService::load_file(stem, path)
                    {
                        log::info!(
                            target: log_targets::I18N,
                            "Hot-reloaded {count} keys from \"{}\"",
                            crate::paths::PathResolver::normalize(path)
                        );
                        let data = path.to_string_lossy().as_bytes().to_vec();
                        Self::with_manager(|m| {
                            if let Some(manager) = m {
                                manager.call_on_event("config_changed", &data);
                            }
                        });
                        if let Some(engine) = Self::engine() {
                            let now = Self::current_time();
                            if let Ok(mut mgr) = crate::menu::menu_manager().lock() {
                                mgr.refresh_all_menus(engine.as_ref(), now);
                            }
                        }
                    }
                }
                _ => {
                    log::debug!(
                        target: log_targets::WATCHER,
                        "Unhandled watcher event '{}' for path \"{}\"",
                        event.watcher_id,
                        crate::paths::PathResolver::normalize(&event.path)
                    );
                }
            }
        }

        Self::with_manager(|m| {
            if let Some(manager) = m {
                manager.call_on_frame();
            }
        });

        let now = Self::current_time();
        if let Some(engine) = Self::engine()
            && let Ok(mut mgr) = crate::menu::menu_manager().lock()
        {
            mgr.tick_frame(now, engine.as_ref());
        }

        // Throttle disk flushing to at most once every second to prevent per-frame I/O stalls
        static LAST_LOG_FLUSH: OnceLock<Mutex<Instant>> = OnceLock::new();
        let tracker = LAST_LOG_FLUSH.get_or_init(|| Mutex::new(Instant::now()));
        if let Ok(mut last) = tracker.try_lock()
            && last.elapsed() >= std::time::Duration::from_millis(1000)
        {
            *last = Instant::now();
            crate::logging::flush();
        }
    }
}
