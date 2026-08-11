//! Metamod backend implementation for GoldSrc.rs.

#![allow(static_mut_refs)]

mod entity;
mod meta_types;
mod vtable;

pub use entity::*;
pub use vtable::*;

use goldsrc_api::Engine;
use std::ffi::c_void;
use std::ffi::CString;

use meta_types::*;

static mut G_ENGFUNCS: Option<goldsrc_sys::enginefuncs_t> = None;
static mut G_GLOBALS: Option<goldsrc_sys::globalvars_t> = None;
static mut G_META_GLOBALS: Option<*mut meta_globals_t> = None;
static mut WASM_MANAGER: Option<goldsrc_wasm_host::PluginManager> = None;

/// Initialize WASM plugin subsystem
pub fn init_wasm_host() {
    goldsrc_wasm_host::set_print_callback(|msg| {
        backend().server_print(msg);
    });
    unsafe {
        let mut manager = goldsrc_wasm_host::PluginManager::new();

        let plugin_dirs = [
            "cstrike/addons/metamod-rs/plugins",
            "addons/metamod-rs/plugins",
        ];
        for dir in plugin_dirs {
            if std::path::Path::new(dir).exists() {
                let _ = manager.enable_hot_reload(dir);
                break;
            }
        }

        let config_dirs = [
            "cstrike/addons/metamod-rs/configs",
            "addons/metamod-rs/configs",
        ];
        let mut watched_config = false;
        for dir in config_dirs {
            if std::path::Path::new(dir).exists() {
                let _ = manager.enable_config_watcher(dir);
                watched_config = true;
                break;
            }
        }
        if !watched_config {
            let _ = manager.enable_config_watcher("cstrike/addons/metamod-rs/configs");
        }

        WASM_MANAGER = Some(manager);
    }
}

pub fn wasm_manager() -> Option<&'static mut goldsrc_wasm_host::PluginManager> {
    unsafe { WASM_MANAGER.as_mut() }
}

/// # Safety
/// Called once from `GiveFnptrsToDll`.
pub unsafe fn init_backend(
    engfuncs: *mut goldsrc_sys::enginefuncs_t,
    globals: *mut goldsrc_sys::globalvars_t,
) {
    unsafe {
        if !engfuncs.is_null() {
            G_ENGFUNCS = Some(*engfuncs);
        }
        if !globals.is_null() {
            G_GLOBALS = Some(*globals);
        }
    }
}

pub fn engfuncs() -> &'static goldsrc_sys::enginefuncs_t {
    unsafe { G_ENGFUNCS.as_ref().expect("Backend not initialized") }
}

pub fn globals() -> &'static goldsrc_sys::globalvars_t {
    unsafe { G_GLOBALS.as_ref().expect("Backend not initialized") }
}

pub fn meta_globals() -> &'static mut meta_globals_t {
    unsafe {
        G_META_GLOBALS
            .expect("Meta globals not initialized")
            .as_mut()
            .expect("Meta globals pointer is null")
    }
}

macro_rules! call_engfunc {
    ($func:expr) => {
        if let Some(f) = $func {
            f();
        }
    };
    ($func:expr, $($arg:expr),*) => {
        if let Some(f) = $func {
            f($($arg),*);
        }
    };
}

macro_rules! call_engfunc_ret {
    ($func:expr) => {
        if let Some(f) = $func {
            f()
        } else {
            Default::default()
        }
    };
    ($func:expr, $($arg:expr),*) => {
        if let Some(f) = $func {
            f($($arg),*)
        } else {
            Default::default()
        }
    };
}

pub struct MetamodBackend;

impl Default for MetamodBackend {
    fn default() -> Self {
        Self
    }
}

impl MetamodBackend {
    pub const fn new() -> Self {
        Self
    }
}

impl Engine for MetamodBackend {
    fn spawn_entity(&self, classname: &str) -> Option<goldsrc_api::Entity> {
        unsafe {
            let funcs = engfuncs();
            let edict = (funcs.pfnCreateEntity)?();
            if edict.is_null() {
                return None;
            }
            let cname = CString::new(classname).unwrap_or_default();
            call_engfunc!(funcs.pfnSetModel, edict, cname.as_ptr());
            let index = (funcs.pfnIndexOfEdict)?(edict);
            Some(goldsrc_api::Entity::from_raw(index, edict))
        }
    }

