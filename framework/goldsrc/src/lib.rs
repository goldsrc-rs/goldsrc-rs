//! Public framework (SDK) for GoldSrc.rs plugin developers.
//!
//! This is the main entry point for plugin developers. It re-exports
//! everything you need from the other crates.

pub mod ecs;

pub use ecs::*;
pub use goldsrc_api::{Engine, Entity, Player, Plugin, Vector3};
pub use goldsrc_macros::{command, event, on_load, plugin};
pub use goldsrc_sys;

/// Logging subsystem for WASM plugins.
pub mod log {
    #[cfg(target_arch = "wasm32")]
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn server_print(ptr: *const u8, len: usize);
    }

    pub fn print(msg: &str) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            server_print(msg.as_ptr(), msg.len());
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
