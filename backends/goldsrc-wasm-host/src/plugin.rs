use crate::bindings::GoldsrcPlugin;
use crate::manager::HostState;
use std::collections::HashMap;
use std::path::PathBuf;
use wasmtime::Store;

/// Metadata structure exported by WASM plugins generated via #[plugin] macro.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PluginMetadata {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default)]
    pub systems: Vec<String>,
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
    pub name: String,
    pub path: PathBuf,
    pub is_paused: bool,
    pub is_poisoned: bool,
    pub metadata: Option<PluginMetadata>,
    pub store: Store<HostState>,
    pub bindings: GoldsrcPlugin,
}

impl LoadedPlugin {
    pub fn call_on_load(&mut self) -> wasmtime::Result<()> {
        self.bindings.call_on_load(&mut self.store)
    }

    pub fn call_on_frame(&mut self) -> wasmtime::Result<()> {
        if self.is_paused || self.is_poisoned {
            return Ok(());
        }
        self.bindings.call_on_frame(&mut self.store)
    }

    pub fn call_on_event(&mut self, event_name: &str, data: &[u8]) -> wasmtime::Result<()> {
        if self.is_paused || self.is_poisoned {
            return Ok(());
        }
        self.bindings
            .call_on_event(&mut self.store, event_name, data)
    }

    pub fn call_on_command(&mut self, cmd_name: &str, args: &str) -> wasmtime::Result<()> {
        if self.is_paused || self.is_poisoned {
            return Ok(());
        }
        self.bindings
            .call_on_command(&mut self.store, cmd_name, args)
    }
}