    fn get_player(&self, index: i32) -> Option<goldsrc_api::Player> {
        unsafe {
            let funcs = engfuncs();
            let edict = (funcs.pfnPEntityOfEntIndex)?(index);
            if edict.is_null() {
                return None;
            }
            Some(goldsrc_api::Player::from_raw(index, edict))
        }
    }

    fn server_print(&self, message: &str) {
        // Defer printing to StartFrame_Post to avoid engine instability during StartFrame.
        if let Ok(mut queue) = PRINT_QUEUE.lock() {
            queue.push(message.to_string());
        }
    }

    fn server_command(&self, command: &str) {
        unsafe {
            let cmd = CString::new(command).unwrap_or_default();
            call_engfunc!(engfuncs().pfnServerCommand, cmd.as_ptr());
        }
    }

    fn cvar_get_float(&self, name: &str) -> f32 {
        unsafe {
            let cname = CString::new(name).unwrap_or_default();
            call_engfunc_ret!(engfuncs().pfnCVarGetFloat, cname.as_ptr())
        }
    }

    fn cvar_set_float(&self, name: &str, value: f32) {
        unsafe {
            let cname = CString::new(name).unwrap_or_default();
            call_engfunc!(engfuncs().pfnCVarSetFloat, cname.as_ptr(), value);
        }
    }
}

use std::fs::OpenOptions;
use std::io::Write;

pub fn file_log(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("cstrike/addons/metamod-rs/debug.log")
    {
        let _ = writeln!(file, "{}", msg);
    }
}

static BACKEND: MetamodBackend = MetamodBackend::new();

pub fn backend() -> &'static MetamodBackend {
    &BACKEND
}

// ============================================================================
// Hook Tables
// ============================================================================

/// Function tables that we provide to Metamod.
/// Metamod calls these to get our hook functions.
/// # Safety
/// Called by Metamod to get entity API hooks. Pointers must be valid.
/// Note: This is only called when the plugin is loaded as a game DLL plugin.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI2(
    dll_table: *mut goldsrc_sys::DLL_FUNCTIONS,
    interface_version: *mut i32,
) -> i32 {
    if dll_table.is_null() || interface_version.is_null() {
        return 0;
    }
    if *interface_version != 140 {
        *interface_version = 140;
        return 0;
    }

    let table = &mut *dll_table;
    table.pfnSpawn = Some(hook_spawn);
    table.pfnClientConnect = Some(hook_client_connect);
    table.pfnClientDisconnect = Some(hook_client_disconnect);
    table.pfnClientCommand = Some(hook_client_command);
    table.pfnStartFrame = Some(hook_start_frame);
    1
}

/// # Safety
/// Called by Metamod to get post-entity API hooks.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI2_Post(
    dll_table: *mut goldsrc_sys::DLL_FUNCTIONS,
    interface_version: *mut i32,
) -> i32 {
    if dll_table.is_null() || interface_version.is_null() {
        return 0;
    }
    if *interface_version != 140 {
        *interface_version = 140;
        return 0;
    }

    let table = &mut *dll_table;
    table.pfnSpawn = Some(hook_spawn_post);
    table.pfnClientConnect = Some(hook_client_connect_post);
    table.pfnClientDisconnect = Some(hook_client_disconnect_post);
    table.pfnStartFrame = Some(hook_start_frame_post);

    1
}

/// # Safety
/// Called by Metamod to get entity API hooks (old interface).
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI(
    dll_table: *mut goldsrc_sys::DLL_FUNCTIONS,
    interface_version: i32,
) -> i32 {
    if dll_table.is_null() {
        return 0;
    }
    if interface_version != 140 {
        return 0;
    }
    backend().server_print("[GoldSrc.rs] GetEntityAPI called.\n");
    1
}

