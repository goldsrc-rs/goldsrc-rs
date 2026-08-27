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

pub use manager::{PluginInfo, PluginManager};
pub use plugin::PluginStatus;

pub type PrintCallback = fn(&str);
pub type ShowMenuCallback = fn(i32, i32, i32, &str);

static PRINT_CALLBACK: std::sync::RwLock<Option<PrintCallback>> = std::sync::RwLock::new(None);
static SHOW_MENU_CALLBACK: std::sync::RwLock<Option<ShowMenuCallback>> =
    std::sync::RwLock::new(None);

/// Set global callback for WASM server_print calls.
pub fn set_print_callback(f: PrintCallback) {
    if let Ok(mut lock) = PRINT_CALLBACK.write() {
        *lock = Some(f);
    }
}

/// Set global callback when WASM plugins call `host_show_menu`.
pub fn set_show_menu_callback(f: ShowMenuCallback) {
    if let Ok(mut lock) = SHOW_MENU_CALLBACK.write() {
        *lock = Some(f);
    }
}

pub(crate) fn notify_show_menu(player_idx: i32, keys_mask: i32, timeout: i32, text: &str) {
    if let Ok(lock) = SHOW_MENU_CALLBACK.read() {
        if let Some(cb) = *lock {
            cb(player_idx, keys_mask, timeout, text);
        }
    }
}

/// Print log message via host callback (engine server_print and unified logger).
pub fn host_log(msg: &str) {
    let bounded = if msg.len() > 4096 {
        let mut end = 4096;
        while end > 0 && !msg.is_char_boundary(end) {
            end -= 1;
        }
        &msg[..end]
    } else {
        msg
    };
    let (level, clean_msg) = if let Some(rest) = bounded.strip_prefix("[ERROR] ") {
        (log::Level::Error, rest)
    } else if let Some(rest) = bounded.strip_prefix("[WARN] ") {
        (log::Level::Warn, rest)
    } else if let Some(rest) = bounded.strip_prefix("[DEBUG] ") {
        (log::Level::Debug, rest)
    } else if let Some(rest) = bounded.strip_prefix("[TRACE] ") {
        (log::Level::Trace, rest)
    } else if let Some(rest) = bounded.strip_prefix("[INFO] ") {
        (log::Level::Info, rest)
    } else {
        (log::Level::Info, bounded)
    };

    match level {
        log::Level::Error => log::error!(target: "plugin", "{clean_msg}"),
        log::Level::Warn => log::warn!(target: "plugin", "{clean_msg}"),
        log::Level::Debug => log::debug!(target: "plugin", "{clean_msg}"),
        log::Level::Trace => log::trace!(target: "plugin", "{clean_msg}"),
        log::Level::Info => log::info!(target: "plugin", "{clean_msg}"),
    }
}
