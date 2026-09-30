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

static DUMMY_ENGFUNCS: enginefuncs_t = unsafe { std::mem::zeroed() };
static DUMMY_GLOBALS: goldsrc_sys::ffi::SyncWrapper<goldsrc_sys::globalvars_t> =
    goldsrc_sys::ffi::SyncWrapper::new(unsafe { std::mem::zeroed() });

/// Returns the current engine functions table, falling back to an empty dummy if uninitialized.
pub fn engfuncs() -> &'static enginefuncs_t {
    G_ENGFUNCS.get().map(|w| **w).unwrap_or(&DUMMY_ENGFUNCS)
}

/// Tries to return the engine functions table without panicking.
pub fn try_engfuncs() -> Option<&'static enginefuncs_t> {
    G_ENGFUNCS.get().map(|w| **w)
}

/// Returns the global variables table, falling back to an empty dummy if uninitialized.
#[allow(dead_code)]
pub fn globals() -> &'static goldsrc_sys::globalvars_t {
    G_GLOBALS.get().map(|w| **w).unwrap_or(&*DUMMY_GLOBALS)
}

/// Tries to return the global variables table without panicking.
pub fn try_globals() -> Option<&'static goldsrc_sys::globalvars_t> {
    G_GLOBALS.get().map(|w| **w)
}
