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
mod proxy;

use goldsrc::backend::EngineBackend;
use goldsrc::logging::LogTarget;
use goldsrc_api::Engine;
use goldsrc_sys::ffi::catch_ffi_panic;
use goldsrc_sys::{enginefuncs_t, globalvars_t, DLL_FUNCTIONS};
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
    let engine: std::sync::Arc<dyn goldsrc_api::EngineOps> = std::sync::Arc::new(*backend());
    if let Err(e) = goldsrc::host::HostRuntime::init(
        "standalone",
        |msg| {
            backend().server_print(msg);
        },
        engine,
    ) {
        goldsrc::gslog_error!(LogTarget::Core, "{e}");
    }
}

// ============================================================================
// Hook implementations
// ============================================================================

unsafe extern "C" fn hook_start_frame() {
    // SAFETY: forward_start_frame does not unwind; catch_unwind guards the ABI boundary.
    catch_ffi_panic("hook_start_frame", (), || {
        proxy::forward_start_frame();
        goldsrc::host::HostRuntime::on_server_frame();
        drain_print_queue();
    });
}

/// Drain deferred server prints with fmtlib-safe escaping.
///
/// ReHLDS routes `ServerPrint` output through fmtlib: `%` and `{}` from plugin
/// text would throw and crash the server, so they are escaped before printing.
fn drain_print_queue() {
    for message in PRINT_QUEUE.drain() {
        unsafe {
            let funcs = engine_api::engfuncs();
            if let Some(f) = funcs.pfnServerPrint {
                if let Ok(cstr) = CString::new(message) {
                    f(cstr.as_ptr());
                }
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
    catch_ffi_panic("hook_client_connect", 1, || {
        let result = proxy::forward_client_connect(edict, name, address, reject_reason);
        goldsrc::host::HostRuntime::with_manager(|m| {
            if let Some(manager) = m {
                manager.call_on_event("client_connect", &[]);
            }
        });
        result
    })
}

unsafe extern "C" fn hook_client_disconnect(edict: *mut goldsrc_sys::edict_t) {
    catch_ffi_panic("hook_client_disconnect", (), || {
        proxy::forward_client_disconnect(edict);
        goldsrc::host::HostRuntime::with_manager(|m| {
            if let Some(manager) = m {
                manager.call_on_event("client_disconnect", &[]);
            }
        });
    });
}

unsafe extern "C" fn hook_client_command(edict: *mut goldsrc_sys::edict_t) {
    catch_ffi_panic("hook_client_command", (), || {
        proxy::forward_client_command(edict);
        let cmd_str;
        let name_str;
        {
            let funcs = engine_api::engfuncs();
            let cmd_ptr = funcs.pfnCmd_Args.map(|f| f()).unwrap_or(std::ptr::null());
            cmd_str = if !cmd_ptr.is_null() {
                CStr::from_ptr(cmd_ptr).to_string_lossy().into_owned()
            } else {
                String::new()
            };
            let name_ptr = funcs.pfnCmd_Argv.map(|f| f(0)).unwrap_or(std::ptr::null());
            name_str = if !name_ptr.is_null() {
                CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
            } else {
                String::new()
            };
        }
        goldsrc::host::HostRuntime::with_manager(|m| {
            if let Some(manager) = m {
                manager.dispatch_command(&name_str, &cmd_str);
            }
        });
    });
}

unsafe extern "C" fn hook_spawn(edict: *mut goldsrc_sys::edict_t) -> i32 {
    catch_ffi_panic("hook_spawn", 0, || proxy::forward_spawn(edict))
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
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GiveFnptrsToDll(engfuncs: *mut enginefuncs_t, globals: *mut globalvars_t) {
    // SAFETY: engfuncs and globals are engine-provided; valid for the server lifetime.
    catch_ffi_panic("GiveFnptrsToDll", (), || {
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
        // 4. Register mrs/meta-rs server commands.
        commands::register_cli_commands();
        backend().server_print("[GoldSrc.rs Standalone] Hello from Rust! (Standalone Mode)\n");
    });
}

/// Called by the engine to retrieve our `DLL_FUNCTIONS` hook table.
///
/// # Safety
/// `dll_table` is a valid pointer provided by the engine.
/// Any Rust panic is caught — the C caller receives 0 instead of UB.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI2(
    dll_table: *mut DLL_FUNCTIONS,
    interface_version: *mut i32,
) -> i32 {
    // SAFETY: dll_table and interface_version are engine-provided; valid at call time.
    catch_ffi_panic("GetEntityAPI2", 0, || {
        if dll_table.is_null() || interface_version.is_null() {
            return 0;
        }
        unsafe {
            if *interface_version != 140 {
                *interface_version = 140;
                return 0;
            }
            // 1. Populate with real GameDLL (mp.dll / cs.so) callbacks.
            proxy::populate_dll_table(dll_table);

            // 2. Overlay our hooks.
            let table = &mut *dll_table;
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
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI(
    dll_table: *mut DLL_FUNCTIONS,
    interface_version: i32,
) -> i32 {
    catch_ffi_panic("GetEntityAPI", 0, || {
        if dll_table.is_null() || interface_version != 140 {
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
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn GetNewDLLFunctions(
    new_dll_table: *mut std::ffi::c_void,
    interface_version: *mut i32,
) -> i32 {
    catch_ffi_panic("GetNewDLLFunctions", 0, || {
        if new_dll_table.is_null() || interface_version.is_null() {
            return 0;
        }
        unsafe {
            if *interface_version != 1 {
                *interface_version = 1;
                return 0;
            }
        }
        if proxy::populate_new_dll_table(new_dll_table) {
            1
        } else {
            0
        }
    })
}