/// # Safety
/// Called by Metamod to get post-entity API hooks (old interface).
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI_Post(
    _dll_table: *mut goldsrc_sys::DLL_FUNCTIONS,
    _interface_version: i32,
) -> i32 {
    0
}

/// # Safety
/// Called by Metamod to get new DLL functions.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetNewDLLFunctions(
    _new_table: *mut c_void,
    _interface_version: *mut i32,
) -> i32 {
    0
}

/// # Safety
/// Called by Metamod to get post-new DLL functions.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetNewDLLFunctions_Post(
    _new_table: *mut c_void,
    _interface_version: *mut i32,
) -> i32 {
    0
}

/// # Safety
/// Called by Metamod to get engine functions. Pointers must be valid.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetEngineFunctions(
    engfuncs: *mut goldsrc_sys::enginefuncs_t,
    _interface_version: *mut i32,
) -> i32 {
    if engfuncs.is_null() {
        return 0;
    }
    backend().server_print("[GoldSrc.rs] GetEngineFunctions called.\n");
    1
}

static PRINT_QUEUE: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// # Safety
/// Called by Metamod to get post-engine functions.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetEngineFunctions_Post(
    _engfuncs: *mut goldsrc_sys::enginefuncs_t,
    _interface_version: *mut i32,
) -> i32 {
    0
}

/// Helper for Metamod ALERT logging.
pub fn alert(level: goldsrc_sys::ALERT_TYPE, message: &str) {
    // SAFETY: engfuncs() pointer valid after initialization.
    unsafe {
        let msg = CString::new(message).unwrap_or_default();
        call_engfunc!(engfuncs().pfnAlertMessage, level, msg.as_ptr());
    }
}

/// Hook for DispatchSpawn - called when an entity spawns.
///
/// # Safety
/// `edict` must be a valid pointer to an edict_t.
#[allow(dead_code)]
unsafe extern "C" fn hook_spawn(_edict: *mut goldsrc_sys::edict_t) -> i32 {
    0
}

/// Post-hook for DispatchSpawn.
#[allow(dead_code)]
unsafe extern "C" fn hook_spawn_post(_edict: *mut goldsrc_sys::edict_t) -> i32 {
    0
}

/// Post-hook for StartFrame.
unsafe extern "C" fn hook_start_frame_post() {
    let message = {
        let mut queue = match PRINT_QUEUE.lock() {
            Ok(q) => q,
            Err(e) => e.into_inner(),
        };
        if queue.is_empty() {
            return;
        }
        queue.remove(0)
    };

    // =========================================================================================
    // WARNING! VERY IMPORTANT COSTELL AND HISTORICAL REFERENCE FOR DESCENDANTS
    // =========================================================================================
    // If you're reading this code, you're probably wondering: "What the fuck is going on here?"
    //
    // Some genius (may their code compile with errors!) decided to update the console logger
    // in ReHLDS / HLDS to use the C++ `fmtlib` library (`std::format`),
    // BUT FORGOT THAT STRINGS FROM PLUGINS CANNOT BE PASSED DIRECTLY AS A FORMAT!
    //
    // The result is that if the plugin outputs JSON or any string with curly braces `{` or `}`,
    // `fmtlib` considers them format specifiers, throws an UNCAUGHT exception
    // `std::format_error("invalid format string")` and THE ENTIRE SERVER CRASHES TO HELL!
    //
    // I spent half me life blaming StartFrame, mutexes, poor Wasmi, memory alignment and Windows,
    // but the fault lay with a damn genius who didn't know how to use `fmt::print("{}")`!
    //
    // Hence:
    // 1. PRINT_QUEUE saves from CRT buffer overflow when spamming logs for 1 frame.
    // 2. All `{` and `}` are ESCAPED as `{{` and `}}` — so `fmt` turns them back into braces without a crash.
    // 3. `%` is escaped as `%%`.
    // =========================================================================================

    let safe_msg = message
        .replace("%", "%%")
        .replace("{", "{{")
        .replace("}", "}}")
        .replace("\r", "")
        .replace("\n", " ");
        
    let mut end = safe_msg.len().min(400);
    while end > 0 && !safe_msg.is_char_boundary(end) {
        end -= 1;
    }
    
    let final_msg = format!("{}\n", safe_msg[..end].trim_end());
    if let Ok(msg) = CString::new(final_msg) {
        call_engfunc!(engfuncs().pfnServerPrint, msg.as_ptr());
    }
}

