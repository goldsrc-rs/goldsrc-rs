#![allow(clippy::collapsible_if)]

//! WASM plugin host for GoldSrc.rs.
//!
//! Uses `wasmtime` (with `pulley32`) as the pure-Rust WASM runtime for maximum compatibility
//! with 32-bit HLDS. Implements the WASM Component Model via `wit-bindgen`.

/// Generated wasmtime bindings for the `goldsrc` WIT world.
pub mod bindings;
/// Error taxonomy.
pub mod error;
/// Plugin lifecycle management and hot-reload.
pub mod manager;
/// Loaded plugin instance and metadata types.
pub mod plugin;

pub use manager::PluginManager;

static PRINT_CALLBACK: std::sync::RwLock<Option<fn(&str)>> = std::sync::RwLock::new(None);

/// Set global callback for WASM server_print calls.
pub fn set_print_callback(f: fn(&str)) {
    if let Ok(mut lock) = PRINT_CALLBACK.write() {
        *lock = Some(f);
    }
}

/// Print log message via host callback (engine server_print).
pub fn host_log(msg: &str) {
    if let Ok(lock) = PRINT_CALLBACK.read() {
        if let Some(print_fn) = *lock {
            print_fn(msg);
            return;
        }
    }
    println!("{}", msg);
}
