use crate::bindings::{goldsrc::engine::api, GoldsrcPlugin};
use crate::plugin::{LoadedPlugin, PluginMetadata};
use std::fs;
use std::path::{Path, PathBuf};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine};

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct CapabilityRegistry {
    pub registered: HashMap<String, String>, // name -> description
    pub player_capabilities: HashMap<i32, HashSet<String>>, // player_index -> capabilities
}

#[derive(Clone)]
pub struct HostState {
    pub caps: Arc<RwLock<CapabilityRegistry>>,
}

pub struct PluginInfo {
    pub name: String,
    pub path: PathBuf,
    pub index: usize,
    pub is_paused: bool,
    pub metadata: Option<PluginMetadata>,
    pub has_on_load: bool,
    pub has_on_unload: bool,
    pub has_on_frame: bool,
}

impl api::Host for HostState {
    fn host_log(&mut self, msg: String) {
        crate::host_log(&msg);
    }

    fn host_entity_is_valid(&mut self, _index: i32) -> bool {
        true
    }
    fn host_entity_classname(&mut self, _index: i32) -> Option<String> {
        None
    }
    fn host_entity_health(&mut self, _index: i32) -> f32 {
        100.0
    }
    fn host_entity_set_health(&mut self, _index: i32, _health: f32) {}
    fn host_entity_origin(&mut self, _index: i32) -> api::Vector3 {
        api::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
    fn host_entity_set_origin(&mut self, _index: i32, _pos: api::Vector3) {}
    fn host_entity_velocity(&mut self, _index: i32) -> api::Vector3 {
        api::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
    fn host_entity_set_velocity(&mut self, _index: i32, _vel: api::Vector3) {}
    fn host_player_name(&mut self, _index: i32) -> Option<String> {
        Some("Player".to_string())
    }
    fn host_player_armorvalue(&mut self, _index: i32) -> f32 {
        0.0
    }
    fn host_player_set_armorvalue(&mut self, index: i32, armor: f32) {
        crate::host_log(&format!("(mock) Player {} armor set to {}", index, armor));
    }

    fn host_register_capability(&mut self, name: String, description: String) -> bool {
        let mut caps = self.caps.write().unwrap();
        if let std::collections::hash_map::Entry::Vacant(e) = caps.registered.entry(name) {
            e.insert(description);
            true
        } else {
            false
        }
    }

    fn host_has_capability(&mut self, player_index: i32, name: String) -> bool {
        let caps = self.caps.read().unwrap();
        caps.player_capabilities
            .get(&player_index)
            .is_some_and(|player_caps| player_caps.contains(&name))
    }

    fn host_grant_capability(&mut self, player_index: i32, name: String) -> bool {
        let mut caps = self.caps.write().unwrap();
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
        let mut caps = self.caps.write().unwrap();
        if let Some(player_caps) = caps.player_capabilities.get_mut(&player_index) {
            player_caps.remove(&name)
        } else {
            false
        }
    }
}

pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
    engine: Engine,
    caps: Arc<RwLock<CapabilityRegistry>>,
}

impl PluginManager {
    pub fn new() -> wasmtime::Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        // config.target("pulley32").unwrap();
        let engine = Engine::new(&config)?;
        Ok(Self {
            plugins: Vec::new(),
            caps: Arc::new(RwLock::new(CapabilityRegistry::default())),
            engine,
        })
    }

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
            caps: self.caps.clone(),
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

    pub fn unload_plugin<P: AsRef<Path>>(&mut self, path: P) {
        let p_ref = path.as_ref();
        let canonical = p_ref.canonicalize().unwrap_or_else(|_| p_ref.to_path_buf());
        if let Some(idx) = self
            .plugins
            .iter()
            .position(|p| p.path == p_ref || p.path == canonical)
        {
            let plugin = self.plugins.remove(idx);
            crate::host_log(&format!("Unloaded plugin {}", plugin.name));
        }
    }

    pub fn unload_all_plugins(&mut self) -> String {
        let count = self.plugins.len();
        self.plugins.clear();
        format!("Unloaded {} plugins.", count)
    }

    pub fn pause_plugin(&mut self, name: &str, pause: bool) -> Result<String, String> {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.name == name) {
            plugin.is_paused = pause;
            Ok(format!("Plugin '{}' pause state set to {}", name, pause))
        } else {
            Err(format!("Plugin '{}' not found", name))
        }
    }

    pub fn pause_all_plugins(&mut self, pause: bool) -> String {
        for p in &mut self.plugins {
            p.is_paused = pause;
        }
        format!("All plugins pause state set to {}", pause)
    }

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

    pub fn reload_all_plugins(&mut self) -> String {
        let count = self.plugins.len();
        self.plugins.clear();
        format!("Reloaded {} plugins. (Placeholder)", count)
    }

    pub fn reload_plugin_by_query(&mut self, query: &str) -> Result<String, String> {
        Ok(format!("Reloaded plugin '{}'. (Placeholder)", query))
    }

    pub fn enable_hot_reload<P: AsRef<Path>>(&mut self, _path: P) -> Result<(), String> {
        Ok(())
    }

    pub fn enable_config_watcher<P: AsRef<Path>>(&mut self, _path: P) -> Result<(), String> {
        Ok(())
    }

    pub fn dispatch_command(&mut self, cmd: &str, args: &str) -> bool {
        self.call_on_command(cmd, args)
    }

    pub fn on_server_frame(&mut self) {
        self.call_on_frame();
    }

    pub fn load_plugin_by_name(&mut self, query: &str) -> Result<String, String> {
        let path = PathBuf::from(query);
        self.load_plugin(path)
    }

    pub fn unload_plugin_by_query(&mut self, query: &str) -> Result<String, String> {
        let path = PathBuf::from(query);
        self.unload_plugin(path);
        Ok(format!("Unloaded '{}'", query))
    }

    pub fn find_plugin_index(&self, name: &str) -> Option<usize> {
        self.plugins.iter().position(|p| p.name == name)
    }

    pub fn get_status_info(&self) -> (usize, usize) {
        (self.plugins.len(), 0)
    }

    pub fn call_on_frame(&mut self) {
        for plugin in &mut self.plugins {
            let _ = plugin.call_on_frame();
        }
    }

    pub fn call_on_event(&mut self, name: &str, data: &[u8]) {
        for plugin in &mut self.plugins {
            let _ = plugin.call_on_event(name, data);
        }
    }

    pub fn call_on_command(&mut self, cmd: &str, args: &str) -> bool {
        for plugin in &mut self.plugins {
            let _ = plugin.call_on_command(cmd, args);
        }
        false
    }
}
