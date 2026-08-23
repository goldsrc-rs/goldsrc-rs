//! Public framework (SDK) for GoldSrc.rs plugin developers.
//!
//! This is the main entry point for plugin developers. It re-exports
//! everything you need from the other crates.

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

/// `goldsrc.toml` configuration types and loader.
pub mod config;
/// Centralized hook dispatching helpers.
#[cfg(feature = "host")]
pub mod hooks;
/// Unified structured logger for backends.
pub mod logging;
/// Filesystem path resolution helpers.
pub mod paths;

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::wasm_log::log_info(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::wasm_log::log_warn(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {
        $crate::wasm_log::log_err(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::wasm_log::log_debug(&format!($($arg)*))
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
    MenuBuilder, MenuContext, MenuItem, MenuRendererKind, MenuStyle, Player, PlayerStateFilter,
    RenderedMenuPage, SlotAction, Spectator, Team, Terrorist, Vector3, VisualDeny,
};
pub use goldsrc_macros as macros;
pub use goldsrc_macros::{command, event, on_load, plugin};

/// Convenient prelude module for plugin authors.
pub mod prelude {
    pub use crate::ecs::*;
    pub use crate::engine;
    pub use crate::hud_api as hud;
    pub use crate::menu_api;
    pub use crate::{
        Alive, Auth, Bot, CapExpr, ChatScope, ClientKind, Command, CommandBuilder, CommandContext,
        CommandError, CommandResult, CommandTarget, Condition, ConnectionState, CounterTerrorist,
        Dead, DenyAction, DenyPolicy, Engine, Entity, ExitBehavior, FromArg, HLTV, HudColor,
        HudCoord, HudEffect, HudKind, HudMessage, HudMessageBuilder, ItemKind, ItemTitle,
        LifeState, Menu, MenuBuilder, MenuContext, MenuItem, MenuRendererKind, MenuStyle, Player,
        PlayerStateFilter, RenderedMenuPage, SlotAction, Spectator, Team, Terrorist, Vector3,
        VisualDeny,
    };
    pub use crate::{command, event, on_load, plugin};
    pub use crate::{log_debug, log_err, log_info, log_warn};
}

/// Direct logging helper for WASM plugins.
pub mod wasm_log {
    /// Logs an info message.
    pub fn log_info(msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::goldsrc_api::bindings::goldsrc::engine::api::host_log(&format!(
                "[INFO] {}",
                msg
            ));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            ::log::info!(target: "plugin", "{}", msg);
        }
    }

    /// Logs a warning message.
    pub fn log_warn(msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::goldsrc_api::bindings::goldsrc::engine::api::host_log(&format!(
                "[WARN] {}",
                msg
            ));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            ::log::warn!(target: "plugin", "{}", msg);
        }
    }

    /// Logs an error message.
    pub fn log_err(msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::goldsrc_api::bindings::goldsrc::engine::api::host_log(&format!(
                "[ERROR] {}",
                msg
            ));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            ::log::error!(target: "plugin", "{}", msg);
        }
    }

    /// Logs a debug message.
    pub fn log_debug(msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::goldsrc_api::bindings::goldsrc::engine::api::host_log(&format!(
                "[DEBUG] {}",
                msg
            ));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            ::log::debug!(target: "plugin", "{}", msg);
        }
    }

    /// Forwards raw `msg` to the host logger (WASM) or `println!` (native).
    pub fn print(msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::goldsrc_api::bindings::goldsrc::engine::api::host_log(msg);
        }
        #[cfg(not(target_arch = "wasm32"))]
        println!("{}", msg);
    }
}
