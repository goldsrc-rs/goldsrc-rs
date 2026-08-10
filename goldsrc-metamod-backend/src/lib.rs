//! Metamod backend implementation for GoldSrc.rs.

#![allow(static_mut_refs)]

mod meta_types;

use goldsrc_api::Engine;

use std::ffi::CString;
use std::ffi::c_void;

use meta_types::*;

static mut G_ENGFUNCS: Option<goldsrc_sys::enginefuncs_t> = None;
static mut G_GLOBALS: Option<goldsrc_sys::globalvars_t> = None;
static mut G_META_GLOBALS: Option<*mut meta_globals_t> = None;

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
    fn default() -> Self { Self }
}

impl MetamodBackend {
    pub const fn new() -> Self { Self }
}

impl Engine for MetamodBackend {
    fn spawn_entity(&self, classname: &str) -> Option<goldsrc_api::Entity> {
        unsafe {
            let funcs = engfuncs();
            let edict = (funcs.pfnCreateEntity)?();
            if edict.is_null() { return None; }
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
            if edict.is_null() { return None; }
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

pub fn backend() -> &'static MetamodBackend { &BACKEND }

// ============================================================================
// Hook System
// ============================================================================

/// Original function pointers that we hook.
static mut ORIGINAL_SPAWN: Option<unsafe extern "C" fn(*mut goldsrc_sys::edict_t) -> std::os::raw::c_int> = None;
static mut ORIGINAL_CLIENT_CONNECT: Option<
    unsafe extern "C" fn(
        *mut goldsrc_sys::edict_t,
        *const std::os::raw::c_char,
        *const std::os::raw::c_char,
        *mut std::os::raw::c_char,
    ) -> std::os::raw::c_int,
> = None;
static mut ORIGINAL_CLIENT_COMMAND: Option<unsafe extern "C" fn(*mut goldsrc_sys::edict_t)> = None;

/// Hook for DispatchSpawn - called when an entity spawns.
///
/// # Safety
/// `edict` must be a valid pointer to an edict_t.
#[allow(dead_code)]
unsafe extern "C" fn hook_spawn(edict: *mut goldsrc_sys::edict_t) -> std::os::raw::c_int {
    if !edict.is_null() {
        backend().server_print("[GoldSrc.rs] Entity spawned.\n");
    }
    match ORIGINAL_SPAWN {
        Some(f) => f(edict),
        None => 0,
    }
}

/// Hook for ClientConnect - called when a player connects.
///
/// # Safety
/// Pointers must be valid C strings.
#[allow(dead_code)]
unsafe extern "C" fn hook_client_connect(
    entity: *mut goldsrc_sys::edict_t,
    name: *const std::os::raw::c_char,
    address: *const std::os::raw::c_char,
    reject_reason: *mut std::os::raw::c_char,
) -> std::os::raw::c_int {
    if !name.is_null() {
        let name_str = std::ffi::CStr::from_ptr(name).to_string_lossy();
        let msg = format!("[GoldSrc.rs] Player {} connecting...\n", name_str);
        let cmsg = CString::new(msg).unwrap_or_default();
        call_engfunc!(engfuncs().pfnServerPrint, cmsg.as_ptr());
    }
    match ORIGINAL_CLIENT_CONNECT {
        Some(f) => f(entity, name, address, reject_reason),
        None => 0,
    }
}

/// Hook for ClientCommand - called when a player issues a command.
///
/// # Safety
/// `entity` must be a valid pointer to an edict_t.
#[allow(dead_code)]
unsafe extern "C" fn hook_client_command(entity: *mut goldsrc_sys::edict_t) {
    if !entity.is_null() {
        backend().server_print("[GoldSrc.rs] Client command received.\n");
    }
    if let Some(f) = ORIGINAL_CLIENT_COMMAND {
        f(entity);
    }
}

/// Register hooks for entity functions.
///
/// # Safety
/// `meta_functions` must be a valid pointer to a META_FUNCTIONS struct.
unsafe fn register_hooks(meta_functions: *mut c_void) {
    if meta_functions.is_null() {
        backend().server_print("[GoldSrc.rs] Warning: meta_functions is null\n");
        return;
    }

    // Cast to META_FUNCTIONS struct
    let meta_funcs = &*(meta_functions as *const MetaFunctions);

    // Get entity API table to hook DispatchSpawn, ClientConnect, ClientCommand
    let mut dll_funcs: goldsrc_sys::DLL_FUNCTIONS = std::mem::zeroed();
    let interface_version = 140; // HL SDK interface version

    if let Some(get_entity_api) = meta_funcs.pfnGetEntityAPI {
        let result = get_entity_api(&mut dll_funcs, interface_version);
        if result != 0 {
            // Save originals and replace with hooks
            ORIGINAL_SPAWN = dll_funcs.pfnSpawn;
            ORIGINAL_CLIENT_CONNECT = dll_funcs.pfnClientConnect;
            ORIGINAL_CLIENT_COMMAND = dll_funcs.pfnClientCommand;

            // Note: Full hook registration requires modifying the DLL_FUNCTIONS table
            // and re-registering it with Metamod. For now, we just save the originals.
            // dll_funcs.pfnSpawn = Some(hook_spawn);
            // dll_funcs.pfnClientConnect = Some(hook_client_connect);
            // dll_funcs.pfnClientCommand = Some(hook_client_command);

            backend().server_print("[GoldSrc.rs] Saved original function pointers.\n");
        } else {
            backend().server_print("[GoldSrc.rs] Warning: GetEntityAPI returned 0\n");
        }
    } else {
        backend().server_print("[GoldSrc.rs] Warning: pfnGetEntityAPI is null\n");
    }

    backend().server_print("[GoldSrc.rs] Hook system initialized.\n");
}

/// META_FUNCTIONS struct from Metamod API.
#[repr(C)]
#[allow(non_snake_case)]
struct MetaFunctions {
    pfnGetEntityAPI: Option<
        unsafe extern "C" fn(*mut goldsrc_sys::DLL_FUNCTIONS, i32) -> i32,
    >,
    pfnGetEntityAPI_Post: Option<
        unsafe extern "C" fn(*mut goldsrc_sys::DLL_FUNCTIONS, i32) -> i32,
    >,
    pfnGetEntityAPI2: Option<
        unsafe extern "C" fn(*mut goldsrc_sys::DLL_FUNCTIONS, *mut i32) -> i32,
    >,
    pfnGetEntityAPI2_Post: Option<
        unsafe extern "C" fn(*mut goldsrc_sys::DLL_FUNCTIONS, *mut i32) -> i32,
    >,
    pfnGetNewDLLFunctions: Option<
        unsafe extern "C" fn(*mut c_void, *mut i32) -> i32,
    >,
    pfnGetNewDLLFunctions_Post: Option<
        unsafe extern "C" fn(*mut c_void, *mut i32) -> i32,
    >,
    pfnGetEngineFunctions: Option<
        unsafe extern "C" fn(*mut goldsrc_sys::enginefuncs_t, *mut i32) -> i32,
    >,
    pfnGetEngineFunctions_Post: Option<
        unsafe extern "C" fn(*mut goldsrc_sys::enginefuncs_t, *mut i32) -> i32,
    >,
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
    unsafe { init_backend(engfuncs, globals); }
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
        if plugin_info.is_null() || meta_util_functions.is_null() { return 0; }
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
    meta_functions: *mut c_void,
    meta_globals: *mut meta_globals_t,
    _gamedll_funcs: *mut c_void,
) -> std::os::raw::c_int {
    unsafe {
        if meta_globals.is_null() { return 0; }
        G_META_GLOBALS = Some(meta_globals);
        register_hooks(meta_functions);
    }
    backend().server_print("[GoldSrc.rs] Meta_Attach called.\n");
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
