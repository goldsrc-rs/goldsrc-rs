//! Metamod backend implementation for GoldSrc.rs.

#![allow(static_mut_refs)]

mod meta_types;

use goldsrc_api::Engine;
use goldsrc_sys;
use std::ffi::c_void;
use std::ffi::CString;

use meta_types::*;

static mut G_ENGFUNCS: Option<goldsrc_sys::enginefuncs_t> = None;
static mut G_GLOBALS: Option<goldsrc_sys::globalvars_t> = None;
static mut G_META_GLOBALS: Option<*mut meta_globals_t> = None;
static mut G_DLL_FUNCTIONS: *mut c_void = std::ptr::null_mut();

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
    ($func:expr, $($arg:expr),*) => {
        if let Some(f) = $func { f($($arg),*); }
    };
}

macro_rules! call_engfunc_ret {
    ($func:expr, $($arg:expr),*) => {
        if let Some(f) = $func { f($($arg),*) } else { Default::default() }
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

static BACKEND: MetamodBackend = MetamodBackend::new();

pub fn backend() -> &'static MetamodBackend {
    &BACKEND
}

// ============================================================================
// Hook System
// ============================================================================

/// Hook callback type for entity functions.
type EntityHookFn = unsafe extern "C" fn(*mut goldsrc_sys::edict_t) -> i32;

/// Hook callback type for client connect.
type ClientConnectHookFn =
    unsafe extern "C" fn(*mut goldsrc_sys::edict_t, *const i8, *const i8, *mut [i8; 128]) -> i32;

/// Hook callback type for client command.
type ClientCommandHookFn = unsafe extern "C" fn(*mut goldsrc_sys::edict_t);

/// Original function pointers that we hook.
static mut ORIGINAL_SPAWN: Option<EntityHookFn> = None;
static mut ORIGINAL_CLIENT_CONNECT: Option<ClientConnectHookFn> = None;
static mut ORIGINAL_CLIENT_COMMAND: Option<ClientCommandHookFn> = None;

/// Hook for DispatchSpawn - called when an entity spawns.
unsafe extern "C" fn hook_spawn(edict: *mut goldsrc_sys::edict_t) -> i32 {
    if !edict.is_null() {
        backend().server_print("[GoldSrc.rs] Entity spawned.\n");
    }

    // Call original function
    if let Some(original) = ORIGINAL_SPAWN {
        original(edict)
    } else {
        0
    }
}

/// Hook for ClientConnect - called when a player connects.
unsafe extern "C" fn hook_client_connect(
    entity: *mut goldsrc_sys::edict_t,
    name: *const i8,
    address: *const i8,
    reject_reason: *mut [i8; 128],
) -> i32 {
    if !name.is_null() {
        let name_str = std::ffi::CStr::from_ptr(name).to_string_lossy();
        let msg = format!("[GoldSrc.rs] Player {} connecting...\n", name_str);
        let cmsg = CString::new(msg).unwrap_or_default();
        call_engfunc!(engfuncs().pfnServerPrint, cmsg.as_ptr());
    }

    // Call original function
    if let Some(original) = ORIGINAL_CLIENT_CONNECT {
        original(entity, name, address, reject_reason)
    } else {
        0
    }
}

/// Hook for ClientCommand - called when a player issues a command.
unsafe extern "C" fn hook_client_command(entity: *mut goldsrc_sys::edict_t) {
    if !entity.is_null() {
        backend().server_print("[GoldSrc.rs] Client command received.\n");
    }

    // Call original function
    if let Some(original) = ORIGINAL_CLIENT_COMMAND {
        original(entity);
    }
}

/// Register hooks for entity and engine functions.
unsafe fn register_hooks(meta_functions: *mut c_void) {
    if meta_functions.is_null() {
        return;
    }

    // TODO: Implement proper hook registration using META_FUNCTIONS table
    // This requires parsing the META_FUNCTIONS struct and calling pfnGetEntityAPI
    // to register our hook functions.

    backend().server_print("[GoldSrc.rs] Hook system initialized (TODO: register hooks).\n");
}

// ============================================================================
// Metamod Entry Points
// ============================================================================

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

#[no_mangle]
#[inline(never)]
pub extern "C" fn Meta_Attach(
    _now: PLUG_LOADTIME,
    meta_functions: *mut c_void,
    meta_globals: *mut meta_globals_t,
    gamedll_funcs: *mut c_void,
) -> std::os::raw::c_int {
    unsafe {
        if meta_globals.is_null() {
            return 0;
        }
        G_META_GLOBALS = Some(meta_globals);
        G_DLL_FUNCTIONS = gamedll_funcs;

        // Register hooks
        register_hooks(meta_functions);
    }
    backend().server_print("[GoldSrc.rs] Meta_Attach called.\n");
    backend().server_print("[GoldSrc.rs] Hello from Rust!\n");
    1
}

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
    version: c"0.1.0".as_ptr(),
    date: c"2026-08-10".as_ptr(),
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
