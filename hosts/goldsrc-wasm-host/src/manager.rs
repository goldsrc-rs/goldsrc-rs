use crate::bindings::{GoldsrcPlugin, goldsrc::engine::api};
use crate::error::{CommandError, LoadError};
use crate::plugin::{LoadedPlugin, PluginMetadata};
use goldsrc_api::EngineOps;
use std::fs;
use std::path::{Path, PathBuf};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine};

use notify::Watcher;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

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

    fn host_entity_angles(&mut self, index: i32) -> api::Vector3 {
        let [x, y, z] = self.engine.entity_angles(index);
        api::Vector3 { x, y, z }
    }
    fn host_entity_set_angles(&mut self, index: i32, angles: api::Vector3) {
        self.engine
            .entity_set_angles(index, [angles.x, angles.y, angles.z]);
    }
    fn host_create_named_entity(&mut self, classname: String) -> Option<i32> {
        self.engine.create_named_entity(&classname)
    }
    fn host_remove_entity(&mut self, index: i32) {
        self.engine.remove_entity(index);
    }
    fn host_drop_to_floor(&mut self, index: i32) -> i32 {
        self.engine.drop_to_floor(index)
    }

    fn host_cvar_get_float(&mut self, name: String) -> f32 {
        self.engine.cvar_get_float(&name)
    }
    fn host_cvar_set_float(&mut self, name: String, val: f32) {
        self.engine.cvar_set_float(&name, val);
    }
    fn host_cvar_get_string(&mut self, name: String) -> Option<String> {
        self.engine.cvar_get_string(&name)
    }
    fn host_cvar_set_string(&mut self, name: String, val: String) {
        self.engine.cvar_set_string(&name, &val);
    }

    fn host_precache_model(&mut self, path: String) -> i32 {
        self.engine.precache_model(&path)
    }
    fn host_precache_sound(&mut self, path: String) -> i32 {
        self.engine.precache_sound(&path)
    }
    fn host_precache_generic(&mut self, path: String) -> i32 {
        self.engine.precache_generic(&path)
    }
    fn host_emit_sound(
        &mut self,
        entity: i32,
        channel: i32,
        sample: String,
        volume: f32,
        attenuation: f32,
        sound_flags: i32,
        pitch: i32,
    ) {
        self.engine.emit_sound(
            entity,
            channel,
            &sample,
            volume,
            attenuation,
            sound_flags,
            pitch,
        );
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
    last_reload: HashMap<PathBuf, Instant>,
    /// command name -> plugin indices that registered it.
    command_registry: HashMap<String, Vec<usize>>,
}

