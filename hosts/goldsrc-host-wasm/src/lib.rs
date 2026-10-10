#![allow(clippy::collapsible_if)]

//! WASM plugin host for GoldSrc.rs.
//!
//! Uses `wasmtime` (with `pulley32`) as the pure-Rust WASM runtime for maximum compatibility
//! with 32-bit HLDS. Implements the WASM Component Model via `wit-bindgen`.

/// Generated wasmtime bindings for the `goldsrc` WIT world.
pub mod bindings;
/// Crash diagnostics, symbol demangling, and reports.
pub mod crash;
/// Error taxonomy.
pub mod error;
/// Plugin lifecycle management and hot-reload.
pub mod manager;
/// Monomorphic U-cycle processing pipelines via stitch-rs.
pub mod pipeline;
/// Loaded plugin instance and metadata types.
pub mod plugin;

pub use crash::{PluginCrashReport, format_crash_report};
pub use error::{CommandError, HostError, LoadError};
pub use manager::{CommandRegistry, PauseAllOutcome, PauseOutcome, PluginInfo, PluginManager};
pub use pipeline::*;
pub use plugin::PluginStatus;

pub type PrintCallback = fn(&str);
pub type ShowMenuCallback = fn(i32, i32, i32, &str);
pub type StorageGetCallback = fn(&str, &str) -> Option<Vec<u8>>;
pub type StorageSetCallback = fn(&str, &str, &[u8]) -> bool;
pub type StorageDeleteCallback = fn(&str, &str) -> bool;
pub type StorageFetchAddCallback = fn(&str, &str, i64) -> i64;
pub type TranslateCallback = fn(&str, &str, &str, &str) -> String;
pub type FormatPlaceholdersCallback = fn(i32, &str) -> String;
pub type TimeCallback = fn() -> f32;
pub type VfsReadTextCallback = fn(&str, &str) -> Result<String, String>;
pub type VfsReadBytesCallback = fn(&str, &str) -> Result<Vec<u8>, String>;
pub type VfsListDirCallback = fn(&str, &str) -> Result<Vec<(String, bool, u64)>, String>;
pub type FeatureQueryCallback = fn(u64) -> bool;

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
static FORMAT_PLACEHOLDERS_CB: std::sync::RwLock<Option<FormatPlaceholdersCallback>> =
    std::sync::RwLock::new(None);
static TIME_CB: std::sync::RwLock<Option<TimeCallback>> = std::sync::RwLock::new(None);
pub(crate) static FEATURE_QUERY_CB: std::sync::RwLock<Option<FeatureQueryCallback>> =
    std::sync::RwLock::new(None);
pub(crate) static VFS_READ_TEXT_CB: std::sync::RwLock<Option<VfsReadTextCallback>> =
    std::sync::RwLock::new(None);
pub(crate) static VFS_READ_BYTES_CB: std::sync::RwLock<Option<VfsReadBytesCallback>> =
    std::sync::RwLock::new(None);
pub(crate) static VFS_LIST_DIR_CB: std::sync::RwLock<Option<VfsListDirCallback>> =
    std::sync::RwLock::new(None);

/// Set global callback for querying host game and runtime features.
pub fn set_feature_query_callback(f: FeatureQueryCallback) {
    if let Ok(mut lock) = FEATURE_QUERY_CB.write() {
        *lock = Some(f);
    }
}

/// Set global callbacks for host virtual filesystem (VFS) operations.
pub fn set_vfs_callbacks(
    read_text: VfsReadTextCallback,
    read_bytes: VfsReadBytesCallback,
    list_dir: VfsListDirCallback,
) {
    if let Ok(mut lock) = VFS_READ_TEXT_CB.write() {
        *lock = Some(read_text);
    }
    if let Ok(mut lock) = VFS_READ_BYTES_CB.write() {
        *lock = Some(read_bytes);
    }
    if let Ok(mut lock) = VFS_LIST_DIR_CB.write() {
        *lock = Some(list_dir);
    }
}

/// Set global callback for retrieving host uptime in seconds.
pub fn set_time_callback(f: TimeCallback) {
    if let Ok(mut lock) = TIME_CB.write() {
        *lock = Some(f);
    }
}

pub(crate) fn get_host_time() -> f32 {
    static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    if let Ok(lock) = TIME_CB.read()
        && let Some(cb) = *lock
    {
        cb()
    } else {
        START_TIME
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_secs_f32()
    }
}

/// Set global callback for formatting placeholders in host messages.
pub fn set_format_placeholders_callback(f: FormatPlaceholdersCallback) {
    if let Ok(mut lock) = FORMAT_PLACEHOLDERS_CB.write() {
        *lock = Some(f);
    }
}

pub(crate) fn format_message_placeholders(player_index: i32, message: &str) -> String {
    if let Ok(lock) = FORMAT_PLACEHOLDERS_CB.read()
        && let Some(cb) = *lock
    {
        cb(player_index, message)
    } else {
        message.to_string()
    }
}

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

static LOG_TOKEN_BUCKET: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(100);
static LOG_LAST_REFILL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Print log message via host callback (engine server_print and unified logger).
pub fn host_log(msg: &str) {
    // Basic token bucket rate limiting (max 100 log messages per second across WASM guests)
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let last = LOG_LAST_REFILL.load(std::sync::atomic::Ordering::Relaxed);
    if now_secs != last {
        LOG_LAST_REFILL.store(now_secs, std::sync::atomic::Ordering::Relaxed);
        LOG_TOKEN_BUCKET.store(100, std::sync::atomic::Ordering::Relaxed);
    }

    let is_error = msg.starts_with("[ERROR] ");
    if !is_error {
        let prev = LOG_TOKEN_BUCKET.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        if prev == 0 {
            // Out of tokens; silently throttle non-error spam to protect HLDS tickrate
            return;
        }
    }

    let max_len = if is_error { 16384 } else { 2048 };
    let bounded = if msg.len() > max_len {
        let mut end = max_len;
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
        log::Level::Error => {
            log::error!(target: goldsrc_api::consts::log_targets::PLUGIN, "{clean_msg}")
        }
        log::Level::Warn => {
            log::warn!(target: goldsrc_api::consts::log_targets::PLUGIN, "{clean_msg}")
        }
        log::Level::Debug => {
            log::debug!(target: goldsrc_api::consts::log_targets::PLUGIN, "{clean_msg}")
        }
        log::Level::Trace => {
            log::trace!(target: goldsrc_api::consts::log_targets::PLUGIN, "{clean_msg}")
        }
        log::Level::Info => {
            log::info!(target: goldsrc_api::consts::log_targets::PLUGIN, "{clean_msg}")
        }
    }
}