/// Hook for ClientConnect - called when a player connects.
///
/// # Safety
/// Pointers must be valid C strings.
#[allow(dead_code)]
unsafe extern "C" fn hook_client_connect(
    _entity: *mut goldsrc_sys::edict_t,
    _name: *const std::os::raw::c_char,
    _address: *const std::os::raw::c_char,
    _reject_reason: *mut std::os::raw::c_char,
) -> goldsrc_sys::qboolean {
    0
}

/// Post-hook for ClientConnect.
#[allow(dead_code)]
unsafe extern "C" fn hook_client_connect_post(
    _entity: *mut goldsrc_sys::edict_t,
    _name: *const std::os::raw::c_char,
    _address: *const std::os::raw::c_char,
    _reject_reason: *mut std::os::raw::c_char,
) -> goldsrc_sys::qboolean {
    0
}

/// Hook for ClientDisconnect - called when a player disconnects.
///
/// # Safety
/// `entity` must be valid player edict pointer or null.
#[allow(dead_code)]
unsafe extern "C" fn hook_client_disconnect(_entity: *mut goldsrc_sys::edict_t) {}

/// Post-hook for ClientDisconnect.
#[allow(dead_code)]
unsafe extern "C" fn hook_client_disconnect_post(_entity: *mut goldsrc_sys::edict_t) {}

/// Hook for ClientCommand - called when a player issues a command.
///
/// # Safety
/// `_entity` must be a valid pointer to an edict_t.
#[allow(dead_code)]
unsafe extern "C" fn hook_client_command(_entity: *mut goldsrc_sys::edict_t) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let argc = call_engfunc_ret!(engfuncs().pfnCmd_Argc);
        if argc == 0 {
            return;
        }
        let cmd_ptr = call_engfunc_ret!(engfuncs().pfnCmd_Argv, 0);
        let args_ptr = call_engfunc_ret!(engfuncs().pfnCmd_Args);
        if !cmd_ptr.is_null() {
            if let Ok(cmd_str) = std::ffi::CStr::from_ptr(cmd_ptr).to_str() {
                let args_str = if !args_ptr.is_null() {
                    std::ffi::CStr::from_ptr(args_ptr)
                        .to_str()
                        .unwrap_or_default()
                } else {
                    ""
                };
                if let Some(manager) = wasm_manager() {
                    manager.dispatch_command(cmd_str, args_str);
                }
            }
        }
    }));
}

/// Hook for StartFrame - called every server frame.
unsafe extern "C" fn hook_start_frame() {
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Some(manager) = wasm_manager() {
            manager.on_server_frame();
        }
    }));
    if let Err(err) = res {
        let err_msg = if let Some(s) = err.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = err.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        backend().server_print(&format!(
            "[GoldSrc.rs PANIC] Caught panic in StartFrame: {}\n",
            err_msg
        ));
    }
}

use lexopt::Arg;

/// Server command handler for `meta-rs` and `mrs` console commands.
unsafe extern "C" fn handle_mrs_command() {
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let argc = call_engfunc_ret!(engfuncs().pfnCmd_Argc);
        if argc == 0 {
            return;
        }

        let mut raw_args = Vec::new();
        for i in 0..argc {
            let arg_ptr = call_engfunc_ret!(engfuncs().pfnCmd_Argv, i);
            if !arg_ptr.is_null() {
                if let Ok(cstr) = std::ffi::CStr::from_ptr(arg_ptr).to_str() {
                    raw_args.push(std::ffi::OsString::from(cstr));
                }
            }
        }

        dispatch_mrs_command(raw_args);
    }));
    if let Err(err) = res {
        let err_msg = if let Some(s) = err.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = err.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        backend().server_print(&format!(
            "[GoldSrc.rs PANIC] Caught panic in CLI Command: {}\n",
            err_msg
        ));
    }
}

