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
pub type StorageGetCallback = fn(&str, &str) -> Option<Vec<u8>>;
pub type StorageSetCallback = fn(&str, &str, &[u8]) -> bool;
pub type StorageDeleteCallback = fn(&str, &str) -> bool;
pub type StorageFetchAddCallback = fn(&str, &str, i64) -> i64;
pub type TranslateCallback = fn(&str, &str, &str, &str) -> String;

static PRINT_CALLBACK: std::sync::RwLock<Option<PrintCallback>> = std::sync::RwLock::new(None);
static SHOW_MENU_CALLBACK: std::sync::RwLock<Option<ShowMenuCallback>> =
    std::sync::RwLock::new(None);
static STORAGE_GET_CB: std::sync::RwLock<Option<StorageGetCallback>> = std::sync::RwLock::new(None);
static STORAGE_SET_CB: std::sync::RwLock<Option<StorageSetCallback>> = std::sync::RwLock::new(None);
static STORAGE_DELETE_CB: std::sync::RwLock<Option<StorageDeleteCallback>> =
    std::sync::RwLock::new(None);
static STORAGE_FETCH_ADD_CB: std::sync::RwLock<Option<StorageFetchAddCallback>> =
    std::sync::RwLock::new(None);
static TRANSLATE_CB: std::sync::RwLock<Option<TranslateCallback>> = std::sync::RwLock::new(None);

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

/// Set global callbacks for WASM host storage operations.
pub fn set_storage_callbacks(
    get: StorageGetCallback,
    set: StorageSetCallback,
    delete: StorageDeleteCallback,
    fetch_add: StorageFetchAddCallback,
) {
    if let Ok(mut lock) = STORAGE_GET_CB.write() {
        *lock = Some(get);
    }
    if let Ok(mut lock) = STORAGE_SET_CB.write() {
        *lock = Some(set);
    }
    if let Ok(mut lock) = STORAGE_DELETE_CB.write() {
        *lock = Some(delete);
    }
    if let Ok(mut lock) = STORAGE_FETCH_ADD_CB.write() {
        *lock = Some(fetch_add);
    }
}

/// Set global callback for WASM host dictionary translations.
pub fn set_translate_callback(f: TranslateCallback) {
    if let Ok(mut lock) = TRANSLATE_CB.write() {
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

static ACTIVE_MENU_OWNERS: std::sync::LazyLock<
    std::sync::RwLock<std::collections::HashMap<i32, String>>,
> = std::sync::LazyLock::new(|| std::sync::RwLock::new(std::collections::HashMap::new()));

/// Registers the owning WASM plugin for an active player menu.
pub fn set_active_menu_owner(player_index: i32, owner: String) {
    if let Ok(mut lock) = ACTIVE_MENU_OWNERS.write() {
        lock.insert(player_index, owner);
    }
}

/// Clears the active menu owner for a player when their menu closes.
pub fn clear_active_menu_owner(player_index: i32) {
    if let Ok(mut lock) = ACTIVE_MENU_OWNERS.write() {
        lock.remove(&player_index);
    }
}

/// Retrieves the owning WASM plugin name for the player's active menu, if any.
pub fn get_active_menu_owner(player_index: i32) -> Option<String> {
    ACTIVE_MENU_OWNERS
        .read()
        .ok()
        .and_then(|lock| lock.get(&player_index).cloned())
}

/// Clears all active menu owners (e.g. on map change / server deactivate).
pub fn clear_all_active_menu_owners() {
    if let Ok(mut lock) = ACTIVE_MENU_OWNERS.write() {
        lock.clear();
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
