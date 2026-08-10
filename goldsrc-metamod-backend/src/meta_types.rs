//! Metamod type definitions (manually defined to avoid C++ bindgen issues).

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_void;
use std::ffi::c_char;

pub const META_INTERFACE_VERSION: &str = "5:13";

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
    pub _padding: [usize; 12], // Reserve space for remaining functions we don't use
}
