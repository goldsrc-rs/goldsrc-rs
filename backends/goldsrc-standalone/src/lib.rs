//! GoldSrc.rs Standalone Backend
//!
//! A proxy GameDLL that loads via `liblist.gam` → `gamedll` key.
//! Receives engine function pointers directly from `hlds.exe`/`hlds_linux`,
//! loads and proxies the real `mp.dll`/`cs.so`, and runs WASM plugins.
//!
//! # Server Setup
//! In `cstrike/liblist.gam`, change:
//! ```text
//! gamedll "dlls/mp.dll"
//! ```
//! to:
//! ```text
//! gamedll "addons/goldsrc/bin/goldsrc_standalone.dll"
//! ```

#![allow(static_mut_refs)]

mod engine_api;
mod proxy;

use goldsrc_api::Engine;
use goldsrc_sys::{enginefuncs_t, globalvars_t, DLL_FUNCTIONS};
use std::ffi::{CStr, CString};

// ============================================================================
// Backend (Engine trait implementation)
// ============================================================================

pub struct StandaloneBackend;

impl StandaloneBackend {
    pub const fn new() -> Self {
        Self
    }
}

static BACKEND: StandaloneBackend = StandaloneBackend::new();

pub fn backend() -> &'static StandaloneBackend {
    &BACKEND
}

impl Default for StandaloneBackend {
    fn default() -> Self {
        Self::new()
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
    ($func:expr, $($arg:expr),*) => {
        if let Some(f) = $func {
            f($($arg),*)
        } else {
            Default::default()
        }
    };
}

static PRINT_QUEUE: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

impl Engine for StandaloneBackend {
    fn spawn_entity(&self, classname: &str) -> Option<goldsrc_api::Entity> {
        unsafe {
            let funcs = engine_api::engfuncs();
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
            let funcs = engine_api::engfuncs();
            let edict = (funcs.pfnPEntityOfEntIndex)?(index);
            if edict.is_null() {
                return None;
            }
            Some(goldsrc_api::Player::from_raw(index, edict))
        }
    }

    fn server_print(&self, message: &str) {
        // Defer to StartFrame to avoid engine instability during initialization.
        if let Ok(mut queue) = PRINT_QUEUE.lock() {
            queue.push(message.to_string());
        }
    }

    fn server_command(&self, command: &str) {
        unsafe {
            let cmd = CString::new(command).unwrap_or_default();
            call_engfunc!(engine_api::engfuncs().pfnServerCommand, cmd.as_ptr());
        }
    }

    fn cvar_get_float(&self, name: &str) -> f32 {
        unsafe {
            let cname = CString::new(name).unwrap_or_default();
            call_engfunc_ret!(engine_api::engfuncs().pfnCVarGetFloat, cname.as_ptr())
        }
    }

    fn cvar_set_float(&self, name: &str, value: f32) {
        unsafe {
            let cname = CString::new(name).unwrap_or_default();
            call_engfunc!(
                engine_api::engfuncs().pfnCVarSetFloat,
                cname.as_ptr(),
                value
            );
        }
    }
}

// ============================================================================
// WASM Host initialization
// ============================================================================

static mut WASM_MANAGER: Option<goldsrc_wasm_host::PluginManager> = None;

fn init_wasm_host() {
    goldsrc_wasm_host::set_print_callback(|msg| {
        backend().server_print(msg);
    });

    unsafe {
        let mut manager = match goldsrc_wasm_host::PluginManager::new() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[GoldSrc.rs Standalone] Failed to initialize PluginManager: {e}");
                return;
            }
        };

        use goldsrc_sys::{paths::PathResolver, GoldSrcConfig};

        let sys_config = GoldSrcConfig::load_or_create();
        let plugin_dir = std::path::PathBuf::from(&sys_config.core.plugins_dir);
        let config_dir = std::path::PathBuf::from(&sys_config.core.configs_dir);

        backend().server_print(&format!(
            "[GoldSrc.rs Standalone] Config loaded from: {:?}\n",
            PathResolver::main_config_path()
        ));
        backend().server_print(&format!(
            "[GoldSrc.rs Standalone] Plugin dir: {:?}\n",
            plugin_dir
        ));

        if sys_config.wasm.hot_reload {
            let _ = manager.enable_hot_reload(&plugin_dir);
        }
        if sys_config.wasm.config_watcher {
            let _ = manager.enable_config_watcher(&config_dir);
        }

        if let Ok(entries) = std::fs::read_dir(&plugin_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                    match manager.load_plugin(&path) {
                        Ok(_) => backend().server_print(&format!(
                            "[GoldSrc.rs Standalone] Loaded plugin: {:?}\n",
                            path.file_name().unwrap_or_default()
                        )),
                        Err(e) => backend().server_print(&format!(
                            "[GoldSrc.rs Standalone] Failed to load {:?}: {e}\n",
                            path.file_name().unwrap_or_default()
                        )),
                    }
                }
            }
        }

        WASM_MANAGER = Some(manager);
    }
}

fn wasm_manager() -> Option<&'static mut goldsrc_wasm_host::PluginManager> {
    unsafe { WASM_MANAGER.as_mut() }
}

// ============================================================================
// Hook implementations
// ============================================================================

unsafe extern "C" fn hook_start_frame() {
    // Forward to real game DLL first.
    proxy::forward_start_frame();
}

