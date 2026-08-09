//! Metamod type definitions (manually defined to avoid C++ bindgen issues).

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;

/// Metamod interface version.
pub const META_INTERFACE_VERSION: &str = "5:13";

/// Plugin load time flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PLUG_LOADTIME {
    PT_NEVER = 0,
    PT_STARTUP,
    PT_CHANGELEVEL,
    PT_ANYTIME,
    PT_ANYPAUSE,
}

/// Plugin unload reason.
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

/// Plugin info structure.
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

// SAFETY: plugin_info_t is only read, never modified after initialization.
unsafe impl Sync for plugin_info_t {}

/// Meta result flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum META_RES {
    MRES_UNSET = 0,
    MRES_IGNORED,
    MRES_HANDLED,
    MRES_OVERRIDE,
    MRES_SUPERCEDE,
}

/// Meta globals.
#[repr(C)]
pub struct meta_globals_t {
    pub mres: META_RES,
    pub prev_mres: META_RES,
    pub status: META_RES,
    pub orig_ret: *mut c_void,
    pub override_ret: *mut c_void,
}

/// Metamod utility functions.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct mutil_funcs_t {
    pub pfnLogConsole: Option<unsafe extern "C" fn(plid: *const plugin_info_t, fmt: *const c_char)>,
    pub pfnLogMessage: Option<unsafe extern "C" fn(plid: *const plugin_info_t, fmt: *const c_char)>,
    pub pfnLogError: Option<unsafe extern "C" fn(plid: *const plugin_info_t, fmt: *const c_char)>,
    pub pfnLogDeveloper:
        Option<unsafe extern "C" fn(plid: *const plugin_info_t, fmt: *const c_char)>,
    pub pfnCenterSay: Option<unsafe extern "C" fn(plid: *const plugin_info_t, fmt: *const c_char)>,
    pub pfnCenterSayParms: Option<
        unsafe extern "C" fn(plid: *const plugin_info_t, tparms: *mut c_void, fmt: *const c_char),
    >,
    pub pfnCenterSayVarargs: Option<
        unsafe extern "C" fn(
            plid: *const plugin_info_t,
            tparms: *mut c_void,
            fmt: *const c_char,
            args: *mut c_void,
        ),
    >,
    pub pfnCallGameEntity: Option<
        unsafe extern "C" fn(
            plid: *const plugin_info_t,
            entStr: *const c_char,
            pev: *mut c_void,
        ) -> c_int,
    >,
    pub pfnGetUserMsgID: Option<
        unsafe extern "C" fn(
            plid: *const plugin_info_t,
            msgname: *const c_char,
            size: *mut c_int,
        ) -> c_int,
    >,
    pub pfnGetUserMsgName: Option<
        unsafe extern "C" fn(
            plid: *const plugin_info_t,
            msgid: c_int,
            size: *mut c_int,
        ) -> *const c_char,
    >,
    pub pfnGetPluginPath: Option<unsafe extern "C" fn(plid: *const plugin_info_t) -> *const c_char>,
    pub pfnGetGameInfo:
        Option<unsafe extern "C" fn(plid: *const plugin_info_t, tag: c_int) -> *const c_char>,
    pub pfnLoadPlugin: Option<
        unsafe extern "C" fn(
            plid: *const plugin_info_t,
            cmdline: *const c_char,
            now: PLUG_LOADTIME,
            plugin_handle: *mut *mut c_void,
        ) -> c_int,
    >,
    pub pfnUnloadPlugin: Option<
        unsafe extern "C" fn(
            plid: *const plugin_info_t,
            cmdline: *const c_char,
            now: PLUG_LOADTIME,
            reason: PL_UNLOAD_REASON,
        ) -> c_int,
    >,
    pub pfnUnloadPluginByHandle: Option<
        unsafe extern "C" fn(
            plid: *const plugin_info_t,
            plugin_handle: *mut c_void,
            now: PLUG_LOADTIME,
            reason: PL_UNLOAD_REASON,
        ) -> c_int,
    >,
    pub pfnIsQueryingClientCvar: Option<
        unsafe extern "C" fn(
            plid: *const plugin_info_t,
            pEdict: *const goldsrc_sys::edict_t,
        ) -> *const c_char,
    >,
    pub pfnMakeRequestId: Option<unsafe extern "C" fn(plid: *const plugin_info_t) -> c_int>,
    pub pfnGetHookTables: Option<
        unsafe extern "C" fn(
            plid: *const plugin_info_t,
            peng: *mut *mut goldsrc_sys::enginefuncs_t,
            pdll: *mut *mut c_void,
            pnewdll: *mut *mut c_void,
        ),
    >,
}

/// Meta function table.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct meta_function_t {
    pub pfnGetEntityAPI:
        Option<unsafe extern "C" fn(pFunctionTable: *mut c_void, interfaceVersion: c_int) -> c_int>,
    pub pfnGetEntityAPI_Post:
        Option<unsafe extern "C" fn(pFunctionTable: *mut c_void, interfaceVersion: c_int) -> c_int>,
    pub pfnGetEntityAPI2: Option<
        unsafe extern "C" fn(pFunctionTable: *mut c_void, interfaceVersion: *mut c_int) -> c_int,
    >,
    pub pfnGetEntityAPI2_Post: Option<
        unsafe extern "C" fn(pFunctionTable: *mut c_void, interfaceVersion: *mut c_int) -> c_int,
    >,
    pub pfnGetNewDLLFunctions: Option<
        unsafe extern "C" fn(pFunctionTable: *mut c_void, interfaceVersion: *mut c_int) -> c_int,
    >,
    pub pfnGetNewDLLFunctions_Post: Option<
        unsafe extern "C" fn(pFunctionTable: *mut c_void, interfaceVersion: *mut c_int) -> c_int,
    >,
    pub pfnGetEngineFunctions: Option<
        unsafe extern "C" fn(pFunctionTable: *mut c_void, interfaceVersion: *mut c_int) -> c_int,
    >,
    pub pfnGetEngineFunctions_Post: Option<
        unsafe extern "C" fn(pFunctionTable: *mut c_void, interfaceVersion: *mut c_int) -> c_int,
    >,
}