fn print_mrs_help() {
    backend().server_print("--- GoldSrc.rs (meta-rs / mrs) Management CLI ---\n");
    backend().server_print("Usage: mrs <COMMAND> [OPTIONS] [TARGET]\n\n");
    backend().server_print("Commands:\n");
    backend().server_print("  [ Plugin Lifecycle ]\n");
    backend()
        .server_print("    load <file>             Load a WASM plugin from plugins/ directory\n");
    backend().server_print(
        "    unload <target>         Gracefully unload plugin(s) (-a, --all supported)\n",
    );
    backend()
        .server_print("    reload <target>         Reload plugin(s) (-a, --all supported)\n\n");
    backend().server_print("  [ Execution Control ]\n");
    backend().server_print(
        "    pause <target>          Suspend plugin execution (-a, --all supported)\n",
    );
    backend().server_print(
        "    unpause <target>        Resume plugin execution (-a, --all supported)\n\n",
    );
    backend().server_print("  [ Inspection & Debugging ]\n");
    backend().server_print("    list [OPTIONS]          List loaded plugins. Options:\n");
    backend().server_print(
        "                              -p, --page <N>    Show specific page (default: 1)\n",
    );
    backend().server_print(
        "                              -s, --size <N>    Set page size (default: 5)\n",
    );
    backend().server_print(
        "                              -a, --all         Show all plugins (ignore pagination)\n",
    );
    backend()
        .server_print("                              --paused          Show only paused plugins\n");
    backend().server_print("    info <target>           Show detailed metadata and exports\n\n");
    backend().server_print("  [ System ]\n");
    backend().server_print(
        "    status                  Show host runtime stats (RAM, watchers, count)\n",
    );
    backend().server_print("    version                 Show host version info\n\n");
    backend().server_print("Options:\n");
    backend().server_print("  -h, --help                Print this help message\n");
    backend().server_print(
        "  -a, --all                 Target all loaded plugins (for unload/reload/pause)\n",
    );
}

