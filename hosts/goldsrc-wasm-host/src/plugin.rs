use crate::bindings::GoldsrcPlugin;
use crate::manager::HostState;
use std::collections::HashMap;
use std::path::PathBuf;
use wasmtime::Store;
use wasmtime::component::Component;

/// Metadata structure exported by WASM plugins generated via the `#[plugin]` macro.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PluginMetadata {
    /// Plugin display name.
    pub name: String,
    /// Plugin version string.
    #[serde(default = "default_version")]
    pub version: String,
    /// Plugin author.
    #[serde(default = "default_author")]
    pub author: String,
    /// Plugin description.
    #[serde(default)]
    pub description: String,
    /// Registered system names (from `#[plugin(system = ...)]`).
    #[serde(default)]
    pub systems: Vec<String>,
    /// Commands this plugin handles (from `#[command(name = ...)]`).
    #[serde(default)]
    pub commands: Vec<String>,
    /// Plugin dependencies: name -> version requirement.
    #[serde(default, deserialize_with = "deserialize_deps")]
    pub dependencies: HashMap<String, String>,
}

fn default_author() -> String {
    "Unknown".to_string()
}
fn default_version() -> String {
    "1.0.0".to_string()
}

fn deserialize_deps<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum RawDeps {
        List(Vec<String>),
        Map(HashMap<String, String>),
    }
    let mut map = HashMap::new();
    match RawDeps::deserialize(deserializer)? {
        RawDeps::Map(m) => return Ok(m),
        RawDeps::List(list) => {
            for item in list {
                let parts: Vec<&str> = item.split('@').collect();
                if parts.len() == 2 {
                    map.insert(parts[0].to_string(), parts[1].to_string());
                } else {
                    map.insert(item, "*".to_string());
                }
            }
        }
    }
    Ok(map)
}

/// Single loaded WASM component instance.
pub struct LoadedPlugin {
    /// Plugin name (derived from file stem).
    pub name: String,
    /// Path the plugin was loaded from.
    pub path: PathBuf,
    /// When `true`, frame/event/command callbacks are skipped.
    pub is_paused: bool,
    /// Set when the plugin panicked/trapped; skips future callbacks.
    pub is_poisoned: bool,
    /// Parsed `get_metadata` output, if any.
    pub metadata: Option<PluginMetadata>,
    /// Wasmtime store holding plugin host state.
    pub store: Store<HostState>,
    /// Generated component bindings for calling the plugin.
    pub bindings: GoldsrcPlugin,
    /// Compiled component used to inspect which exports exist.
    pub(crate) component: Component,
}

impl LoadedPlugin {
    /// Invokes the plugin's `on_load` export.
    pub fn call_on_load(&mut self) -> wasmtime::Result<()> {
        self.bindings.call_on_load(&mut self.store)
    }

    /// Invokes the plugin's `on_unload` export, if present.
    pub fn call_on_unload(&mut self) -> wasmtime::Result<()> {
        self.bindings.call_on_unload(&mut self.store)
    }

    /// Invokes the plugin's `on_frame` export (skipped if paused/poisoned).
    /// On trap/panic marks the plugin poisoned and logs once.
    pub fn call_on_frame(&mut self) -> wasmtime::Result<()> {
        if self.is_paused || self.is_poisoned {
            return Ok(());
        }
        let res = self.bindings.call_on_frame(&mut self.store);
        if let Err(ref e) = res {
            self.poison(e);
        }
        res
    }

    /// Invokes the plugin's `on_event` export (skipped if paused/poisoned).
    pub fn call_on_event(&mut self, event_name: &str, data: &[u8]) -> wasmtime::Result<()> {
        if self.is_paused || self.is_poisoned {
            return Ok(());
        }
        let res = self
            .bindings
            .call_on_event(&mut self.store, event_name, data);
        if let Err(ref e) = res {
            self.poison(e);
        }
        res
    }

    /// Invokes the plugin's `on_command` export (skipped if paused/poisoned).
    /// Returns whether the plugin consumed the command.
    pub fn call_on_command(&mut self, cmd_name: &str, args: &str) -> wasmtime::Result<bool> {
        if self.is_paused || self.is_poisoned {
            return Ok(false);
        }
        let res = self
            .bindings
            .call_on_command(&mut self.store, cmd_name, args);
        match res {
            Err(e) => {
                self.poison(&e);
                Err(e)
            }
            Ok(consumed) => Ok(consumed),
        }
    }

    /// Returns whether the component exports a top-level function `name`.
    pub fn has_export(&self, name: &str) -> bool {
        self.component.get_export(None, name).is_some()
    }

    /// Marks the plugin poisoned on a trap/panic and logs once.
    fn poison(&mut self, err: &wasmtime::Error) {
        self.is_poisoned = true;
        crate::host_log(&format!(
            "Plugin '{}' panicked and was poisoned: {}",
            self.name, err
        ));
    }
}