#[allow(dead_code)]
unsafe extern "C" fn hook_start_frame_post() {
    // Drain the deferred print queue.
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
    unsafe {
        let funcs = engine_api::engfuncs();
        if let Some(f) = funcs.pfnServerPrint {
            if let Ok(cstr) = CString::new(message.as_str()) {
                f(cstr.as_ptr());
            }
        }
    }
}

unsafe extern "C" fn hook_client_connect(
    edict: *mut goldsrc_sys::edict_t,
    name: *const std::os::raw::c_char,
    address: *const std::os::raw::c_char,
    reject_reason: *mut std::os::raw::c_char,
) -> i32 {
    let result = proxy::forward_client_connect(edict, name, address, reject_reason);
    if let Some(manager) = wasm_manager() {
        manager.call_on_event("client_connect", &[]);
    }
    result
}

unsafe extern "C" fn hook_client_disconnect(edict: *mut goldsrc_sys::edict_t) {
    proxy::forward_client_disconnect(edict);
    if let Some(manager) = wasm_manager() {
        manager.call_on_event("client_disconnect", &[]);
    }
}

unsafe extern "C" fn hook_client_command(edict: *mut goldsrc_sys::edict_t) {
    proxy::forward_client_command(edict);
    if let Some(manager) = wasm_manager() {
        unsafe {
            let funcs = engine_api::engfuncs();
            let cmd_ptr = funcs
                .pfnCmd_Args
                .and_then(|f| Some(f()))
                .unwrap_or(std::ptr::null());
            let cmd_str = if !cmd_ptr.is_null() {
                CStr::from_ptr(cmd_ptr).to_string_lossy().into_owned()
            } else {
                String::new()
            };
            let name_ptr = funcs
                .pfnCmd_Argv
                .and_then(|f| Some(f(0)))
                .unwrap_or(std::ptr::null());
            let name_str = if !name_ptr.is_null() {
                CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
            } else {
                String::new()
            };
            manager.dispatch_command(&name_str, &cmd_str);
        }
    }
}

unsafe extern "C" fn hook_spawn(edict: *mut goldsrc_sys::edict_t) -> i32 {
    proxy::forward_spawn(edict)
}

// ============================================================================
// GameDLL Entry Points (loaded via liblist.gam `gamedll` key)
// ============================================================================

/// Called by the engine immediately after loading the DLL.
/// Provides engine function pointers and global variables.
///
/// # Safety
/// Pointers are provided by `hlds.exe` / `hlds_linux` and are always valid at this point.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GiveFnptrsToDll(engfuncs: *mut enginefuncs_t, globals: *mut globalvars_t) {
    unsafe {
        // 1. Initialize our engine API layer.
        engine_api::init(engfuncs, globals);

        // 2. Load the real game DLL and forward engine funcs to it.
        let _loaded = proxy::load(engfuncs, globals);
    }

    backend().server_print(&format!(
        "[GoldSrc.rs Standalone] GiveFnptrsToDll received. Engine tier: {}\n",
        engine_api::tier_name()
    ));

    // 3. Initialize WASM host.
    init_wasm_host();

    backend().server_print("[GoldSrc.rs Standalone] WASM Host initialized.\n");
    backend().server_print("[GoldSrc.rs Standalone] Hello from Rust! (Standalone Mode)\n");
}

/// Called by the engine to retrieve our `DLL_FUNCTIONS` hook table.
///
/// # Safety
/// `dll_table` is a valid pointer provided by the engine.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI2(
    dll_table: *mut DLL_FUNCTIONS,
    interface_version: *mut i32,
) -> i32 {
    if dll_table.is_null() || interface_version.is_null() {
        return 0;
    }
    unsafe {
        if *interface_version != 140 {
            *interface_version = 140;
            return 0;
        }

        // 1. Populate the table with all function pointers from the real GameDLL (mp.dll / cs.so).
        proxy::populate_dll_table(dll_table);

        // 2. Wrap our hooks over the table callbacks.
        let table = &mut *dll_table;
        table.pfnSpawn = Some(hook_spawn);
        table.pfnClientConnect = Some(hook_client_connect);
        table.pfnClientDisconnect = Some(hook_client_disconnect);
        table.pfnClientCommand = Some(hook_client_command);
        table.pfnStartFrame = Some(hook_start_frame);
    }
    1
}

/// Old-style entity API (required for compatibility with some engine versions).
///
/// # Safety
/// `dll_table` must be a valid pointer.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI(
    dll_table: *mut DLL_FUNCTIONS,
    interface_version: i32,
) -> i32 {
    if dll_table.is_null() || interface_version != 140 {
        return 0;
    }
    let mut ver = interface_version;
    GetEntityAPI2(dll_table, &mut ver)
}

/// Called by engine for NEW_DLL_FUNCTIONS interface (ReGameDLL, Sven Co-op, HLSDK 2.x).
///
/// # Safety
/// Pointers must be valid.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetNewDLLFunctions(
    new_dll_table: *mut std::ffi::c_void,
    interface_version: *mut i32,
) -> i32 {
    if new_dll_table.is_null() || interface_version.is_null() {
        return 0;
    }
    if *interface_version != 1 {
        *interface_version = 1;
        return 0;
    }
    if proxy::populate_new_dll_table(new_dll_table) {
        1
    } else {
        0
    }
}