fn dispatch_mrs_command(raw_args: Vec<std::ffi::OsString>) {
    const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    const GIT_HASH: &str = env!("GIT_HASH");
    const BUILD_TARGET: &str = env!("BUILD_TARGET");
    let manager = match wasm_manager() {
        Some(m) => m,
        None => {
            backend().server_print("[GoldSrc.rs] Error: WASM Host not initialized.\n");
            return;
        }
    };

    let mut parser = lexopt::Parser::from_args(raw_args);
    // Skip binary name ("mrs" or "meta-rs")
    let _ = parser.next();

    let command = match parser.next() {
        Ok(Some(Arg::Value(val))) => val.to_string_lossy().to_lowercase(),
        Ok(Some(Arg::Short('h') | Arg::Long("help"))) => {
            print_mrs_help();
            return;
        }
        _ => {
            print_mrs_help();
            return;
        }
    };

    match command.as_str() {
        "list" => {
            let mut page: usize = 1;
            let mut size: Option<usize> = None;
            let mut only_paused = false;
            let mut all = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('p') | Arg::Long("page") => {
                        if let Ok(val) = parser.value() {
                            page = val.to_string_lossy().parse().unwrap_or(1);
                        }
                    }
                    Arg::Short('s') | Arg::Long("size") => {
                        if let Ok(val) = parser.value() {
                            size = Some(val.to_string_lossy().parse().unwrap_or(5));
                        }
                    }
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Long("paused") => only_paused = true,
                    _ => {}
                }
            }

            let mut plugins = manager.get_plugins_info();
            if only_paused {
                plugins.retain(|p| p.is_paused);
            }

            let total_plugins = plugins.len();
            let page_size = if all {
                total_plugins.max(1)
            } else {
                size.unwrap_or(5)
            };
            let total_pages = (total_plugins + page_size - 1) / page_size.max(1);
            let page_idx = page.saturating_sub(1);

            backend().server_print(&format!(
                "[GoldSrc.rs] WASM plugins ({}) [Page {}/{}]:\n",
                total_plugins,
                if total_pages == 0 { 1 } else { page },
                if total_pages == 0 { 1 } else { total_pages }
            ));

            if plugins.is_empty() {
                backend().server_print("  (No plugins found)\n");
                return;
            }

            let start = (page_idx * page_size).min(total_plugins);
            let end = (start + page_size).min(total_plugins);

            for p in &plugins[start..end] {
                let status = if p.is_paused { "PAUSED" } else { "RUNNING" };
                let mut exports = Vec::new();
                if p.has_on_load {
                    exports.push("on_load");
                }
                if p.has_on_unload {
                    exports.push("on_unload");
                }
                if p.has_on_frame {
                    exports.push("on_frame");
                }
                let exports_str = if exports.is_empty() {
                    "none".to_string()
                } else {
                    exports.join(", ")
                };
                backend().server_print(&format!(
                    "  [#{}] {:<20} | Status: {:<7} | Exports: {}\n",
                    p.index, p.name, status, exports_str
                ));
            }
        }
        "load" => {
            let mut targets = Vec::new();
            while let Ok(Some(arg)) = parser.next() {
                if let Arg::Value(val) = arg {
                    targets.push(val.to_string_lossy().into_owned());
                }
            }
            if targets.is_empty() {
                backend().server_print("Usage: mrs load <file1> [file2...]\n");
                return;
            }
            for t in targets {
                match manager.load_plugin_by_name(&t) {
                    Ok(msg) => backend().server_print(&msg),
                    Err(err) => backend().server_print(&err),
                }
            }
        }
        "unload" => {
            let mut targets = Vec::new();
            let mut all = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Value(val) => targets.push(val.to_string_lossy().into_owned()),
                    _ => {}
                }
            }
            if all {
                let msg = manager.unload_all_plugins();
                backend().server_print(&msg);
            } else if !targets.is_empty() {
                for t in targets {
                    match manager.unload_plugin_by_query(&t) {
                        Ok(msg) => backend().server_print(&msg),
                        Err(err) => backend().server_print(&err),
                    }
                }
            } else {
                backend()
                    .server_print("Usage: mrs unload <name|index...> or mrs unload -a/--all\n");
            }
        }
        "reload" => {
            let mut targets = Vec::new();
            let mut all = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Value(val) => targets.push(val.to_string_lossy().into_owned()),
                    _ => {}
                }
            }
            if all {
                let msg = manager.reload_all_plugins();
                backend().server_print(&msg);
            } else if !targets.is_empty() {
                for t in targets {
                    match manager.reload_plugin_by_query(&t) {
                        Ok(msg) => backend().server_print(&msg),
                        Err(err) => backend().server_print(&err),
                    }
                }
            } else {
                backend()
                    .server_print("Usage: mrs reload <name|index...> or mrs reload -a/--all\n");
            }
        }
        "pause" => {
            let mut targets = Vec::new();
            let mut all = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Value(val) => targets.push(val.to_string_lossy().into_owned()),
                    _ => {}
                }
            }
            if all {
                let msg = manager.pause_all_plugins(true);
                backend().server_print(&msg);
            } else if !targets.is_empty() {
                for t in targets {
                    match manager.pause_plugin(&t, true) {
                        Ok(msg) => backend().server_print(&msg),
                        Err(err) => backend().server_print(&err),
                    }
                }
            } else {
                backend().server_print("Usage: mrs pause <name|index...> or mrs pause -a/--all\n");
            }
        }
        "unpause" => {
            let mut targets = Vec::new();
            let mut all = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Value(val) => targets.push(val.to_string_lossy().into_owned()),
                    _ => {}
                }
            }
            if all {
                let msg = manager.pause_all_plugins(false);
                backend().server_print(&msg);
            } else if !targets.is_empty() {
                for t in targets {
                    match manager.pause_plugin(&t, false) {
                        Ok(msg) => backend().server_print(&msg),
                        Err(err) => backend().server_print(&err),
                    }
                }
            } else {
                backend()
                    .server_print("Usage: mrs unpause <name|index...> or mrs unpause -a/--all\n");
            }
        }
        "info" => {
            let mut targets = Vec::new();
            while let Ok(Some(arg)) = parser.next() {
                if let Arg::Value(val) = arg {
                    targets.push(val.to_string_lossy().into_owned());
                }
            }
            if targets.is_empty() {
                backend().server_print("Usage: mrs info <name|index...>\n");
                return;
            }
            for t in targets {
                if let Some(idx) = manager.find_plugin_index(&t) {
                    let info = &manager.get_plugins_info()[idx];
                    let clean_path = info.path.to_string_lossy().replace('\\', "/");
                    backend().server_print(&format!("--- Plugin Info: {} ---\n", info.name));
                    backend().server_print(&format!("  Index:        #{}\n", info.index));
                    backend().server_print(&format!("  Path:         {}\n", clean_path));
                    backend().server_print(&format!(
                        "  Status:       {}\n",
                        if info.is_paused { "Paused" } else { "Running" }
                    ));
                    if let Some(meta) = &info.metadata {
                        backend().server_print(&format!("  Meta Name:    {}\n", meta.name));
                        backend().server_print(&format!("  Version:      {}\n", meta.version));
                        let systems_str = if meta.systems.is_empty() {
                            "none".to_string()
                        } else {
                            meta.systems.join(", ")
                        };
                        backend().server_print(&format!("  Systems:      {}\n", systems_str));
                        let deps_str = if meta.dependencies.is_empty() {
                            "none".to_string()
                        } else {
                            meta.dependencies
                                .iter()
                                .map(|(k, v)| format!("{} ({})", k, v))
                                .collect::<Vec<_>>()
                                .join(", ")
                        };
                        backend().server_print(&format!("  Dependencies: {}\n", deps_str));
                    }
                    backend().server_print(&format!("  on_load:      {}\n", info.has_on_load));
                    backend().server_print(&format!("  on_unload:    {}\n", info.has_on_unload));
                    backend().server_print(&format!("  on_frame:     {}\n", info.has_on_frame));
                } else {
                    backend().server_print(&format!("[GoldSrc.rs] Plugin '{}' not found.\n", t));
                }
            }
        }
        "status" => {
            let (plugins_count, watchers_count) = manager.get_status_info();
            backend().server_print("--- GoldSrc.rs Host Engine Status ---\n");
            backend().server_print(&format!(
                "  Version:    v{} (git: {})\n",
                CARGO_PKG_VERSION, GIT_HASH
            ));
            backend().server_print(&format!("  Target:     {}\n", BUILD_TARGET));
            backend().server_print("  WASM Engine: wasmi (Pure Rust Interpreter)\n");
            backend().server_print(&format!("  Plugins:    {} loaded\n", plugins_count));
            backend().server_print(&format!(
                "  Watchers:   {} active directory watcher(s)\n",
                watchers_count
            ));
        }
        "version" => {
            backend().server_print(&format!(
                "[GoldSrc.rs] meta-rs v{} (git: {}, target: {})\n",
                CARGO_PKG_VERSION, GIT_HASH, BUILD_TARGET
            ));
        }
        _ => {
            print_mrs_help();
        }
    }
}

