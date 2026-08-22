//! GoldSrc.rs Standalone Backend — engine API initialization.
//!
//! Standard HLSDK engine functions (`enginefuncs_t`).

use goldsrc_sys::enginefuncs_t;

static G_ENGFUNCS: std::sync::OnceLock<goldsrc_sys::ffi::SyncWrapper<&'static enginefuncs_t>> =
    std::sync::OnceLock::new();
static G_GLOBALS: std::sync::OnceLock<
    goldsrc_sys::ffi::SyncWrapper<&'static goldsrc_sys::globalvars_t>,
> = std::sync::OnceLock::new();

/// Initialize engine functions received from the engine on DLL load.
///
/// # Safety
/// `engfuncs` and `globals` must be valid pointers provided by the engine.
pub unsafe fn init(engfuncs: *mut enginefuncs_t, globals: *mut goldsrc_sys::globalvars_t) {
    if !engfuncs.is_null() {
        // SAFETY: engfuncs is checked for null
        let _ = G_ENGFUNCS.set(goldsrc_sys::ffi::SyncWrapper::new(unsafe { &*engfuncs }));
    }
    if !globals.is_null() {
        // SAFETY: globals is checked for null
        let _ = G_GLOBALS.set(goldsrc_sys::ffi::SyncWrapper::new(unsafe { &*globals }));
    }
}

/// Returns the current engine functions table.
///
/// # Panics
/// Panics if called before `init`.
pub fn engfuncs() -> &'static enginefuncs_t {
    G_ENGFUNCS
        .get()
        .expect("[GoldSrc.rs Standalone] Engine not initialized")
}

/// Returns the global variables table.
///
/// # Panics
/// Panics if called before `init`.
#[allow(dead_code)]
pub fn globals() -> &'static goldsrc_sys::globalvars_t {
    G_GLOBALS
        .get()
        .expect("[GoldSrc.rs Standalone] Globals not initialized")
}
