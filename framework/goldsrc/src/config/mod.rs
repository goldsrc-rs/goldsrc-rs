//! System and plugin configuration schemas (`goldsrc.toml` and `plugins.toml`).

pub mod host;
pub mod plugins;

pub use host::{
    CoreConfig, DEFAULT_COMMAND_EPOCH_DEADLINE, DEFAULT_DEBOUNCE_MS, DEFAULT_EVENT_EPOCH_DEADLINE,
    DEFAULT_FRAME_EPOCH_DEADLINE, DEFAULT_LOAD_EPOCH_DEADLINE, DEFAULT_MAX_TABLE_ELEMENTS,
    DEFAULT_MEMORY_LIMIT_MB, HostConfig, MAX_DEBOUNCE_MS, MAX_FRAME_EPOCH_DEADLINE,
    MAX_MEMORY_LIMIT_MB, MAX_TABLE_ELEMENTS, MIN_DEBOUNCE_MS, MIN_FRAME_EPOCH_DEADLINE,
    MIN_MEMORY_LIMIT_MB, MIN_TABLE_ELEMENTS, OnFileDeleted, RuntimeConfig, WatcherConfig,
};
pub use plugins::{
    PluginDebugConfig, PluginDebugSetting, PluginEntry, PluginEntryItem, PluginGroup,
    PluginLogLevel, PluginsConfig, PluginsSection, RuleConfig, RuleItemConfig, RulesSection,
};
