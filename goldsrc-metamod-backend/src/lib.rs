//! Metamod backend implementation for GoldSrc.rs.
//!
//! This crate implements the `Engine` trait using the Metamod API.
//! It compiles as a `.dll`/`.so` plugin for classic Metamod-r.

#![allow(static_mut_refs)]

use goldsrc_api::{Engine, Entity, Player};
use std::ffi::CString;

// SAFETY: These are written once during DLL initialization (GiveFnptrsToDll)
// and only read afterwards from the game thread. No concurrent access.
static mut G_ENGFUNCS: Option<goldsrc_sys::enginefuncs_t> = None;
static mut G_GLOBALS: Option<goldsrc_sys::globalvars_t> = None;

/// Initialize the backend with engine functions and global variables.
///
/// # Safety
/// Must be called exactly once from `GiveFnptrsToDll`.
pub unsafe fn init_backend(
    engfuncs: *const goldsrc_sys::enginefuncs_t,
    globals: *const goldsrc_sys::globalvars_t,
) {
    // SAFETY: Called once during initialization, before any reads.
    unsafe {
        if !engfuncs.is_null() {
            G_ENGFUNCS = Some(*engfuncs);
        }
        if !globals.is_null() {
            G_GLOBALS = Some(*globals);
        }
    }
}

/// Get the engine functions.
pub fn engfuncs() -> &'static goldsrc_sys::enginefuncs_t {
    // SAFETY: After init_backend, the value is only read, never modified.
    unsafe { G_ENGFUNCS.as_ref().expect("Backend not initialized") }
}

/// Get the global variables.
pub fn globals() -> &'static goldsrc_sys::globalvars_t {
    // SAFETY: After init_backend, the value is only read, never modified.
    unsafe { G_GLOBALS.as_ref().expect("Backend not initialized") }
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
        // SAFETY: Called from the game thread with valid engine functions.
        unsafe {
            let funcs = engfuncs();
            let edict = (funcs.pfnCreateEntity)?();
            if edict.is_null() {
                return None;
            }
            let cname = CString::new(classname).unwrap_or_default();
            call_engfunc!(funcs.pfnSetModel, edict, cname.as_ptr());
            Some(Entity { index: 0, edict })
        }
    }

    fn get_player(&self, index: i32) -> Option<Player> {
        // TODO: Get player edict by index
        let _ = index;
        None
    }

    fn server_print(&self, message: &str) {
        // SAFETY: Called from the game thread with valid engine functions.
        unsafe {
            let msg = CString::new(message).unwrap_or_default();
            call_engfunc!(engfuncs().pfnServerPrint, msg.as_ptr());
        }
    }

    fn server_command(&self, command: &str) {
        // SAFETY: Called from the game thread with valid engine functions.
        unsafe {
            let cmd = CString::new(command).unwrap_or_default();
            call_engfunc!(engfuncs().pfnServerCommand, cmd.as_ptr());
        }
    }

    fn cvar_get_float(&self, name: &str) -> f32 {
        // SAFETY: Called from the game thread with valid engine functions.
        unsafe {
            let cname = CString::new(name).unwrap_or_default();
            call_engfunc_ret!(engfuncs().pfnCVarGetFloat, cname.as_ptr())
        }
    }

    fn cvar_set_float(&self, name: &str, value: f32) {
        // SAFETY: Called from the game thread with valid engine functions.
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
