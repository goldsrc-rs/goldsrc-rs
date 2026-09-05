//! Host-side runtime orchestrator, engine bridge, configuration, storage, and hook dispatcher for GoldSrc.rs.

pub mod api_registry;
pub mod backend;
pub mod chat;
#[cfg(feature = "cli")]
pub mod cli;
pub mod config;
pub mod hooks;
pub mod host;
pub mod hud;
pub mod i18n;
pub mod logging;
pub mod menu;
pub mod paths;
pub mod placeholders;
pub mod reapi;
pub mod rules;
pub mod storage;

pub use ::log;
pub use chat::process_chat_message;
pub use config::plugins as plugins_config;
pub use config::{
    HostConfig, PluginDebugConfig, PluginDebugSetting, PluginEntry, PluginGroup, PluginsConfig,
};
pub use host::HostRuntime;
pub use i18n::{I18nEngine, I18nService};
pub use paths::PathResolver;
pub use placeholders::{PlaceholderRegistry, format_placeholders};
pub use reapi::ReApiBridge;
pub use storage::{Bucket, JsonFormat, SqliteStorageEngine, StorageFormat};
