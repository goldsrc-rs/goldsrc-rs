use crate::bindings::GoldsrcPlugin;
use crate::manager::HostState;
use std::path::PathBuf;
use wasmtime::Store;
use wasmtime::component::Component;

/// Detailed command definition exported in plugin metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct CommandMetadata {
    /// Canonical primary command name.
    pub name: String,
    /// Human-readable explanation of command effect.
    #[serde(default)]
    pub description: String,
    /// Command usage syntax (e.g. `vipmenu <player_index>`).
    #[serde(default)]
    pub usage: String,
    /// List of alternative names/aliases (e.g. `["vip", "/vip", "!vip"]`).
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Required capability expression for authorization (e.g. `Some("vip.access")`).
    #[serde(default)]
    pub capability: Option<String>,
}

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
    #[serde(default = "default_description")]
    pub description: String,
    /// Plugin website or repository URL.
    #[serde(default = "default_url")]
    pub url: String,
    /// Plugin license (e.g. `MIT`, `GPL-3.0`, `Proprietary`).
    #[serde(default = "default_license")]
    pub license: String,
    /// Target subfolder or bundle directory within `plugins/` (e.g. `test_suite`, `admin_system`).
    #[serde(default)]
    pub bundle: Option<String>,
    /// Registered system names (from `#[plugin(system = ...)]`).
    #[serde(default)]
    pub systems: Vec<String>,
    /// Commands this plugin handles (from `#[command(name = ...)]`).
    #[serde(default)]
    pub commands: Vec<String>,
    /// Structured command definitions with descriptions, usage, aliases, and permissions.
    #[serde(default)]
    pub command_defs: Vec<CommandMetadata>,
    /// Unified requirements DSL expressions (e.g. `["plugin:vip_core", "cvar:vip_enabled!=0"]`).
    #[serde(default)]
    pub require: Vec<String>,
    /// Explicitly allowed shared storage buckets (e.g. `["global/ranks"]`).
    #[serde(default)]
    pub shared_buckets: Vec<String>,
}

fn default_author() -> String {
    goldsrc_api::consts::DEFAULT_PLUGIN_AUTHOR.to_string()
}
fn default_version() -> String {
    goldsrc_api::consts::DEFAULT_PLUGIN_VERSION.to_string()
}
fn default_description() -> String {
    goldsrc_api::consts::DEFAULT_PLUGIN_DESCRIPTION.to_string()
}
fn default_license() -> String {
    goldsrc_api::consts::DEFAULT_PLUGIN_LICENSE.to_string()
}
fn default_url() -> String {
    goldsrc_api::consts::DEFAULT_PLUGIN_URL.to_string()
}

/// Unified lifecycle state of a WASM plugin.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum PluginStatus {
    /// 1. Instantiated in memory, but `on_load` not yet executed.
    Loaded,
    /// 2. Active, fully initialized and executing ticks/events/commands.
    Running,
    /// 3. Paused by administrator, profile group, or reactive rule.
    Paused {
        #[serde(default)]
        reason: Option<String>,
    },
    /// 4. Cannot run due to missing mandatory requirements.
    Blocked { reason: String },
    /// 5. Temporarily degraded because a requirement is paused.
    Degraded { reason: String },
    /// 6. Trapped/panicked in WASM runtime and isolated.
    Poisoned { error: String },
    /// 7. Completely unloaded (`on_unload` executed, inactive).
    Unloaded,
}

impl PluginStatus {
    /// Returns whether the plugin can currently execute callbacks.
    pub fn is_executable(&self) -> bool {
        matches!(self, PluginStatus::Running)
    }

    /// Returns whether the plugin is currently in a running/loaded state.
    pub fn is_active(&self) -> bool {
        matches!(self, PluginStatus::Running | PluginStatus::Loaded)
    }

    /// Short label representation for CLI tables.
    pub fn label(&self) -> &'static str {
        match self {
            PluginStatus::Loaded => "LOADED",
            PluginStatus::Running => "RUNNING",
            PluginStatus::Paused { .. } => "PAUSED",
            PluginStatus::Blocked { .. } => "BLOCKED",
            PluginStatus::Degraded { .. } => "DEGRADED",
            PluginStatus::Poisoned { .. } => "POISONED",
            PluginStatus::Unloaded => "UNLOADED",
        }
    }
}

/// Epoch deadline constants for WASM execution timeout protection.
/// The background epoch timer thread increments epoch every 2ms.
/// 250 epochs = ~500ms, 500 epochs = ~1000ms (1 second).
pub const EPOCH_DEADLINE_FRAME: u64 = 250;
pub const EPOCH_DEADLINE_EVENT: u64 = 500;
pub const EPOCH_DEADLINE_COMMAND: u64 = 500;
pub const EPOCH_DEADLINE_LOAD: u64 = 1000;

/// Single loaded WASM component instance.
pub struct LoadedPlugin {
    /// Plugin name (derived from file stem).
    pub name: String,
    /// Path the plugin was loaded from.
    pub path: PathBuf,
    /// Current lifecycle status of the plugin.
    pub status: PluginStatus,
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
    /// Invokes the plugin's `on_load` export and transitions to Running.
    pub fn call_on_load(&mut self) -> wasmtime::Result<()> {
        self.store.set_epoch_deadline(EPOCH_DEADLINE_LOAD);
        let res = self.bindings.call_on_load(&mut self.store);
        if res.is_ok() {
            self.status = PluginStatus::Running;
        }
        res
    }

    /// Invokes the plugin's `on_unload` export, if present, and transitions to Unloaded.
    pub fn call_on_unload(&mut self) -> wasmtime::Result<()> {
        self.store.set_epoch_deadline(EPOCH_DEADLINE_LOAD);
        let res = self.bindings.call_on_unload(&mut self.store);
        self.status = PluginStatus::Unloaded;
        res
    }

    /// Invokes the plugin's `on_frame` export (skipped if not executable).
    /// On trap/panic marks the plugin poisoned and logs once.
    pub fn call_on_frame(&mut self) -> wasmtime::Result<()> {
        if !self.status.is_executable() {
            return Ok(());
        }
        self.store.set_epoch_deadline(EPOCH_DEADLINE_FRAME);
        let res = self.bindings.call_on_frame(&mut self.store);
        if let Err(ref e) = res {
            self.poison(e);
        }
        res
    }

    /// Invokes the plugin's `on_event` export (skipped if not executable).
    pub fn call_on_event(&mut self, event_name: &str, data: &[u8]) -> wasmtime::Result<()> {
        if !self.status.is_executable() {
            return Ok(());
        }
        self.store.set_epoch_deadline(EPOCH_DEADLINE_EVENT);
        let res = self
            .bindings
            .call_on_event(&mut self.store, event_name, data);
        if let Err(ref e) = res {
            self.poison(e);
        }
        res
    }

    /// Invokes the plugin's `on_command` export (skipped if not executable).
    /// Returns whether the plugin consumed the command.
    pub fn call_on_command(
        &mut self,
        cmd_name: &str,
        caller: i32,
        args: &str,
    ) -> wasmtime::Result<bool> {
        if !self.status.is_executable() {
            return Ok(false);
        }
        self.store.set_epoch_deadline(EPOCH_DEADLINE_COMMAND);
        let res = self
            .bindings
            .call_on_command(&mut self.store, cmd_name, caller, args);
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
        self.status = PluginStatus::Poisoned {
            error: err.to_string(),
        };
        crate::host_log(&format!(
            "Plugin '{}' panicked and was poisoned: {}",
            self.name, err
        ));
    }
}
