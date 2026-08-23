//! Metamod `#[unsafe(no_mangle)]` entry points (FFI boundary).

use goldsrc_sys::ffi::catch_ffi_panic;
use std::ffi::c_void;

use crate::meta_types::*;
use crate::{backend, init_backend, init_wasm_host};

/// # Safety
/// Called by the engine during DLL loading. `engfuncs` and `globals` are
/// valid pointers for the lifetime of the server process.
/// Any Rust panic is caught — an unhandled panic here would crash HLDS.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "system" fn GiveFnptrsToDll(
    engfuncs: *mut goldsrc_sys::enginefuncs_t,
    globals: *mut goldsrc_sys::globalvars_t,
) {
    // SAFETY: engfuncs and globals are engine-provided; valid for the server lifetime.
    catch_ffi_panic("GiveFnptrsToDll", (), || {
        unsafe { init_backend(engfuncs, globals) };
        backend().server_print("[GoldSrc.rs] Engine functions received.\n");
    });
}

/// # Safety
/// Called by Metamod during plugin loading. Pointers must be valid.
/// Any Rust panic is caught — the C caller receives 0 instead of UB.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn Meta_Query(
    _ifvers: *const std::os::raw::c_char,
    plugin_info: *mut *const plugin_info_t,
    meta_util_functions: *mut mutil_funcs_t,
) -> std::os::raw::c_int {
    // SAFETY: plugin_info and meta_util_functions are Metamod-provided; valid at call time.
    catch_ffi_panic("Meta_Query", 0, || {
        unsafe {
            if plugin_info.is_null() {
                return 0;
            }
            *plugin_info = &PLUGIN_INFO;
            if !meta_util_functions.is_null() {
                crate::set_meta_util_funcs(meta_util_functions);
            }
        }
        backend().server_print("[GoldSrc.rs] Meta_Query called.\n");
        1
    })
}

/// # Safety
/// Called by Metamod after Meta_Query. Pointers must be valid.
/// Any Rust panic is caught — the C caller receives 0 instead of UB.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn Meta_Attach(
    _now: PLUG_LOADTIME,
    meta_functions: *mut meta_function_t,
    meta_globals: *mut meta_globals_t,
    _gamedll_funcs: *mut c_void,
) -> std::os::raw::c_int {
    // SAFETY: meta_functions and meta_globals are Metamod-provided; valid at call time.
    catch_ffi_panic("Meta_Attach", 0, || {
        unsafe {
            if meta_globals.is_null() || meta_functions.is_null() {
                return 0;
            }
            crate::set_meta_globals(meta_globals);

            // Fill the META_FUNCTIONS table with our hook functions.
            (*meta_functions).pfnGetEntityAPI = Some(crate::GetEntityAPI);
            (*meta_functions).pfnGetEntityAPI_Post = Some(crate::GetEntityAPI_Post);
            (*meta_functions).pfnGetEntityAPI2 = Some(crate::GetEntityAPI2);
            (*meta_functions).pfnGetEntityAPI2_Post = Some(crate::GetEntityAPI2_Post);
            (*meta_functions).pfnGetNewDLLFunctions = Some(crate::GetNewDLLFunctions);
            (*meta_functions).pfnGetNewDLLFunctions_Post = Some(crate::GetNewDLLFunctions_Post);
            (*meta_functions).pfnGetEngineFunctions = Some(crate::GetEngineFunctions);
            (*meta_functions).pfnGetEngineFunctions_Post = Some(crate::GetEngineFunctions_Post);
        }
        init_wasm_host();
        crate::commands::register_cli_commands();
        backend().server_print("[GoldSrc.rs] Meta_Attach called.\n");
        backend().server_print("[GoldSrc.rs] WASM Host Engine initialized.\n");
        backend()
            .server_print("[GoldSrc.rs] Host Management CLI registered (`meta-rs` / `goldsrc`).\n");
        backend().server_print("[GoldSrc.rs] Hello from Rust!\n");
        1
    })
}

/// # Safety
/// Called by Metamod during plugin unloading.
/// Any Rust panic is caught — the C caller receives 0 instead of UB.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn Meta_Detach(
    _now: PLUG_LOADTIME,
    _reason: PL_UNLOAD_REASON,
) -> std::os::raw::c_int {
    catch_ffi_panic("Meta_Detach", 1, || {
        backend().server_print("[GoldSrc.rs] Meta_Detach called. Goodbye!\n");
        1
    })
}

#[allow(non_upper_case_globals)]
pub static PLUGIN_INFO: plugin_info_t = plugin_info_t {
    ifvers: META_INTERFACE_VERSION.as_ptr(),
    name: c"GoldSrc.rs Metamod Backend".as_ptr(),
    version: concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const i8,
    date: concat!(env!("GIT_HASH"), "\0").as_ptr() as *const i8,
    author: c"GoldSrc.rs Contributors".as_ptr(),
    url: c"https://github.com/ulquiorracode/GoldSrc.rs".as_ptr(),
    logtag: c"GOLDSRC.RS".as_ptr(),
    loadable: PLUG_LOADTIME::PT_ANYTIME,
    unloadable: PLUG_LOADTIME::PT_ANYTIME,
};

// ============================================================================
// Entity API entry points
// ============================================================================

