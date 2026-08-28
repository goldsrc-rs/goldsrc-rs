//! Declarative `plugins.toml` Configuration Model & Parser.
//!
//! Provides granular plugin debugging, profile groups, and reactive rules.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// Log level for plugin debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PluginLogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

/// Detailed debugging and profiling configuration for a plugin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginDebugConfig {
    /// Logging verbosity level.
    #[serde(default)]
    pub level: PluginLogLevel,
    /// Whether to profile per-tick and event execution times.
    #[serde(default)]
    pub profile: bool,
    /// Specific events to trace/log.
    #[serde(default)]
    pub log_events: Vec<String>,
    /// Whether to log command invocations and arguments.
    #[serde(default = "default_true")]
    pub log_commands: bool,
    /// Watchdog epoch deadline override (default: 100 epochs = ~200ms).
    #[serde(default = "default_epoch_limit")]
    pub epoch_limit: u64,
    /// Dedicated log file for this plugin (in `logs/` directory).
    #[serde(default)]
    pub log_file: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_epoch_limit() -> u64 {
    100
}

impl Default for PluginDebugConfig {
    fn default() -> Self {
        Self {
            level: PluginLogLevel::Info,
            profile: false,
            log_events: Vec::new(),
            log_commands: true,
            epoch_limit: 100,
            log_file: None,
        }
    }
}

/// Debug setting which can be either a simple boolean (`debug = true`) or a full table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginDebugSetting {
    Simple(bool),
    Detailed(PluginDebugConfig),
}

impl PluginDebugSetting {
    /// Resolves the setting into a full `PluginDebugConfig`.
    pub fn resolve(&self) -> PluginDebugConfig {
        match self {
            Self::Simple(enabled) => {
                if *enabled {
                    PluginDebugConfig {
                        level: PluginLogLevel::Debug,
                        profile: true,
                        ..Default::default()
                    }
                } else {
                    PluginDebugConfig::default()
                }
            }
            Self::Detailed(cfg) => cfg.clone(),
        }
    }
}

/// Individual plugin entry in `plugins.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginEntry {
    /// Plugin name or relative path (e.g. "admin_system", "test_suite/test_hud").
    pub name: String,
    /// Whether the plugin is enabled for loading.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Load priority (higher priority loads earlier, default 100).
    #[serde(default = "default_priority")]
    pub priority: i32,
    /// Debugging and profiling configuration.
    #[serde(default)]
    pub debug: Option<PluginDebugSetting>,
}

fn default_priority() -> i32 {
    100
}

/// A named profile group of plugins (e.g. `[groups.vip_pack]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginGroup {
    /// Whether the entire group is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// List of plugin names or relative paths belonging to this group.
    #[serde(default)]
    pub plugins: Vec<String>,
}

/// A reactive lifecycle rule in `plugins.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleConfig {
    /// Name or description of the rule.
    pub name: String,
    /// Condition map: condition name -> TOML value.
    #[serde(default)]
    pub when: BTreeMap<String, toml::Value>,
    /// Action map: action name -> TOML value.
    #[serde(default)]
    pub action: BTreeMap<String, toml::Value>,
}

/// Root configuration schema for `plugins.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PluginsConfig {
    /// Explicit list of plugin declarations.
    #[serde(default)]
    pub plugins: Vec<PluginEntry>,
    /// Named profile groups (`[groups.<name>]`).
    #[serde(default)]
    pub groups: HashMap<String, PluginGroup>,
    /// Reactive lifecycle rules (`[[rules]]`).
    #[serde(default)]
    pub rules: Vec<RuleConfig>,
}

impl PluginsConfig {
    /// Parses a `plugins.toml` string.
    pub fn parse(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// Checks whether a plugin is enabled, taking both individual entry and groups into account.
    pub fn is_plugin_enabled(&self, plugin_name: &str) -> bool {
        // 1. If any group containing this plugin is explicitly disabled, the plugin is disabled
        for group in self.groups.values() {
            if !group.enabled && group.plugins.iter().any(|p| p == plugin_name) {
                return false;
            }
        }

        // 2. Check individual entry
        if let Some(entry) = self.plugins.iter().find(|p| p.name == plugin_name) {
            return entry.enabled;
        }

        // 3. Default to enabled if not explicitly declared or disabled
        true
    }

    /// Returns the resolved `PluginDebugConfig` for a given plugin.
    pub fn get_debug_config(&self, plugin_name: &str) -> PluginDebugConfig {
        self.plugins
            .iter()
            .find(|p| p.name == plugin_name)
            .and_then(|p| p.debug.as_ref())
            .map(|d| d.resolve())
            .unwrap_or_default()
    }

    /// Loads `plugins.toml` from the specified path, or generates a documented default template.
    pub fn load_or_create(config_path: &std::path::Path) -> Self {
        if config_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(config_path) {
                match Self::parse(&content) {
                    Ok(cfg) => return cfg,
                    Err(e) => {
                        log::error!(
                            target: "wasm",
                            "CRITICAL: Failed to parse '{:?}': {e}. Preserving file and using default in-memory config.",
                            config_path
                        );
                        // Backup the corrupted file so administrator edits are not lost
                        let bak_path = config_path.with_extension("toml.bak");
                        let _ = std::fs::copy(config_path, &bak_path);
                        return Self::default();
                    }
                }
            } else {
                log::warn!(
                    target: "wasm",
                    "Failed to read '{:?}', using default discovery.",
                    config_path
                );
                return Self::default();
            }
        }

        let default_template = include_str!("../resources/plugins.template.toml");

        if let Some(parent) = config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(config_path, default_template);

        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plugins_config() {
        let toml_str = r#"
            [[plugins]]
            name = "admin_system"
            enabled = true
            priority = 150
            debug = true

            [[plugins]]
            name = "vip_core"
            enabled = true
            [plugins.debug]
            level = "trace"
            profile = true
            epoch_limit = 250
            log_file = "vip_debug.log"

            [groups.fun_mods]
            enabled = false
            plugins = ["gungame", "paintball"]

            [[rules]]
            name = "disable_on_warmup"
            when = { cvar = "mp_warmup_time > 0", map = ["fy_*", "aim_*"] }
            action = { pause = ["vip_core"], set_cvar = { "sv_gravity" = 700 } }
        "#;

        let cfg = PluginsConfig::parse(toml_str).unwrap();
        assert_eq!(cfg.plugins.len(), 2);
        assert_eq!(cfg.plugins[0].name, "admin_system");
        assert_eq!(cfg.plugins[0].priority, 150);

        let debug0 = cfg.get_debug_config("admin_system");
        assert_eq!(debug0.level, PluginLogLevel::Debug);
        assert!(debug0.profile);

        let debug1 = cfg.get_debug_config("vip_core");
        assert_eq!(debug1.level, PluginLogLevel::Trace);
        assert_eq!(debug1.epoch_limit, 250);
        assert_eq!(debug1.log_file, Some("vip_debug.log".to_string()));

        assert!(cfg.is_plugin_enabled("admin_system"));
        assert!(!cfg.is_plugin_enabled("gungame")); // disabled via fun_mods group
        assert!(cfg.is_plugin_enabled("unlisted_plugin"));

        assert_eq!(cfg.rules.len(), 1);
        assert_eq!(cfg.rules[0].name, "disable_on_warmup");
        assert!(cfg.rules[0].when.contains_key("cvar"));
        assert!(cfg.rules[0].action.contains_key("pause"));
    }
}
