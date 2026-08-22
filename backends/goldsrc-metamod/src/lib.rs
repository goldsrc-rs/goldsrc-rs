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
static MSVC_EXPORTS: [u8; 468] = *b"/EXPORT:GiveFnptrsToDll=_GiveFnptrsToDll@8 /EXPORT:Meta_Query=_Meta_Query /EXPORT:Meta_Attach=_Meta_Attach /EXPORT:Meta_Detach=_Meta_Detach /EXPORT:GetEntityAPI2=_GetEntityAPI2 /EXPORT:GetEntityAPI2_Post=_GetEntityAPI2_Post /EXPORT:GetNewDLLFunctions=_GetNewDLLFunctions /EXPORT:GetEntityAPI=_GetEntityAPI /EXPORT:GetEntityAPI_Post=_GetEntityAPI_Post /EXPORT:GetNewDLLFunctions_Post=_GetNewDLLFunctions_Post /EXPORT:GetEngineFunctions=_GetEngineFunctions /EXPORT:GetEngineFunctions_Post=_GetEngineFunctions_Post";

mod commands;
mod entrypoints;
mod hooks;
mod meta_types;

use goldsrc::log;

use meta_types::*;

static G_ENGFUNCS: std::sync::OnceLock<
    goldsrc_sys::ffi::SyncWrapper<&'static goldsrc_sys::enginefuncs_t>,
> = std::sync::OnceLock::new();
static G_GLOBALS: std::sync::OnceLock<
    goldsrc_sys::ffi::SyncWrapper<&'static goldsrc_sys::globalvars_t>,
> = std::sync::OnceLock::new();
static G_META_GLOBALS: std::sync::atomic::AtomicPtr<meta_globals_t> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

/// Deferred server-print queue shared with the standalone backend.
pub static PRINT_QUEUE: goldsrc::backend::PrintQueue = goldsrc::backend::PrintQueue::new();

/// Initialize WASM plugin subsystem and the unified logger.
pub fn init_wasm_host() {
    let engine: std::sync::Arc<dyn goldsrc_api::Engine> =
        std::sync::Arc::new(goldsrc::backend::EngineBackend::new(engfuncs, &PRINT_QUEUE));
    if let Err(e) = goldsrc::host::HostRuntime::init(
        goldsrc_api::consts::BackendType::Metamod,
        |msg| {
            backend().server_print(msg);
        },
        engine,
    ) {
        log::error!(target: "core", "{e}");
    }
}

/// # Safety
/// Called once from `GiveFnptrsToDll`.
pub unsafe fn init_backend(
    engfuncs: *mut goldsrc_sys::enginefuncs_t,
    globals: *mut goldsrc_sys::globalvars_t,
) {
    if !engfuncs.is_null() {
        // SAFETY: engfuncs is checked for null
        let _ = G_ENGFUNCS.set(goldsrc_sys::ffi::SyncWrapper::new(unsafe { &*engfuncs }));
    }
    if !globals.is_null() {
        // SAFETY: globals is checked for null
        let _ = G_GLOBALS.set(goldsrc_sys::ffi::SyncWrapper::new(unsafe { &*globals }));
    }
}

pub fn engfuncs() -> &'static goldsrc_sys::enginefuncs_t {
    G_ENGFUNCS.get().expect("Backend not initialized")
}

pub fn globals() -> &'static goldsrc_sys::globalvars_t {
    G_GLOBALS.get().expect("Backend not initialized")
}

pub fn meta_globals() -> &'static mut meta_globals_t {
    let ptr = G_META_GLOBALS.load(std::sync::atomic::Ordering::Relaxed);
    if ptr.is_null() {
        panic!("Meta globals not initialized");
    }
    unsafe { &mut *ptr }
}

pub fn set_meta_globals(ptr: *mut meta_globals_t) {
    G_META_GLOBALS.store(ptr, std::sync::atomic::Ordering::Relaxed);
}

use goldsrc::backend::EngineBackend;

/// Metamod backend: the shared `EngineBackend` fed by this crate's engfunc
/// accessor and print queue. The backend is a thin adapter.
pub type MetamodBackend = EngineBackend;

pub use goldsrc::call_engfunc;
pub use goldsrc::call_engfunc_ret;

static BACKEND: MetamodBackend = EngineBackend::new(engfuncs, &PRINT_QUEUE);

pub fn backend() -> &'static MetamodBackend {
    &BACKEND
}

pub use entrypoints::{
    GetEngineFunctions, GetEngineFunctions_Post, GetEntityAPI, GetEntityAPI_Post, GetEntityAPI2,
    GetEntityAPI2_Post, GetNewDLLFunctions, GetNewDLLFunctions_Post, GiveFnptrsToDll, Meta_Attach,
    Meta_Detach, Meta_Query,
};