/// Minimum gap between two hot-reloads of the same file. Compilers write in
/// several passes, so a rebuild would otherwise reload a half-written file.
const RELOAD_DEBOUNCE: Duration = Duration::from_millis(150);

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
            last_reload: HashMap::new(),
            command_registry: HashMap::new(),
        })
    }

    /// Loads a WASM plugin from `path`. Accepts either a pre-compiled
    /// component (magic `\0asm`) or a plain core module, which is embedded
    /// with component metadata first. Calls the plugin's `on_load`.
    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<String, LoadError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| LoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;

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
            .map_err(|e| LoadError::Embed(e.to_string()))?;

            let mut encoder = wit_component::ComponentEncoder::default()
                .module(&wasm_bytes)
                .map_err(|e| LoadError::Encode(format!("{e:#?}")))?
                .validate(true);

            encoder
                .encode()
                .map_err(|e| LoadError::Encode(format!("{e:#?}")))?
        };

        let component = Component::new(&self.engine, &component_bytes)
            .map_err(|e| LoadError::Compile(e.to_string()))?;

        let mut linker = Linker::new(&self.engine);
        api::add_to_linker::<HostState, wasmtime::component::HasSelf<HostState>>(
            &mut linker,
            |state: &mut HostState| state,
        )
        .map_err(|e| LoadError::Link(e.to_string()))?;

        let state = HostState {
            engine: self.engine_ops.clone(),
        };
        let mut store = wasmtime::Store::new(&self.engine, state);
        let bindings = GoldsrcPlugin::instantiate(&mut store, &component, &linker)
            .map_err(|e| LoadError::Instantiate(e.to_string()))?;

        let metadata = bindings
            .call_get_metadata(&mut store)
            .ok()
            .and_then(|meta_str| toml::from_str::<PluginMetadata>(&meta_str).ok());

        let mut plugin = LoadedPlugin {
            name: path.file_stem().unwrap().to_string_lossy().to_string(),
            path: path.to_path_buf(),
            is_paused: false,
            is_poisoned: false,
            metadata,
            store,
            bindings,
            component,
        };

        plugin
            .call_on_load()
            .map_err(|e| LoadError::LoadPanic(e.to_string()))?;
        crate::host_log(&format!("Loaded component plugin: {}", plugin.name));
        let idx = self.plugins.len();
        if let Some(meta) = &plugin.metadata {
            for cmd in &meta.commands {
                self.command_registry
                    .entry(cmd.clone())
                    .or_default()
                    .push(idx);
            }
        }
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

    /// Calls `on_unload` (if exported) and removes the plugin at `idx`.
    fn unload_plugin_at(&mut self, idx: usize) -> LoadedPlugin {
        // Deregister this plugin's commands.
        let meta = self.plugins[idx].metadata.clone();
        for cmd in meta.iter().flat_map(|m| &m.commands) {
            if let Some(owners) = self.command_registry.get_mut(cmd) {
                owners.retain(|i| *i != idx);
                if owners.is_empty() {
                    self.command_registry.remove(cmd);
                }
            }
        }
        let mut plugin = self.plugins.remove(idx);
        if plugin.has_export("on-unload") {
            let _ = plugin.call_on_unload();
        }
        // Shift indices of plugins after idx in the registry.
        for owners in self.command_registry.values_mut() {
            for i in owners.iter_mut() {
                if *i > idx {
                    *i -= 1;
                }
            }
        }
        plugin
    }

    /// Unloads all loaded plugins and returns a summary message.
    pub fn unload_all_plugins(&mut self) -> String {
        let count = self.plugins.len();
        while !self.plugins.is_empty() {
            self.unload_plugin_at(self.plugins.len() - 1);
        }
        format!("Unloaded {} plugins.", count)
    }

    /// Sets or clears the pause flag on a plugin by name.
    pub fn pause_plugin(&mut self, name: &str, pause: bool) -> Result<String, CommandError> {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.name == name) {
            plugin.is_paused = pause;
            Ok(format!("Plugin '{}' pause state set to {}", name, pause))
        } else {
            Err(CommandError::NotFound(name.to_string()))
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
                has_on_load: p.has_export("on-load"),
                has_on_unload: p.has_export("on-unload"),
                has_on_frame: p.has_export("on-frame"),
            })
            .collect()
    }

    /// Unloads and reloads every loaded plugin from its recorded path.
    /// Returns a summary counting failures.
    pub fn reload_all_plugins(&mut self) -> String {
        let paths: Vec<PathBuf> = self.plugins.iter().map(|p| p.path.clone()).collect();
        let count = paths.len();
        while !self.plugins.is_empty() {
            self.unload_plugin_at(self.plugins.len() - 1);
        }
        let mut failed = 0;
        for path in &paths {
            if self.load_plugin(path).is_err() {
                failed += 1;
            }
        }
        format!("Reloaded {} plugins ({} failed).", count - failed, failed)
    }

    /// Reloads a single plugin by name or index.
    pub fn reload_plugin_by_query(&mut self, query: &str) -> Result<String, CommandError> {
        let idx = self
            .find_plugin(query)
            .ok_or_else(|| CommandError::NotFound(query.to_string()))?;
        let path = self.plugins[idx].path.clone();
        let name = self.plugins[idx].name.clone();
        self.unload_plugin_at(idx);
        self.load_plugin(&path)
            .map(|_| format!("Reloaded '{}'", name))
            .map_err(|source| CommandError::Load { name, source })
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
            self.unload_plugin_at(idx);
            match self.load_plugin(&path) {
                Ok(_) => crate::host_log(&format!("Hot-reloaded plugin '{}'", name)),
                Err(e) => crate::host_log(&format!("Hot-reload of '{}' failed: {}", name, e)),
            }
        }
    }

    /// Debounced wrapper around [`reload_plugin_path`]: ignores events for a
    /// file reloaded less than [`RELOAD_DEBOUNCE`] ago.
    fn reload_plugin_path_debounced(&mut self, path: &Path) {
        let now = Instant::now();
        if let Some(last) = self.last_reload.get(path) {
            if now.duration_since(*last) < RELOAD_DEBOUNCE {
                return;
            }
        }
        self.last_reload.insert(path.to_path_buf(), now);
        self.reload_plugin_path(path);
    }

    /// Spawns a `notify` watcher on `dir` that forwards changed files with
    /// extension `ext` to the event channel, drained in [`on_server_frame`].
    ///
    /// [`on_server_frame`]: PluginManager::on_server_frame
    fn spawn_watcher<P: AsRef<Path>>(
        &mut self,
        dir: P,
        ext: &'static str,
    ) -> Result<(), CommandError> {
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
        .map_err(|e| CommandError::Watcher(e.to_string()))?;
        watcher
            .watch(&dir, notify::RecursiveMode::NonRecursive)
            .map_err(|e| CommandError::Watcher(e.to_string()))?;

        self.watchers.push(watcher);
        self.watcher_count += 1;
        Ok(())
    }

    /// Watches `dir` for changed `.wasm` files and reloads matching plugins
    /// on the next server frame.
    pub fn enable_hot_reload<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), CommandError> {
        self.spawn_watcher(dir, "wasm")
    }

    /// Watches `dir` for changed `.toml` files. Change events are drained in
    /// [`on_server_frame`], where `.wasm` events trigger reloads.
    ///
    /// [`on_server_frame`]: PluginManager::on_server_frame
    pub fn enable_config_watcher<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), CommandError> {
        self.spawn_watcher(dir, "toml")
    }

    /// Dispatches a server command to the plugins that registered it.
    /// Stops at the first plugin that reports handling it (consume).
    pub fn dispatch_command(&mut self, cmd: &str, args: &str) -> bool {
        let Some(owners) = self.command_registry.get(cmd).cloned() else {
            return false;
        };
        for idx in owners {
            if let Some(plugin) = self.plugins.get_mut(idx) {
                if plugin.call_on_command(cmd, args).unwrap_or(false) {
                    return true;
                }
            }
        }
        false
    }

    /// Drains watcher events (reloading changed `.wasm` plugins, debounced)
    /// then ticks every plugin's `on_frame`. `.toml` events are forwarded to
    /// plugins as `on_event("config_changed", <path>)`. Call once per frame.
    pub fn on_server_frame(&mut self) {
        while let Ok(path) = self.event_rx.try_recv() {
            match path.extension().and_then(|s| s.to_str()) {
                Some("wasm") => self.reload_plugin_path_debounced(&path),
                Some("toml") => {
                    let data = path.to_string_lossy().as_bytes().to_vec();
                    self.call_on_event("config_changed", &data);
                }
                _ => {}
            }
        }
        self.call_on_frame();
    }

    /// Loads a plugin by filesystem path (string form of [`load_plugin`]).
    ///
    /// [`load_plugin`]: PluginManager::load_plugin
    pub fn load_plugin_by_name(&mut self, query: &str) -> Result<String, LoadError> {
        let path = PathBuf::from(query);
        self.load_plugin(path)
    }

    /// Unloads a single plugin by name or index.
    pub fn unload_plugin_by_query(&mut self, query: &str) -> Result<String, CommandError> {
        let idx = self
            .find_plugin(query)
            .ok_or_else(|| CommandError::NotFound(query.to_string()))?;
        let plugin = self.unload_plugin_at(idx);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopEngineOps;

    impl goldsrc_api::EnginePrecache for NoopEngineOps {
        fn precache_model(&self, _path: &str) -> i32 {
            0
        }
        fn precache_sound(&self, _path: &str) -> i32 {
            0
        }
        fn precache_generic(&self, _path: &str) -> i32 {
            0
        }
    }

    impl goldsrc_api::EngineMessages for NoopEngineOps {
        fn message_begin(
            &self,
            _msg_dest: i32,
            _msg_type: i32,
            _origin: Option<[f32; 3]>,
            _edict_index: Option<i32>,
        ) {
        }
        fn message_end(&self) {}
        fn write_byte(&self, _val: i32) {}
        fn write_char(&self, _val: i32) {}
        fn write_short(&self, _val: i32) {}
        fn write_long(&self, _val: i32) {}
        fn write_angle(&self, _val: f32) {}
        fn write_coord(&self, _val: f32) {}
        fn write_string(&self, _val: &str) {}
        fn write_entity(&self, _val: i32) {}
    }

    impl goldsrc_api::EngineEntities for NoopEngineOps {
        fn entity_is_valid(&self, _index: i32) -> bool {
            false
        }
        fn entity_classname(&self, _index: i32) -> Option<String> {
            None
        }
        fn entity_health(&self, _index: i32) -> f32 {
            0.0
        }
        fn entity_set_health(&self, _index: i32, _health: f32) {}
        fn entity_origin(&self, _index: i32) -> [f32; 3] {
            [0.0; 3]
        }
        fn entity_set_origin(&self, _index: i32, _pos: [f32; 3]) {}
        fn entity_velocity(&self, _index: i32) -> [f32; 3] {
            [0.0; 3]
        }
        fn entity_set_velocity(&self, _index: i32, _vel: [f32; 3]) {}
        fn entity_angles(&self, _index: i32) -> [f32; 3] {
            [0.0; 3]
        }
        fn entity_set_angles(&self, _index: i32, _angles: [f32; 3]) {}
        fn player_name(&self, _index: i32) -> Option<String> {
            None
        }
        fn player_armorvalue(&self, _index: i32) -> f32 {
            0.0
        }
        fn player_set_armorvalue(&self, _index: i32, _armor: f32) {}
        fn create_named_entity(&self, _classname: &str) -> Option<i32> {
            None
        }
        fn remove_entity(&self, _index: i32) {}
        fn drop_to_floor(&self, _index: i32) -> i32 {
            0
        }
    }

    impl goldsrc_api::EngineCvars for NoopEngineOps {
        fn cvar_get_float(&self, _name: &str) -> f32 {
            0.0
        }
        fn cvar_set_float(&self, _name: &str, _val: f32) {}
        fn cvar_get_string(&self, _name: &str) -> Option<String> {
            None
        }
        fn cvar_set_string(&self, _name: &str, _val: &str) {}
    }

    impl goldsrc_api::EnginePhysics for NoopEngineOps {
        fn point_contents(&self, _point: [f32; 3]) -> i32 {
            0
        }
        fn trace_line(
            &self,
            _start: [f32; 3],
            _end: [f32; 3],
            _flags: i32,
            _ignore_ent: i32,
        ) -> goldsrc_api::TraceResult {
            goldsrc_api::TraceResult::default()
        }
        fn trace_hull(
            &self,
            _start: [f32; 3],
            _end: [f32; 3],
            _flags: i32,
            _hull_number: i32,
            _ignore_ent: i32,
        ) -> goldsrc_api::TraceResult {
            goldsrc_api::TraceResult::default()
        }
    }

    impl goldsrc_api::EngineSound for NoopEngineOps {
        fn emit_sound(
            &self,
            _entity: i32,
            _channel: i32,
            _sample: &str,
            _volume: f32,
            _attenuation: f32,
            _flags: i32,
            _pitch: i32,
        ) {
        }
        fn emit_ambient_sound(
            &self,
            _entity: i32,
            _pos: [f32; 3],
            _sample: &str,
            _volume: f32,
            _attenuation: f32,
            _flags: i32,
            _pitch: i32,
        ) {
        }
    }

    /// Loads the built demo plugin and checks the command registry + consume semantics.
    #[test]
    fn command_registry_registers_and_consumes() {
        // Skipped when the demo plugin was not built (e.g. `cargo test -p goldsrc-wasm-host`).
        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../target/wasm32-unknown-unknown/debug/test_suite.wasm"
        );
        if !std::path::Path::new(wasm_path).exists() {
            eprintln!("test_suite.wasm not built; skipping command registry test");
            return;
        }

        let mut manager = PluginManager::new(Arc::new(NoopEngineOps)).unwrap();
        manager.load_plugin(wasm_path).unwrap();

        // The #[command(name = "testcmd")] marker must be discoverable in metadata.
        let meta = &manager.plugins[0].metadata;
        assert!(meta.is_some());
        assert!(
            meta.as_ref()
                .unwrap()
                .commands
                .contains(&"testcmd".to_string())
        );

        // Dispatch finds the plugin via the registry and consumes the command.
        assert!(manager.dispatch_command("testcmd", "hello"));
        // Unknown commands are not dispatched at all.
        assert!(!manager.dispatch_command("nonexistent", ""));
    }
}
