//! Metamod backend implementation for GoldSrc.rs.

#![allow(static_mut_refs)]

mod meta_types;

use goldsrc_api::{Engine, Entity, Player};
use std::ffi::c_void;
use std::ffi::CString;

use meta_types::*;

// SAFETY: Written once during init, only read afterwards from game thread.
static mut G_ENGFUNCS: Option<goldsrc_sys::enginefuncs_t> = None;
static mut G_GLOBALS: Option<goldsrc_sys::globalvars_t> = None;
static mut G_META_UTILS: Option<mutil_funcs_t> = None;
static mut G_META_GLOBALS: Option<*mut meta_globals_t> = None;

/// Initialize the backend with engine functions and global variables.
///
/// # Safety
/// Must be called exactly once from `GiveFnptrsToDll`.
pub unsafe fn init_backend(
    engfuncs: *const goldsrc_sys::enginefuncs_t,
    globals: *const goldsrc_sys::globalvars_t,
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

/// Initialize the backend with Metamod utility functions.
///
/// # Safety
/// Must be called exactly once from `Meta_Attach`.
pub unsafe fn init_meta_utils(utils: *const mutil_funcs_t, meta_globals: *mut meta_globals_t) {
    unsafe {
        if !utils.is_null() {
            G_META_UTILS = Some(*utils);
        }
        G_META_GLOBALS = Some(meta_globals);
    }
}

/// Get the engine functions.
pub fn engfuncs() -> &'static goldsrc_sys::enginefuncs_t {
    unsafe { G_ENGFUNCS.as_ref().expect("Backend not initialized") }
}

/// Get the global variables.
pub fn globals() -> &'static goldsrc_sys::globalvars_t {
    unsafe { G_GLOBALS.as_ref().expect("Backend not initialized") }
}

/// Get the Metamod utility functions.
pub fn meta_utils() -> &'static mutil_funcs_t {
    unsafe { G_META_UTILS.as_ref().expect("Meta utils not initialized") }
}

/// Get the Metamod globals pointer.
pub fn meta_globals() -> &'static mut meta_globals_t {
    unsafe {
        G_META_GLOBALS
            .expect("Meta globals not initialized")
            .as_mut()
            .expect("Meta globals pointer is null")
    }
}

/// Call an engine function, skipping if not available.
macro_rules! call_engfunc {
    ($func:expr, $($arg:expr),*) => {
        if let Some(f) = $func {
            f($($arg),*);
        }
    };
}

/// Call an engine function returning a value, returning default if not available.
macro_rules! call_engfunc_ret {
    ($func:expr, $($arg:expr),*) => {
        if let Some(f) = $func {
            f($($arg),*)
        } else {
            Default::default()
        }
    };
}

/// Metamod backend — implements `Engine` using the Metamod API.
pub struct MetamodBackend;

impl Default for MetamodBackend {
    fn default() -> Self {
        Self
    }
}

impl MetamodBackend {
    /// Create a new Metamod backend instance.
    pub const fn new() -> Self {
        Self
    }
}

impl Engine for MetamodBackend {
    fn spawn_entity(&self, classname: &str) -> Option<Entity> {
        unsafe {
            let funcs = engfuncs();
            let edict = (funcs.pfnCreateEntity)?();
            if edict.is_null() {
                return None;
            }
            let cname = CString::new(classname).unwrap_or_default();
            (funcs.pfnSetModel)?(edict, cname.as_ptr());
            let index = (funcs.pfnIndexOfEdict)?(edict);
            Some(Entity::from_raw(index, edict))
        }
    }

    fn get_player(&self, index: i32) -> Option<Player> {
        unsafe {
            let funcs = engfuncs();
            let edict = (funcs.pfnPEntityOfEntIndex)?(index);
            if edict.is_null() {
                return None;
            }
            Some(Player::from_raw(index, edict))
        }
    }

