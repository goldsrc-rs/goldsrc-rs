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
        $crate::wasm_log::print(&format!("[INFO] {}", format_args!($($arg)*)))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::wasm_log::print(&format!("[WARN] {}", format_args!($($arg)*)))
    };
}

#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {
        $crate::wasm_log::print(&format!("[ERROR] {}", format_args!($($arg)*)))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::wasm_log::print(&format!("[DEBUG] {}", format_args!($($arg)*)))
    };
}

pub use ::log;
pub use config::*;
pub use ecs::*;
pub use goldsrc_api as api;
pub use goldsrc_api;
#[cfg(target_arch = "wasm32")]
pub use goldsrc_api::engine_api as engine;
pub use goldsrc_api::{Engine, Entity, Player, Plugin, Vector3, auth::Auth};
pub use goldsrc_macros as macros;
pub use goldsrc_macros::{command, event, on_load, plugin};

/// Convenient prelude module for plugin authors.
pub mod prelude {
    pub use crate::Auth;
    pub use crate::ecs::*;
    #[cfg(target_arch = "wasm32")]
    pub use crate::engine;
    pub use crate::{Engine, Entity, Player, Plugin, Vector3};
    pub use crate::{command, event, on_load, plugin};
    pub use crate::{log_debug, log_err, log_info, log_warn};
}

/// Direct console print helper for WASM plugins.
pub mod wasm_log {
    /// Forwards `msg` to the host logger (WASM) or `println!` (native).
    pub fn print(msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::goldsrc_api::bindings::goldsrc::engine::api::host_log(msg);
        }
        #[cfg(not(target_arch = "wasm32"))]
        println!("{}", msg);
    }
}
