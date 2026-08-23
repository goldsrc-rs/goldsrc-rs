//! Metamod type definitions (manually defined to avoid C++ bindgen issues).

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_void;
use std::ffi::c_char;

pub const META_INTERFACE_VERSION: &std::ffi::CStr = c"5:13";

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PLUG_LOADTIME {
    PT_NEVER = 0,
    PT_STARTUP,
    PT_CHANGELEVEL,
    PT_ANYTIME,
    PT_ANYPAUSE,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PL_UNLOAD_REASON {
    PNL_NULL = 0,
    PNL_INI_DELETED,
    PNL_FILE_NEWER,
    PNL_COMMAND,
    PNL_CMD_FORCED,
    PNL_DELAYED,
    PNL_PLUGIN,
    PNL_PLG_FORCED,
    PNL_RELOAD,
}

#[repr(C)]
pub struct plugin_info_t {
    pub ifvers: *const c_char,
    pub name: *const c_char,
    pub version: *const c_char,
    pub date: *const c_char,
    pub author: *const c_char,
    pub url: *const c_char,
    pub logtag: *const c_char,
    pub loadable: PLUG_LOADTIME,
    pub unloadable: PLUG_LOADTIME,
}

unsafe impl Sync for plugin_info_t {}

#[allow(dead_code)]
pub const MRES_UNSET: i32 = 0;
#[allow(dead_code)]
pub const MRES_IGNORED: i32 = 1;
#[allow(dead_code)]
pub const MRES_HANDLED: i32 = 2;
#[allow(dead_code)]
pub const MRES_OVERRIDE: i32 = 3;
pub const MRES_SUPERCEDE: i32 = 4;

#[repr(C)]
pub struct meta_globals_t {
    pub mres: i32,
    pub prev_mres: i32,
    pub status: i32,
    pub orig_ret: *mut c_void,
    pub override_ret: *mut c_void,
}

#[repr(C)]
pub struct mutil_funcs_t {
    pub pfnLogConsole: Option<unsafe extern "C" fn(*const plugin_info_t, *const c_char)>,
    pub pfnLogMessage: Option<unsafe extern "C" fn(*const plugin_info_t, *const c_char)>,
    pub pfnLogError: Option<unsafe extern "C" fn(*const plugin_info_t, *const c_char)>,
    pub pfnLogDeveloper: Option<unsafe extern "C" fn(*const plugin_info_t, *const c_char)>,
    pub pfnCenterSay: Option<unsafe extern "C" fn(*const plugin_info_t, *const c_char)>,
    pub pfnCenterSayParms:
        Option<unsafe extern "C" fn(*const plugin_info_t, *const c_void, *const c_char)>,
    pub pfnCenterSayVarargs: Option<
        unsafe extern "C" fn(*const plugin_info_t, *const c_void, *const c_char, *mut c_void),
    >,
    pub pfnCallGameEntity:
        Option<unsafe extern "C" fn(*const plugin_info_t, *const c_char, *mut c_void) -> i32>,
    pub pfnGetUserMsgID:
        Option<unsafe extern "C" fn(*const plugin_info_t, *const c_char, *mut i32) -> i32>,
    pub pfnGetUserMsgName:
        Option<unsafe extern "C" fn(*const plugin_info_t, i32, *mut i32) -> *const c_char>,
    pub pfnGetPluginPath: Option<unsafe extern "C" fn(*const plugin_info_t) -> *const c_char>,
    pub pfnGetGameInfo: Option<unsafe extern "C" fn(*const plugin_info_t, i32) -> *const c_char>,
    pub _padding: [usize; 24], // Reserve space for remaining functions
}

/// META_FUNCTIONS struct from Metamod API.
/// This is filled in Meta_Attach and passed back to Metamod.
#[repr(C)]
#[allow(non_snake_case)]
pub struct meta_function_t {
    pub pfnGetEntityAPI: Option<unsafe extern "C" fn(*mut goldsrc_sys::DLL_FUNCTIONS, i32) -> i32>,
    pub pfnGetEntityAPI_Post:
        Option<unsafe extern "C" fn(*mut goldsrc_sys::DLL_FUNCTIONS, i32) -> i32>,
    pub pfnGetEntityAPI2:
        Option<unsafe extern "C" fn(*mut goldsrc_sys::DLL_FUNCTIONS, *mut i32) -> i32>,
    pub pfnGetEntityAPI2_Post:
        Option<unsafe extern "C" fn(*mut goldsrc_sys::DLL_FUNCTIONS, *mut i32) -> i32>,
    pub pfnGetNewDLLFunctions: Option<unsafe extern "C" fn(*mut c_void, *mut i32) -> i32>,
    pub pfnGetNewDLLFunctions_Post: Option<unsafe extern "C" fn(*mut c_void, *mut i32) -> i32>,
    pub pfnGetEngineFunctions:
        Option<unsafe extern "C" fn(*mut goldsrc_sys::enginefuncs_t, *mut i32) -> i32>,
    pub pfnGetEngineFunctions_Post:
        Option<unsafe extern "C" fn(*mut goldsrc_sys::enginefuncs_t, *mut i32) -> i32>,
}