    fn server_print(&self, message: &str) {
        unsafe {
            let msg = CString::new(message).unwrap_or_default();
            call_engfunc!(engfuncs().pfnServerPrint, msg.as_ptr());
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

/// Global backend instance.
static BACKEND: MetamodBackend = MetamodBackend::new();

/// Get the global backend instance.
pub fn backend() -> &'static MetamodBackend {
    &BACKEND
}

// ============================================================================
// Metamod Entry Points
// ============================================================================

/// Called by the engine to provide engine function pointers.
///
/// # Safety
/// Called by the engine during DLL loading.
#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn GiveFnptrsToDll(
    engfuncs: *const goldsrc_sys::enginefuncs_t,
    globals: *const goldsrc_sys::globalvars_t,
) {
    unsafe {
        init_backend(engfuncs, globals);
    }
    backend().server_print("[GoldSrc.rs] Engine functions received.\n");
}

/// Called by Metamod to query the plugin interface.
///
/// # Safety
/// Called by Metamod during plugin loading.
#[no_mangle]
#[inline(never)]
pub extern "C" fn Meta_Query(
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

/// Called by Metamod to attach the plugin.
///
/// # Safety
/// Called by Metamod after Meta_Query.
#[no_mangle]
#[inline(never)]
pub extern "C" fn Meta_Attach(
    _now: PLUG_LOADTIME,
    _meta_functions: *mut meta_function_t,
    meta_globals: *mut meta_globals_t,
    _gamedll_funcs: *const c_void,
) -> std::os::raw::c_int {
    unsafe {
        if meta_globals.is_null() {
            return 0;
        }
        init_meta_utils(std::ptr::null(), meta_globals);
        backend().server_print("[GoldSrc.rs] Meta_Attach called.\n");
        backend().server_print("[GoldSrc.rs] Hello from Rust!\n");
    }
    1
}

/// Called by Metamod to detach the plugin.
///
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

/// Plugin info structure.
#[allow(non_upper_case_globals)]
static PLUGIN_INFO: plugin_info_t = plugin_info_t {
    ifvers: META_INTERFACE_VERSION.as_ptr() as *const i8,
    name: c"GoldSrc.rs Metamod Backend".as_ptr(),
    version: c"0.1.0".as_ptr(),
    date: c"2026-08-10".as_ptr(),
    author: c"GoldSrc.rs Contributors".as_ptr(),
    url: c"https://github.com/ulquiorracode/GoldSrc.rs".as_ptr(),
    logtag: c"GOLDSRC.RS".as_ptr(),
    loadable: PLUG_LOADTIME::PT_ANYTIME,
    unloadable: PLUG_LOADTIME::PT_ANYTIME,
};

/// Get the meta utility functions.
fn get_meta_util_funcs() -> mutil_funcs_t {
    mutil_funcs_t {
        pfnLogConsole: Some(meta_log_console),
        pfnLogMessage: Some(meta_log_message),
        pfnLogError: Some(meta_log_error),
        pfnLogDeveloper: Some(meta_log_developer),
        pfnCenterSay: None,
        pfnCenterSayParms: None,
        pfnCenterSayVarargs: None,
        pfnCallGameEntity: None,
        pfnGetUserMsgID: None,
        pfnGetUserMsgName: None,
        pfnGetPluginPath: None,
        pfnGetGameInfo: None,
        pfnLoadPlugin: None,
        pfnUnloadPlugin: None,
        pfnUnloadPluginByHandle: None,
        pfnIsQueryingClientCvar: None,
        pfnMakeRequestId: None,
        pfnGetHookTables: None,
    }
}

unsafe extern "C" fn meta_log_console(_plid: *const plugin_info_t, _fmt: *const i8) {}
unsafe extern "C" fn meta_log_message(_plid: *const plugin_info_t, _fmt: *const i8) {}
unsafe extern "C" fn meta_log_error(_plid: *const plugin_info_t, _fmt: *const i8) {}
unsafe extern "C" fn meta_log_developer(_plid: *const plugin_info_t, _fmt: *const i8) {}

// Prevent the linker from stripping exported functions
#[used]
static EXPORT_POINTERS: &[unsafe extern "C" fn()] = &[
    unsafe { std::mem::transmute(GiveFnptrsToDll as *const ()) },
    unsafe { std::mem::transmute(Meta_Query as *const ()) },
    unsafe { std::mem::transmute(Meta_Attach as *const ()) },
    unsafe { std::mem::transmute(Meta_Detach as *const ()) },
];
