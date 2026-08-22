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

mod commands;
mod engine_api;
mod entities;
mod proxy;

use goldsrc::backend::EngineBackend;
use goldsrc::log;
use goldsrc_sys::ffi::catch_ffi_panic;
use goldsrc_sys::{DLL_FUNCTIONS, enginefuncs_t, globalvars_t};
use std::ffi::{CStr, CString};

// ============================================================================
// Backend (Engine trait implementation)
// ============================================================================

/// Standalone backend: the shared `EngineBackend` fed by this crate's engfunc
/// accessor and print queue. The backend is a thin adapter.
pub type StandaloneBackend = EngineBackend;

static PRINT_QUEUE: goldsrc::backend::PrintQueue = goldsrc::backend::PrintQueue::new();

static BACKEND: StandaloneBackend = EngineBackend::new(engine_api::engfuncs, &PRINT_QUEUE);

pub fn backend() -> &'static StandaloneBackend {
    &BACKEND
}

pub use goldsrc::call_engfunc;
pub use goldsrc::call_engfunc_ret;

// ============================================================================
// WASM Host initialization
// ============================================================================

fn init_wasm_host() {
    let engine: std::sync::Arc<dyn goldsrc_api::Engine> = std::sync::Arc::new(*backend());
    if let Err(e) = goldsrc::host::HostRuntime::init(
        goldsrc_api::consts::BackendType::Standalone,
        |msg| {
            backend().server_print(msg);
        },
        engine,
    ) {
        log::error!(target: "core", "{e}");
    }
}

// ============================================================================
// Hook implementations
// ============================================================================

unsafe extern "C" fn hook_game_init() {
    catch_ffi_panic("hook_game_init", (), || {
        // 1. Forward GameDLLInit to real GameDLL
        proxy::forward_game_init();
        // 2. Initialize WASM host and plugins
        init_wasm_host();
        // 3. Register CLI commands after engine command system is initialized
        commands::register_cli_commands();
        log::info!(target: "core", "hook_game_init: WASM host & commands initialized successfully");
    });
}

unsafe extern "C" fn hook_start_frame() {
    // SAFETY: forward_start_frame does not unwind; catch_unwind guards the ABI boundary.
    catch_ffi_panic("hook_start_frame", (), || {
        proxy::forward_start_frame();
        goldsrc::hooks::on_server_frame();
        drain_print_queue();
    });
}

/// Drain deferred server prints with fmtlib-safe escaping.
///
/// ReHLDS routes `ServerPrint` output through fmtlib: `%` and `{}` from plugin
/// text would throw and crash the server, so they are escaped before printing.
fn drain_print_queue() {
    for message in PRINT_QUEUE.drain() {
        let funcs = engine_api::engfuncs();
        if let Some(f) = funcs.pfnServerPrint
            && let Ok(cstr) = CString::new(message)
        {
            unsafe { f(cstr.as_ptr()) };
        }
    }
}

unsafe extern "C" fn hook_client_connect(
    edict: *mut goldsrc_sys::edict_t,
    name: *const std::os::raw::c_char,
    address: *const std::os::raw::c_char,
    reject_reason: *mut std::os::raw::c_char,
) -> goldsrc_sys::qboolean {
    catch_ffi_panic("hook_client_connect", 1 as goldsrc_sys::qboolean, || {
        let result = proxy::forward_client_connect(edict, name, address, reject_reason);
        if result != 0 {
            let funcs = engine_api::engfuncs();
            let index = funcs
                .pfnIndexOfEdict
                .map(|f| unsafe { f(edict) })
                .unwrap_or(0);
            goldsrc::hooks::emit_player_event("client_connect", index);
        }
        result
    })
}

unsafe extern "C" fn hook_client_disconnect(edict: *mut goldsrc_sys::edict_t) {
    catch_ffi_panic("hook_client_disconnect", (), || {
        let funcs = engine_api::engfuncs();
        let index = funcs
            .pfnIndexOfEdict
            .map(|f| unsafe { f(edict) })
            .unwrap_or(0);
        proxy::forward_client_disconnect(edict);
        goldsrc::hooks::emit_player_event("client_disconnect", index);
    });
}

unsafe extern "C" fn hook_client_command(edict: *mut goldsrc_sys::edict_t) {
    catch_ffi_panic("hook_client_command", (), || {
        let index = if !edict.is_null() {
            unsafe {
                crate::engine_api::engfuncs()
                    .pfnIndexOfEdict
                    .map(|f| f(edict))
                    .unwrap_or(0)
            }
        } else {
            0
        };

        let cmd_str;
        let name_str;
        {
            let funcs = engine_api::engfuncs();
            let cmd_ptr = unsafe { funcs.pfnCmd_Args.map(|f| f()).unwrap_or(std::ptr::null()) };
            cmd_str = if !cmd_ptr.is_null() {
                unsafe { CStr::from_ptr(cmd_ptr) }
                    .to_string_lossy()
                    .into_owned()
            } else {
                String::new()
            };
            let name_ptr = unsafe { funcs.pfnCmd_Argv.map(|f| f(0)).unwrap_or(std::ptr::null()) };
            name_str = if !name_ptr.is_null() {
                unsafe { CStr::from_ptr(name_ptr) }
                    .to_string_lossy()
                    .into_owned()
            } else {
                String::new()
            };
        }

        let handled = goldsrc::hooks::dispatch_client_command(index, &name_str, &cmd_str);
        if !handled {
            proxy::forward_client_command(edict);
        }
    });
}

