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
/// Unified structured logger for backends.
pub mod logging;
/// Filesystem path resolution helpers.
pub mod paths;

pub use config::*;
pub use ecs::*;
pub use goldsrc_api;
pub use goldsrc_api::{auth::Auth, Engine, Entity, Player, Plugin, Vector3};
pub use goldsrc_macros::{command, event, on_load, plugin};
pub use goldsrc_sys;

/// Logging subsystem for WASM plugins.
pub mod log {
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

/// Print an info message to the server console.
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::log::print(&format!("\x1b[36m[INFO]\x1b[0m {}\n", format_args!($($arg)*)))
    };
}

/// Print a warning message to the server console.
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::log::print(&format!("\x1b[33m[WARN]\x1b[0m {}\n", format_args!($($arg)*)))
    };
}

/// Print an error message to the server console.
#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {
        $crate::log::print(&format!("\x1b[31m[ERROR]\x1b[0m {}\n", format_args!($($arg)*)))
    };
}