pub fn register_cli_commands() {
    let cmd_meta_rs = CString::new("meta-rs").unwrap();
    let cmd_mrs = CString::new("mrs").unwrap();
    unsafe {
        call_engfunc!(
            engfuncs().pfnAddServerCommand,
            cmd_meta_rs.as_ptr(),
            Some(handle_mrs_command)
        );
        call_engfunc!(
            engfuncs().pfnAddServerCommand,
            cmd_mrs.as_ptr(),
            Some(handle_mrs_command)
        );
    }
}

// ============================================================================
// Metamod Entry Points
// ============================================================================

/// # Safety
/// Called by the engine during DLL loading. Pointers must be valid.
#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn GiveFnptrsToDll(
    engfuncs: *mut goldsrc_sys::enginefuncs_t,
    globals: *mut goldsrc_sys::globalvars_t,
) {
    unsafe {
        init_backend(engfuncs, globals);
    }
    backend().server_print("[GoldSrc.rs] Engine functions received.\n");
}

/// # Safety
/// Called by Metamod during plugin loading. Pointers must be valid.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn Meta_Query(
    _ifvers: *const std::os::raw::c_char,
    plugin_info: *mut *const plugin_info_t,
    meta_util_functions: *mut mutil_funcs_t,
) -> std::os::raw::c_int {
    unsafe {
        if plugin_info.is_null() || meta_util_functions.is_null() {
            return 0;
        }
        *plugin_info = &PLUGIN_INFO;
        *meta_util_functions = get_meta_util_funcs();
    }
    backend().server_print("[GoldSrc.rs] Meta_Query called.\n");
    1
}

