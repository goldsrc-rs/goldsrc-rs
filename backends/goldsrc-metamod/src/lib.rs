//! Metamod backend implementation for GoldSrc.rs.

#![allow(static_mut_refs)]

// ============================================================================
// MSVC Linker Directives
//
// On i686-pc-windows-msvc, __stdcall functions are decorated by the linker as
// `_FunctionName@N` (where N is the argument byte count). The GoldSrc engine
// looks for undecorated names (e.g. `GiveFnptrsToDll`), so we must instruct
// the linker to export the decorated symbol under its clean name.
//
// This replaces the legacy `exports.c` / `#pragma comment(linker, ...)` C file.
// The `.drectve` section is the standard COFF mechanism for embedding linker
// directives directly inside an object file — exactly what the C pragma did.
// ============================================================================
#[cfg(all(target_arch = "x86", target_env = "msvc"))]
#[unsafe(link_section = ".drectve")]
#[used]
static MSVC_EXPORTS: [u8; 204] = *b"\
 /EXPORT:GiveFnptrsToDll=_GiveFnptrsToDll@8\
 /EXPORT:Meta_Query=_Meta_Query\
 /EXPORT:Meta_Attach=_Meta_Attach\
 /EXPORT:Meta_Detach=_Meta_Detach\
 /EXPORT:GetEntityAPI2=_GetEntityAPI2\
 /EXPORT:GetEntityAPI2_Post=_GetEntityAPI2_Post\
 /EXPORT:GetNewDLLFunctions=_GetNewDLLFunctions\
";

mod commands;
mod entrypoints;
mod hooks;
mod meta_types;
mod vtable;

pub use vtable::*;

use goldsrc::logging::LogTarget;
use goldsrc_api::Engine;
use std::ffi::CString;

use meta_types::*;

static mut G_ENGFUNCS: Option<goldsrc_sys::enginefuncs_t> = None;
static mut G_GLOBALS: Option<goldsrc_sys::globalvars_t> = None;
static mut G_META_GLOBALS: Option<*mut meta_globals_t> = None;
static mut HOST_RUNTIME: Option<goldsrc::host::HostRuntime> = None;

pub static PRINT_QUEUE: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// Initialize WASM plugin subsystem and the unified logger.
pub fn init_wasm_host() {
    match goldsrc::host::HostRuntime::init("metamod", |msg| {
        backend().server_print(msg);
    }) {
        Ok(runtime) => unsafe {
            HOST_RUNTIME = Some(runtime);
        },
        Err(e) => {
            goldsrc::gslog_error!(LogTarget::Core, "{e}");
        }
    }
}

pub fn wasm_manager() -> Option<&'static mut goldsrc_wasm_host::PluginManager> {
    unsafe { HOST_RUNTIME.as_mut().map(|r| r.manager_mut()) }
}

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

pub fn set_meta_globals(ptr: *mut meta_globals_t) {
    unsafe { G_META_GLOBALS = Some(ptr) };
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
    ($func:expr) => {
        if let Some(f) = $func {
            f()
        } else {
            Default::default()
        }
    };
    ($func:expr, $($arg:expr),*) => {
        if let Some(f) = $func {
            f($($arg),*)
        } else {
            Default::default()
        }
    };
}

pub(crate) use call_engfunc;
pub(crate) use call_engfunc_ret;

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
        // Defer printing to StartFrame_Post to avoid engine instability during StartFrame.
        if let Ok(mut queue) = PRINT_QUEUE.lock() {
            queue.push(message.to_string());
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

use std::fs::OpenOptions;
use std::io::Write;

pub fn file_log(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(goldsrc::paths::PathResolver::debug_log_path())
    {
        let _ = writeln!(file, "{}", msg);
    }
}

static BACKEND: MetamodBackend = MetamodBackend::new();

pub fn backend() -> &'static MetamodBackend {
    &BACKEND
}

pub use entrypoints::{
    alert, GetEngineFunctions, GetEngineFunctions_Post, GetEntityAPI, GetEntityAPI2,
    GetEntityAPI2_Post, GetEntityAPI_Post, GetNewDLLFunctions, GetNewDLLFunctions_Post,
    GiveFnptrsToDll, Meta_Attach, Meta_Detach, Meta_Query,
};
