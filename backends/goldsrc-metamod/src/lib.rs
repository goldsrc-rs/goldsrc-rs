//! Metamod backend implementation for GoldSrc.rs.

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
static MSVC_EXPORTS: [u8; 270] = *b"/EXPORT:GiveFnptrsToDll=_GiveFnptrsToDll@8 /EXPORT:Meta_Query=_Meta_Query /EXPORT:Meta_Attach=_Meta_Attach /EXPORT:Meta_Detach=_Meta_Detach /EXPORT:GetEntityAPI2=_GetEntityAPI2 /EXPORT:GetEntityAPI2_Post=_GetEntityAPI2_Post /EXPORT:GetNewDLLFunctions=_GetNewDLLFunctions";

mod commands;
mod entrypoints;
mod hooks;
mod meta_types;
mod vtable;

pub use vtable::*;

use goldsrc::logging::LogTarget;
use goldsrc_api::Engine;

use meta_types::*;

static G_ENGFUNCS: std::sync::OnceLock<
    goldsrc_sys::ffi::SyncWrapper<&'static goldsrc_sys::enginefuncs_t>,
> = std::sync::OnceLock::new();
static G_GLOBALS: std::sync::OnceLock<
    goldsrc_sys::ffi::SyncWrapper<&'static goldsrc_sys::globalvars_t>,
> = std::sync::OnceLock::new();
thread_local! {
    static G_META_GLOBALS: std::cell::RefCell<Option<*mut meta_globals_t>> = const {
        std::cell::RefCell::new(None)
    };
}

/// Deferred server-print queue shared with the standalone backend.
pub static PRINT_QUEUE: goldsrc::backend::PrintQueue = goldsrc::backend::PrintQueue::new();

/// Initialize WASM plugin subsystem and the unified logger.
pub fn init_wasm_host() {
    if let Err(e) = goldsrc::host::HostRuntime::init("metamod", |msg| {
        backend().server_print(msg);
    }) {
        goldsrc::gslog_error!(LogTarget::Core, "{e}");
    }
}

/// # Safety
/// Called once from `GiveFnptrsToDll`.
pub unsafe fn init_backend(
    engfuncs: *mut goldsrc_sys::enginefuncs_t,
    globals: *mut goldsrc_sys::globalvars_t,
) {
    // Copy the engine-provided structs into leaked boxes; they are set once and
    // then only read, so a leaked 'static reference is sound and avoids the
    // aliasing UB of `static mut` accessors.
    if !engfuncs.is_null() {
        let leaked: &'static _ = Box::leak(Box::new(unsafe { *engfuncs }));
        let _ = G_ENGFUNCS.set(goldsrc_sys::ffi::SyncWrapper::new(leaked));
    }
    if !globals.is_null() {
        let leaked: &'static _ = Box::leak(Box::new(unsafe { *globals }));
        let _ = G_GLOBALS.set(goldsrc_sys::ffi::SyncWrapper::new(leaked));
    }
}

pub fn engfuncs() -> &'static goldsrc_sys::enginefuncs_t {
    G_ENGFUNCS.get().expect("Backend not initialized")
}

pub fn globals() -> &'static goldsrc_sys::globalvars_t {
    G_GLOBALS.get().expect("Backend not initialized")
}

pub fn meta_globals() -> &'static mut meta_globals_t {
    G_META_GLOBALS
        .with(|c| {
            let guard = c.borrow();
            // SAFETY: pointer set once by Metamod; single-threaded engine.
            unsafe { guard.map(|p| &mut *p) }
        })
        .expect("Meta globals not initialized")
}

pub fn set_meta_globals(ptr: *mut meta_globals_t) {
    G_META_GLOBALS.with(|c| {
        *c.borrow_mut() = Some(ptr);
    });
}

use goldsrc::backend::EngineBackend;

/// Metamod backend: the shared `EngineBackend` fed by this crate's engfunc
/// accessor and print queue. The backend is a thin adapter.
pub type MetamodBackend = EngineBackend;

pub use goldsrc::call_engfunc;
pub use goldsrc::call_engfunc_ret;

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

static BACKEND: MetamodBackend = EngineBackend::new(engfuncs, &PRINT_QUEUE);

pub fn backend() -> &'static MetamodBackend {
    &BACKEND
}

pub use entrypoints::{
    alert, GetEngineFunctions, GetEngineFunctions_Post, GetEntityAPI, GetEntityAPI2,
    GetEntityAPI2_Post, GetEntityAPI_Post, GetNewDLLFunctions, GetNewDLLFunctions_Post,
    GiveFnptrsToDll, Meta_Attach, Meta_Detach, Meta_Query,
};