/// # Safety
/// Called by Metamod after Meta_Query. Pointers must be valid.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn Meta_Attach(
    _now: PLUG_LOADTIME,
    meta_functions: *mut meta_function_t,
    meta_globals: *mut meta_globals_t,
    _gamedll_funcs: *mut c_void,
) -> std::os::raw::c_int {
    unsafe {
        if meta_globals.is_null() || meta_functions.is_null() {
            return 0;
        }
        G_META_GLOBALS = Some(meta_globals);

        // Fill the META_FUNCTIONS table with our hook functions
        (*meta_functions).pfnGetEntityAPI = Some(GetEntityAPI);
        (*meta_functions).pfnGetEntityAPI_Post = Some(GetEntityAPI_Post);
        (*meta_functions).pfnGetEntityAPI2 = Some(GetEntityAPI2);
        (*meta_functions).pfnGetEntityAPI2_Post = Some(GetEntityAPI2_Post);
        (*meta_functions).pfnGetNewDLLFunctions = Some(GetNewDLLFunctions);
        (*meta_functions).pfnGetNewDLLFunctions_Post = Some(GetNewDLLFunctions_Post);
        (*meta_functions).pfnGetEngineFunctions = Some(GetEngineFunctions);
        (*meta_functions).pfnGetEngineFunctions_Post = Some(GetEngineFunctions_Post);
    }
    init_wasm_host();
    register_cli_commands();
    backend().server_print("[GoldSrc.rs] Meta_Attach called.\n");
    backend().server_print("[GoldSrc.rs] WASM Host Engine initialized.\n");
    backend()
        .server_print("[GoldSrc.rs] Host Management CLI registered (`meta-rs` / `goldsrc`).\n");
    backend().server_print("[GoldSrc.rs] Hello from Rust!\n");
    1
}

/// # Safety
/// Called by Metamod during plugin unloading.
#[no_mangle]
#[inline(never)]
pub extern "C" fn Meta_Detach(
    _now: PLUG_LOADTIME,
    _reason: PL_UNLOAD_REASON,
) -> std::os::raw::c_int {
    backend().server_print("[GoldSrc.rs] Meta_Detach called. Goodbye!\n");
    1
}

#[allow(non_upper_case_globals)]
static PLUGIN_INFO: plugin_info_t = plugin_info_t {
    ifvers: META_INTERFACE_VERSION.as_ptr() as *const i8,
    name: c"GoldSrc.rs Metamod Backend".as_ptr(),
    version: concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const i8,
    date: concat!(env!("GIT_HASH"), "\0").as_ptr() as *const i8,
    author: c"GoldSrc.rs Contributors".as_ptr(),
    url: c"https://github.com/ulquiorracode/GoldSrc.rs".as_ptr(),
    logtag: c"GOLDSRC.RS".as_ptr(),
    loadable: PLUG_LOADTIME::PT_ANYTIME,
    unloadable: PLUG_LOADTIME::PT_ANYTIME,
};

fn get_meta_util_funcs() -> mutil_funcs_t {
    mutil_funcs_t {
        pfnLogConsole: None,
        pfnLogMessage: None,
        pfnLogError: None,
        pfnLogDeveloper: None,
        _padding: [0; 12],
    }
}
