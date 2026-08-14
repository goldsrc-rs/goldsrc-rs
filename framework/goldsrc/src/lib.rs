//! Public framework (SDK) for GoldSrc.rs plugin developers.
//!
//! This is the main entry point for plugin developers. It re-exports
//! everything you need from the other crates.

pub mod ecs;

#[cfg(feature = "host-cli")]
pub mod cli;

#[cfg(feature = "host")]
pub mod host;

pub use ecs::*;
pub use goldsrc_api;
pub use goldsrc_api::{auth::Auth, events::*, Engine, Entity, Player, Plugin, Vector3};
pub use goldsrc_macros::{command, event, on_load, plugin};
pub use goldsrc_sys;

/// Logging subsystem for WASM plugins.
pub mod log {
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
