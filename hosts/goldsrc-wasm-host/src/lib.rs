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

/// Print log message via host callback (engine server_print and unified logger).
pub fn host_log(msg: &str) {
    let (level, clean_msg) = if let Some(rest) = msg.strip_prefix("[ERROR] ") {
        (log::Level::Error, rest)
    } else if let Some(rest) = msg.strip_prefix("[WARN] ") {
        (log::Level::Warn, rest)
    } else if let Some(rest) = msg.strip_prefix("[DEBUG] ") {
        (log::Level::Debug, rest)
    } else if let Some(rest) = msg.strip_prefix("[TRACE] ") {
        (log::Level::Trace, rest)
    } else if let Some(rest) = msg.strip_prefix("[INFO] ") {
        (log::Level::Info, rest)
    } else {
        (log::Level::Info, msg)
    };

    match level {
        log::Level::Error => log::error!(target: "plugin", "{clean_msg}"),
        log::Level::Warn => log::warn!(target: "plugin", "{clean_msg}"),
        log::Level::Debug => log::debug!(target: "plugin", "{clean_msg}"),
        log::Level::Trace => log::trace!(target: "plugin", "{clean_msg}"),
        log::Level::Info => log::info!(target: "plugin", "{clean_msg}"),
    }
}
