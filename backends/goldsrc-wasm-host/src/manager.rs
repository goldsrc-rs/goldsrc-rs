use crate::bindings::{goldsrc::engine::api, GoldsrcPlugin};
use crate::plugin::{LoadedPlugin, PluginMetadata};
use goldsrc_api::EngineOps;
use std::fs;
use std::path::{Path, PathBuf};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine};

use notify::Watcher;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

/// Wasmtime store state exposed to WASM plugins via host functions.
pub struct HostState {
    /// Engine bridge for real game-state access.
    pub engine: Arc<dyn EngineOps>,
}

/// Read-only snapshot of a loaded plugin, used for CLI listing/info.
pub struct PluginInfo {
    /// Plugin name.
    pub name: String,
    /// Path the plugin was loaded from.
    pub path: PathBuf,
    /// Index in the plugin list.
    pub index: usize,
    /// Whether the plugin is paused.
    pub is_paused: bool,
    /// Parsed metadata, if any.
    pub metadata: Option<PluginMetadata>,
    /// Whether the plugin exports `on_load`.
    pub has_on_load: bool,
    /// Whether the plugin exports `on_unload`.
    pub has_on_unload: bool,
    /// Whether the plugin exports `on_frame`.
    pub has_on_frame: bool,
}

impl api::Host for HostState {
    fn host_log(&mut self, msg: String) {
        crate::host_log(&msg);
    }

    fn host_entity_is_valid(&mut self, index: i32) -> bool {
        self.engine.entity_is_valid(index)
    }
    fn host_entity_classname(&mut self, index: i32) -> Option<String> {
        self.engine.entity_classname(index)
    }
    fn host_entity_health(&mut self, index: i32) -> f32 {
        self.engine.entity_health(index)
    }
    fn host_entity_set_health(&mut self, index: i32, health: f32) {
        self.engine.entity_set_health(index, health);
    }
    fn host_entity_origin(&mut self, index: i32) -> api::Vector3 {
        let [x, y, z] = self.engine.entity_origin(index);
        api::Vector3 { x, y, z }
    }
    fn host_entity_set_origin(&mut self, index: i32, pos: api::Vector3) {
        self.engine.entity_set_origin(index, [pos.x, pos.y, pos.z]);
    }
    fn host_entity_velocity(&mut self, index: i32) -> api::Vector3 {
        let [x, y, z] = self.engine.entity_velocity(index);
        api::Vector3 { x, y, z }
    }
    fn host_entity_set_velocity(&mut self, index: i32, vel: api::Vector3) {
        self.engine
            .entity_set_velocity(index, [vel.x, vel.y, vel.z]);
    }
    fn host_player_name(&mut self, index: i32) -> Option<String> {
        self.engine.player_name(index)
    }
    fn host_player_armorvalue(&mut self, index: i32) -> f32 {
        self.engine.player_armorvalue(index)
    }
    fn host_player_set_armorvalue(&mut self, index: i32, armor: f32) {
        self.engine.player_set_armorvalue(index, armor);
    }

    fn host_register_capability(&mut self, name: String, description: String) -> bool {
        let mut caps = goldsrc_api::caps::CAPS
            .write()
            .unwrap_or_else(|e| e.into_inner());
        if let std::collections::hash_map::Entry::Vacant(e) = caps.registered.entry(name) {
            e.insert(description);
            true
        } else {
            false
        }
    }

    fn host_has_capability(&mut self, player_index: i32, name: String) -> bool {
        let caps = goldsrc_api::caps::CAPS
            .read()
            .unwrap_or_else(|e| e.into_inner());
        caps.player_capabilities
            .get(&player_index)
            .is_some_and(|player_caps| player_caps.contains(&name))
    }

    fn host_grant_capability(&mut self, player_index: i32, name: String) -> bool {
        let mut caps = goldsrc_api::caps::CAPS
            .write()
            .unwrap_or_else(|e| e.into_inner());
        // Check if capability exists in the registry
        if !caps.registered.contains_key(&name) {
            return false;
        }
        caps.player_capabilities
            .entry(player_index)
            .or_default()
            .insert(name)
    }

    fn host_revoke_capability(&mut self, player_index: i32, name: String) -> bool {
        let mut caps = goldsrc_api::caps::CAPS
            .write()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(player_caps) = caps.player_capabilities.get_mut(&player_index) {
            player_caps.remove(&name)
        } else {
            false
        }
    }
}

/// Manages the lifecycle of loaded WASM plugins: loading, unloading,
/// reloading, pausing, frame dispatch and hot-reload via directory watchers.
///
/// Not `Send`/`Sync` (holds wasmtime stores) — keep it on the server thread.
pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
    engine: Engine,
    engine_ops: Arc<dyn EngineOps>,
    event_rx: Receiver<PathBuf>,
    event_tx: Sender<PathBuf>,
    watchers: Vec<notify::RecommendedWatcher>,
    watcher_count: usize,
}

