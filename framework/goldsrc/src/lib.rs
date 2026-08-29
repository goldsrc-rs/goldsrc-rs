//! Public framework (SDK) for GoldSrc.rs plugin developers.
//!
//! This is the main entry point for plugin developers. It re-exports
//! everything you need from the other crates.

/// Unified FFI registration point for engine function tables (single source
/// of truth for `DLL_FUNCTIONS` hooks). Enabled by the `host` feature.
#[cfg(feature = "host")]
pub mod api_registry;

/// Flat ECS for plugin state storage.
pub mod ecs;

/// Shared host CLI (`meta-rs`/`mrs`). Enabled by the `host-cli` feature.
#[cfg(feature = "host-cli")]
pub mod cli;

/// Host runtime orchestrator. Enabled by the `host` feature.
#[cfg(feature = "host")]
pub mod host;

/// Shared backend plumbing (engine access, print queue, engfunc macros).
/// Enabled by the `host` feature.
#[cfg(feature = "host")]
pub mod backend;

/// System (`goldsrc.toml`) and plugins (`plugins.toml`) configuration models.
pub mod config;
/// Backward-compatible alias for plugins_config module.
pub use config::plugins as plugins_config;
pub use config::{
    HostConfig, PluginDebugConfig, PluginDebugSetting, PluginEntry, PluginGroup, PluginsConfig,
};
/// Centralized hook dispatching helpers.
#[cfg(feature = "host")]
pub mod hooks;
/// Lightweight i18n & Localization Dictionary Engine.
pub mod i18n;
/// Unified structured logger for backends and transparent WASM guest logger.
pub mod logging;
pub use logging::init_guest_logger;
/// Filesystem path resolution helpers.
pub mod paths;
/// Built-in server reactive rule providers and executors.
#[cfg(feature = "host")]
pub mod rules;
/// Unified SQLite WAL storage engine and background batching.
pub mod storage;
pub use i18n::I18nEngine;

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            #[cfg(target_arch = "wasm32")]
            $crate::logging::init_guest_logger();
            $crate::log::info!(target: "plugin", $($arg)*)
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        {
            #[cfg(target_arch = "wasm32")]
            $crate::logging::init_guest_logger();
            $crate::log::warn!(target: "plugin", $($arg)*)
        }
    };
}

#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {
        {
            #[cfg(target_arch = "wasm32")]
            $crate::logging::init_guest_logger();
            $crate::log::error!(target: "plugin", $($arg)*)
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            #[cfg(target_arch = "wasm32")]
            $crate::logging::init_guest_logger();
            $crate::log::debug!(target: "plugin", $($arg)*)
        }
    };
}

/// Screen HUD and DHUD message serialization and formatting.
#[cfg(feature = "host")]
pub mod hud;
/// Runtime menu session manager and pagination.
#[cfg(feature = "host")]
pub mod menu;

pub use ::log;
pub use config::*;
pub use ecs::*;
pub use goldsrc_api as api;
pub use goldsrc_api;
pub use goldsrc_api::engine_api as engine;
pub use goldsrc_api::hud as hud_api;
pub use goldsrc_api::menu as menu_api;
pub use goldsrc_api::{
    Alive, Auth, Bot, CapExpr, ChatScope, ClientKind, Command, CommandBuilder, CommandContext,
    CommandError, CommandResult, CommandTarget, Condition, ConnectionState, CounterTerrorist, Dead,
    DenyAction, DenyPolicy, Engine, Entity, ExitBehavior, FromArg, HLTV, HudColor, HudCoord,
    HudEffect, HudKind, HudMessage, HudMessageBuilder, ItemKind, ItemTitle, LifeState, Menu,
    MenuBuilder, MenuContext, MenuItem, MenuPageBuilder, MenuRendererKind, MenuStyle, Player,
    PlayerStateFilter, RenderedMenuPage, SlotAction, Spectator, SqlDatabase, StorageError,
    StorageProvider, Team, Terrorist, Vector3, VisualDeny,
};
pub use goldsrc_macros as macros;
pub use goldsrc_macros::{
    command, event, menu_action, on_frame, on_load, on_unload, plugin, system,
};
pub use storage::Bucket;

/// Convenient prelude module for plugin authors.
pub mod prelude {
    pub use crate::ecs::*;
    pub use crate::engine;
    pub use crate::hud_api as hud;
    pub use crate::menu_api;
    pub use crate::tr;
    pub use crate::{
        Alive, Auth, Bot, Bucket, CapExpr, ChatScope, ClientKind, Command, CommandBuilder,
        CommandContext, CommandError, CommandResult, CommandTarget, Condition, ConnectionState,
        CounterTerrorist, Dead, DenyAction, DenyPolicy, Engine, Entity, ExitBehavior, FromArg,
        HLTV, HudColor, HudCoord, HudEffect, HudKind, HudMessage, HudMessageBuilder, ItemKind,
        ItemTitle, LifeState, Menu, MenuBuilder, MenuContext, MenuItem, MenuPageBuilder,
        MenuRendererKind, MenuStyle, Player, PlayerStateFilter, RenderedMenuPage, SlotAction,
        Spectator, SqlDatabase, StorageError, StorageProvider, Team, Terrorist, Vector3,
        VisualDeny,
    };
    pub use crate::{command, event, menu_action, on_frame, on_load, on_unload, plugin, system};
    pub use crate::{log_debug, log_err, log_info, log_warn};
}
