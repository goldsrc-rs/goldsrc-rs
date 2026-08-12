#![allow(clippy::collapsible_if)]

//! WASM plugin host for GoldSrc.rs.
//!
//! Uses `wasmtime` (with `pulley32`) as the pure-Rust WASM runtime for maximum compatibility
//! with 32-bit HLDS. Implements the WASM Component Model via `wit-bindgen`.

pub mod bindings;
pub mod manager;
pub mod plugin;

use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

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

pub fn file_log(msg: &str) {
    use goldsrc_sys::paths::{ADDONS_DIR_NAME, DEFAULT_MOD_DIR, FRAMEWORK_NAME, LOGS_DIR_NAME};
    let logs_dir = Path::new(DEFAULT_MOD_DIR)
        .join(ADDONS_DIR_NAME)
        .join(FRAMEWORK_NAME)
        .join(LOGS_DIR_NAME);
    let _ = fs::create_dir_all(&logs_dir);
    let log_file_path = logs_dir.join(format!("{}.log", FRAMEWORK_NAME));

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
    {
        let _ = writeln!(file, "{}", msg);
    }
}