impl PluginManager {
    /// Creates an empty plugin manager backed by a fresh wasmtime engine
    /// with the Component Model enabled.
    pub fn new(engine_ops: Arc<dyn EngineOps>) -> wasmtime::Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        // config.target("pulley32").unwrap();
        let engine = Engine::new(&config)?;
        let (event_tx, event_rx) = mpsc::channel::<PathBuf>();
        Ok(Self {
            plugins: Vec::new(),
            engine,
            engine_ops,
            event_rx,
            event_tx,
            watchers: Vec::new(),
            watcher_count: 0,
        })
    }

    /// Loads a WASM plugin from `path`. Accepts either a pre-compiled
    /// component (magic `\0asm`) or a plain core module, which is embedded
    /// with component metadata first. Calls the plugin's `on_load`.
    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<String, String> {
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;

        let is_comp = bytes.len() >= 8 && &bytes[0..8] == b"\0asm\x0d\0\x01\0";
        let component_bytes = if is_comp {
            bytes
        } else {
            let mut resolve = wit_parser::Resolve::default();
            let pkg = resolve
                .push_str(
                    "goldsrc.wit",
                    include_str!("../../../core/goldsrc-api/wit/goldsrc.wit"),
                )
                .unwrap();
            let world_id = resolve
                .select_world(&[pkg], Some("goldsrc-plugin"))
                .unwrap();

            let mut wasm_bytes = bytes.to_vec();
            wit_component::embed_component_metadata(
                &mut wasm_bytes,
                &resolve,
                world_id,
                wit_component::StringEncoding::UTF8,
            )
            .map_err(|e| format!("Failed to embed component metadata: {}", e))?;

            let mut encoder = wit_component::ComponentEncoder::default()
                .module(&wasm_bytes)
                .map_err(|e| format!("ComponentEncoder module error: {:#?}", e))?
                .validate(true);

            encoder
                .encode()
                .map_err(|e| format!("ComponentEncoder encode error: {:#?}", e))?
        };

        let component =
            Component::new(&self.engine, &component_bytes).map_err(|e| e.to_string())?;

        let mut linker = Linker::new(&self.engine);
        api::add_to_linker::<HostState, wasmtime::component::HasSelf<HostState>>(
            &mut linker,
            |state: &mut HostState| state,
        )
        .map_err(|e| e.to_string())?;

        let state = HostState {
            engine: self.engine_ops.clone(),
        };
        let mut store = wasmtime::Store::new(&self.engine, state);
        let bindings = GoldsrcPlugin::instantiate(&mut store, &component, &linker)
            .map_err(|e| e.to_string())?;

        let metadata = bindings
            .call_get_metadata(&mut store)
            .ok()
            .and_then(|meta_str| toml::from_str::<PluginMetadata>(&meta_str).ok());

        let mut plugin = LoadedPlugin {
            name: path
                .as_ref()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            path: path.as_ref().to_path_buf(),
            is_paused: false,
            is_poisoned: false,
            metadata,
            store,
            bindings,
        };

        plugin.call_on_load().map_err(|e| e.to_string())?;
        crate::host_log(&format!("Loaded component plugin: {}", plugin.name));
        self.plugins.push(plugin);
        Ok("Loaded successfully".to_string())
    }

    /// Resolves a plugin query (either numeric index or name) to a plugin index.
    pub fn find_plugin(&self, query: &str) -> Option<usize> {
        if let Ok(idx) = query.parse::<usize>() {
            return (idx < self.plugins.len()).then_some(idx);
        }
        self.plugins.iter().position(|p| p.name == query)
    }

    /// Unloads all loaded plugins and returns a summary message.
    pub fn unload_all_plugins(&mut self) -> String {
        let count = self.plugins.len();
        self.plugins.clear();
        format!("Unloaded {} plugins.", count)
    }

    /// Sets or clears the pause flag on a plugin by name.
    pub fn pause_plugin(&mut self, name: &str, pause: bool) -> Result<String, String> {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.name == name) {
            plugin.is_paused = pause;
            Ok(format!("Plugin '{}' pause state set to {}", name, pause))
        } else {
            Err(format!("Plugin '{}' not found", name))
        }
    }

    /// Sets or clears the pause flag on every loaded plugin.
    pub fn pause_all_plugins(&mut self, pause: bool) -> String {
        for p in &mut self.plugins {
            p.is_paused = pause;
        }
        format!("All plugins pause state set to {}", pause)
    }

    /// Returns a snapshot of metadata for all loaded plugins.
    pub fn get_plugins_info(&self) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .enumerate()
            .map(|(index, p)| PluginInfo {
                name: p.name.clone(),
                path: p.path.clone(),
                index,
                is_paused: p.is_paused,
                metadata: p.metadata.clone(),
                has_on_load: true,
                has_on_unload: false,
                has_on_frame: true,
            })
            .collect()
    }

    /// Unloads and reloads every loaded plugin from its recorded path.
    /// Returns a summary counting failures.
    pub fn reload_all_plugins(&mut self) -> String {
        let paths: Vec<PathBuf> = self.plugins.iter().map(|p| p.path.clone()).collect();
        let count = paths.len();
        self.plugins.clear();
        let mut failed = 0;
        for path in &paths {
            if self.load_plugin(path).is_err() {
                failed += 1;
            }
        }
        format!("Reloaded {} plugins ({} failed).", count - failed, failed)
    }

    /// Reloads a single plugin by name or index.
    pub fn reload_plugin_by_query(&mut self, query: &str) -> Result<String, String> {
        let idx = self
            .find_plugin(query)
            .ok_or_else(|| format!("Plugin '{}' not found", query))?;
        let path = self.plugins[idx].path.clone();
        let name = self.plugins[idx].name.clone();
        self.plugins.remove(idx);
        match self.load_plugin(&path) {
            Ok(_) => Ok(format!("Reloaded '{}'", name)),
            Err(e) => Err(format!("Reload of '{}' failed: {}", name, e)),
        }
    }

    /// Reloads the plugin whose recorded path matches `path`. Used by the
    /// hot-reload watcher; failures are logged, not returned.
    fn reload_plugin_path(&mut self, path: &Path) {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if let Some(idx) = self
            .plugins
            .iter()
            .position(|p| p.path == path || p.path.canonicalize().is_ok_and(|c| c == canonical))
        {
            let name = self.plugins[idx].name.clone();
            let path = self.plugins[idx].path.clone();
            self.plugins.remove(idx);
            match self.load_plugin(&path) {
                Ok(_) => crate::host_log(&format!("Hot-reloaded plugin '{}'", name)),
                Err(e) => crate::host_log(&format!("Hot-reload of '{}' failed: {}", name, e)),
            }
        }
    }

    /// Spawns a `notify` watcher on `dir` that forwards changed files with
    /// extension `ext` to the event channel, drained in [`on_server_frame`].
    ///
    /// [`on_server_frame`]: PluginManager::on_server_frame
    fn spawn_watcher<P: AsRef<Path>>(&mut self, dir: P, ext: &'static str) -> Result<(), String> {
        let dir = dir.as_ref().to_path_buf();
        let tx = self.event_tx.clone();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                for path in event.paths {
                    if path
                        .extension()
                        .and_then(|s| s.to_str())
                        .is_some_and(|e| e == ext)
                    {
                        let _ = tx.send(path);
                    }
                }
            }
        })
        .map_err(|e| e.to_string())?;
        watcher
            .watch(&dir, notify::RecursiveMode::NonRecursive)
            .map_err(|e| e.to_string())?;

        self.watchers.push(watcher);
        self.watcher_count += 1;
        Ok(())
    }

    /// Watches `dir` for changed `.wasm` files and reloads matching plugins
    /// on the next server frame.
    pub fn enable_hot_reload<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), String> {
        self.spawn_watcher(dir, "wasm")
    }

    /// Watches `dir` for changed `.toml` files. Change events are drained in
    /// [`on_server_frame`], where `.wasm` events trigger reloads.
    ///
    /// [`on_server_frame`]: PluginManager::on_server_frame
    pub fn enable_config_watcher<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), String> {
        self.spawn_watcher(dir, "toml")
    }

    /// Dispatches a server command to all plugins.
    pub fn dispatch_command(&mut self, cmd: &str, args: &str) -> bool {
        self.call_on_command(cmd, args)
    }

    /// Drains watcher events (reloading changed `.wasm` plugins) then ticks
    /// every plugin's `on_frame`. Call once per server frame.
    pub fn on_server_frame(&mut self) {
        while let Ok(path) = self.event_rx.try_recv() {
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                self.reload_plugin_path(&path);
            }
        }
        self.call_on_frame();
    }

    /// Loads a plugin by filesystem path (string form of [`load_plugin`]).
    ///
    /// [`load_plugin`]: PluginManager::load_plugin
    pub fn load_plugin_by_name(&mut self, query: &str) -> Result<String, String> {
        let path = PathBuf::from(query);
        self.load_plugin(path)
    }

    /// Unloads a single plugin by name or index.
    pub fn unload_plugin_by_query(&mut self, query: &str) -> Result<String, String> {
        let idx = self
            .find_plugin(query)
            .ok_or_else(|| format!("Plugin '{}' not found", query))?;
        let plugin = self.plugins.remove(idx);
        Ok(format!("Unloaded '{}'", plugin.name))
    }

    /// Returns `(loaded_plugins, active_watchers)` for status displays.
    pub fn get_status_info(&self) -> (usize, usize) {
        (self.plugins.len(), self.watcher_count)
    }

    /// Calls `on_frame` on every (non-paused) plugin.
    pub fn call_on_frame(&mut self) {
        for plugin in &mut self.plugins {
            let _ = plugin.call_on_frame();
        }
    }

    /// Calls `on_event` on every (non-paused) plugin.
    pub fn call_on_event(&mut self, name: &str, data: &[u8]) {
        for plugin in &mut self.plugins {
            let _ = plugin.call_on_event(name, data);
        }
    }

    /// Calls `on_command` on every (non-paused) plugin. Returns `false`
    /// (reserved for future "command handled" signalling).
    pub fn call_on_command(&mut self, cmd: &str, args: &str) -> bool {
        for plugin in &mut self.plugins {
            let _ = plugin.call_on_command(cmd, args);
        }
        false
    }
}