unsafe extern "C" fn hook_spawn(edict: *mut goldsrc_sys::edict_t) -> i32 {
    catch_ffi_panic("hook_spawn", 0, || {
        crate::backend().precache_pending_resources();
        proxy::forward_spawn(edict)
    })
}

// ============================================================================
// GameDLL Entry Points (loaded via liblist.gam `gamedll` key)
// ============================================================================

/// Called by the engine immediately after loading the DLL.
/// Provides engine function pointers and global variables.
///
/// # Safety
/// Pointers are provided by `hlds.exe` / `hlds_linux` and are always valid at this point.
/// Any Rust panic is caught — an unhandled panic here would crash HLDS.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "system" fn GiveFnptrsToDll(
    engfuncs: *mut enginefuncs_t,
    globals: *mut globalvars_t,
) {
    // SAFETY: engfuncs and globals are engine-provided; valid for the server lifetime.
    catch_ffi_panic("GiveFnptrsToDll", (), || {
        unsafe {
            // 1. Initialize our engine API layer.
            engine_api::init(engfuncs, globals);
            // 2. Forward engine funcs to the real game DLL.
            proxy::forward_give_fnptrs_to_dll(engfuncs, globals);
        }
    });
}

/// Called by the engine to retrieve our `DLL_FUNCTIONS` hook table.
///
/// # Safety
/// `dll_table` is a valid pointer provided by the engine.
/// Any Rust panic is caught — the C caller receives 0 instead of UB.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI2(
    dll_table: *mut DLL_FUNCTIONS,
    interface_version: *mut i32,
) -> i32 {
    // SAFETY: dll_table and interface_version are engine-provided; valid at call time.
    catch_ffi_panic("GetEntityAPI2", 0, || {
        if dll_table.is_null() {
            return 0;
        }
        unsafe {
            if !interface_version.is_null() {
                *interface_version = 140;
            }
            // 1. Populate with real GameDLL (mp.dll / cs.so) callbacks.
            proxy::populate_dll_table(dll_table);

            // 2. Overlay our hooks.
            let table = &mut *dll_table;
            table.pfnGameInit = Some(hook_game_init);
            table.pfnSpawn = Some(hook_spawn);
            table.pfnClientConnect = Some(hook_client_connect);
            table.pfnClientDisconnect = Some(hook_client_disconnect);
            table.pfnClientCommand = Some(hook_client_command);
            table.pfnStartFrame = Some(hook_start_frame);
        }
        1
    })
}

/// Old-style entity API (required for compatibility with some engine versions).
///
/// # Safety
/// `dll_table` must be a valid pointer.
/// Any Rust panic is caught — the C caller receives 0 instead of UB.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI(
    dll_table: *mut DLL_FUNCTIONS,
    interface_version: i32,
) -> i32 {
    catch_ffi_panic("GetEntityAPI", 0, || {
        if dll_table.is_null() {
            return 0;
        }
        let mut ver = interface_version;
        // SAFETY: dll_table is valid, verified above.
        unsafe { GetEntityAPI2(dll_table, &mut ver) }
    })
}

/// Called by engine for NEW_DLL_FUNCTIONS interface (ReGameDLL, Sven Co-op, HLSDK 2.x).
///
/// # Safety
/// Pointers must be valid.
/// Any Rust panic is caught — the C caller receives 0 instead of UB.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetNewDLLFunctions(
    new_dll_table: *mut std::ffi::c_void,
    interface_version: *mut i32,
) -> i32 {
    catch_ffi_panic("GetNewDLLFunctions", 0, || {
        if new_dll_table.is_null() {
            return 0;
        }
        unsafe {
            if !interface_version.is_null() {
                *interface_version = 1;
            }
        }
        if proxy::populate_new_dll_table(new_dll_table) {
            1
        } else {
            0
        }
    })
}

/// Called by engine to query the studio model animation blending interface.
///
/// # Safety
/// Pointers must be valid or null as accepted by HLSDK.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn Server_GetBlendingInterface(
    version: i32,
    ppinterface: *mut *mut std::ffi::c_void,
    pstudio: *mut std::ffi::c_void,
    rotationmatrix: *mut std::ffi::c_void,
    bonetransform: *mut std::ffi::c_void,
) -> i32 {
    catch_ffi_panic("Server_GetBlendingInterface", 0, || unsafe {
        proxy::forward_server_get_blending_interface(
            version,
            ppinterface,
            pstudio,
            rotationmatrix,
            bonetransform,
        )
    })
}

/// Called by engine/modules to query abstract interfaces (e.g. VServerAdmin).
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn CreateInterface(
    name: *const std::os::raw::c_char,
    return_code: *mut i32,
) -> *mut std::ffi::c_void {
    catch_ffi_panic("CreateInterface", std::ptr::null_mut(), || unsafe {
        proxy::forward_create_interface(name, return_code)
    })
}