/// Function tables that we provide to Metamod.
/// Metamod calls these to get our hook functions.
/// # Safety
/// Called by Metamod to get entity API hooks. Pointers must be valid.
/// Note: This is only called when the plugin is loaded as a game DLL plugin.
/// Any Rust panic is caught and contained — the C caller receives 0 instead of UB.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI2(
    dll_table: *mut goldsrc_sys::DLL_FUNCTIONS,
    interface_version: *mut i32,
) -> i32 {
    // SAFETY: dll_table and interface_version are engine-provided; valid at call time.
    catch_ffi_panic("GetEntityAPI2", 0, || {
        if dll_table.is_null() || interface_version.is_null() {
            return 0;
        }
        unsafe {
            if *interface_version != goldsrc_api::consts::ENGINE_INTERFACE_VERSION {
                *interface_version = goldsrc_api::consts::ENGINE_INTERFACE_VERSION;
                return 0;
            }
            let table = &mut *dll_table;
            table.pfnSpawn = Some(crate::hooks::hook_spawn);
            table.pfnServerActivate = Some(crate::hooks::hook_server_activate);
            table.pfnServerDeactivate = Some(crate::hooks::hook_server_deactivate);
            table.pfnClientConnect = Some(crate::hooks::hook_client_connect);
            table.pfnClientDisconnect = Some(crate::hooks::hook_client_disconnect);
            table.pfnClientCommand = Some(crate::hooks::hook_client_command);
            table.pfnStartFrame = Some(crate::hooks::hook_start_frame);
            table.pfnPlayerPostThink = Some(crate::hooks::hook_player_post_think);
            table.pfnClientKill = Some(crate::hooks::hook_client_kill);
            table.pfnTouch = Some(crate::hooks::hook_touch);
            table.pfnUse = Some(crate::hooks::hook_use);
        }
        1
    })
}

/// # Safety
/// Called by Metamod to get post-entity API hooks.
/// Any Rust panic is caught — the C caller receives 0 instead of UB.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI2_Post(
    dll_table: *mut goldsrc_sys::DLL_FUNCTIONS,
    interface_version: *mut i32,
) -> i32 {
    // SAFETY: dll_table and interface_version are engine-provided; valid at call time.
    catch_ffi_panic("GetEntityAPI2_Post", 0, || {
        if dll_table.is_null() || interface_version.is_null() {
            return 0;
        }
        unsafe {
            if *interface_version != goldsrc_api::consts::ENGINE_INTERFACE_VERSION {
                *interface_version = goldsrc_api::consts::ENGINE_INTERFACE_VERSION;
                return 0;
            }
            let table = &mut *dll_table;
            table.pfnSpawn = Some(crate::hooks::hook_spawn_post);
            table.pfnClientConnect = Some(crate::hooks::hook_client_connect_post);
            table.pfnClientDisconnect = Some(crate::hooks::hook_client_disconnect_post);
            table.pfnStartFrame = Some(crate::hooks::hook_start_frame_post);
        }
        1
    })
}

/// # Safety
/// Called by Metamod to get entity API hooks (old interface).
/// Any Rust panic is caught — the C caller receives 0 instead of UB.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI(
    dll_table: *mut goldsrc_sys::DLL_FUNCTIONS,
    interface_version: i32,
) -> i32 {
    catch_ffi_panic("GetEntityAPI", 0, || {
        if dll_table.is_null() || interface_version != goldsrc_api::consts::ENGINE_INTERFACE_VERSION
        {
            return 0;
        }
        backend().server_print("[GoldSrc.rs] GetEntityAPI called.\n");
        1
    })
}

/// # Safety
/// Called by Metamod to get post-entity API hooks (old interface).
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI_Post(
    _dll_table: *mut goldsrc_sys::DLL_FUNCTIONS,
    _interface_version: i32,
) -> i32 {
    catch_ffi_panic("GetEntityAPI_Post", 0, || 0)
}

/// # Safety
/// Called by Metamod to get new DLL functions.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetNewDLLFunctions(
    _new_table: *mut c_void,
    _interface_version: *mut i32,
) -> i32 {
    catch_ffi_panic("GetNewDLLFunctions", 0, || 0)
}

/// # Safety
/// Called by Metamod to get post-new DLL functions.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetNewDLLFunctions_Post(
    _new_table: *mut c_void,
    _interface_version: *mut i32,
) -> i32 {
    catch_ffi_panic("GetNewDLLFunctions_Post", 0, || 0)
}

/// # Safety
/// Called by Metamod to get engine functions. Pointers must be valid.
/// Any Rust panic is caught — the C caller receives 0 instead of UB.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetEngineFunctions(
    engfuncs: *mut goldsrc_sys::enginefuncs_t,
    _interface_version: *mut i32,
) -> i32 {
    catch_ffi_panic("GetEngineFunctions", 0, || {
        if engfuncs.is_null() {
            return 0;
        }
        backend().server_print("[GoldSrc.rs] GetEngineFunctions called.\n");
        1
    })
}

/// # Safety
/// Called by Metamod to get post-engine functions.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetEngineFunctions_Post(
    _engfuncs: *mut goldsrc_sys::enginefuncs_t,
    _interface_version: *mut i32,
) -> i32 {
    catch_ffi_panic("GetEngineFunctions_Post", 0, || 0)
}
