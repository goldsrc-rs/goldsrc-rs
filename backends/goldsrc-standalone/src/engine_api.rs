//! GoldSrc.rs Standalone Backend — engine API detection at runtime.
//!
//! Detects and wraps the available engine API tier:
//! - **ReHLDS** (`IRehldsApi`) — extended API with extra hooks.
//! - **Vanilla** (`enginefuncs_t`) — standard HLSDK engine functions.
//!
//! ReGameDLL detection is handled separately in `proxy.rs`.

use goldsrc_sys::enginefuncs_t;

/// Unified engine API abstraction resolved at runtime.
pub enum EngineApiTier {
    /// ReHLDS detected: extended API available.
    Rehlds,
    /// Vanilla HLDS/ReHLDS without extended API.
    Vanilla,
}

/// Detected tier for the current server environment.
static mut ENGINE_TIER: EngineApiTier = EngineApiTier::Vanilla;
static mut G_ENGFUNCS: Option<enginefuncs_t> = None;
static mut G_GLOBALS: Option<goldsrc_sys::globalvars_t> = None;

/// Initialize engine functions received from the engine on DLL load.
///
/// # Safety
/// `engfuncs` and `globals` must be valid pointers provided by the engine.
pub unsafe fn init(engfuncs: *mut enginefuncs_t, globals: *mut goldsrc_sys::globalvars_t) {
    unsafe {
        if !engfuncs.is_null() {
            G_ENGFUNCS = Some(*engfuncs);
        }
        if !globals.is_null() {
            G_GLOBALS = Some(*globals);
        }

        // Attempt ReHLDS detection via the exported symbol `RehldsApi`.
        // ReHLDS exports this from `swds.dll` (Windows) / `engine_i486.so` (Linux).
        let tier = detect_rehlds();
        ENGINE_TIER = tier;
    }
}

/// Returns the current engine functions table.
///
/// # Panics
/// Panics if called before `init`.
pub fn engfuncs() -> &'static enginefuncs_t {
    unsafe {
        G_ENGFUNCS
            .as_ref()
            .expect("[GoldSrc.rs Standalone] Engine not initialized")
    }
}

/// Returns the global variables table.
///
/// # Panics
/// Panics if called before `init`.
#[allow(dead_code)]
pub fn globals() -> &'static goldsrc_sys::globalvars_t {
    unsafe {
        G_GLOBALS
            .as_ref()
            .expect("[GoldSrc.rs Standalone] Globals not initialized")
    }
}

/// Returns a string description of the detected engine tier.
pub fn tier_name() -> &'static str {
    unsafe {
        match ENGINE_TIER {
            EngineApiTier::Rehlds => "ReHLDS (extended API)",
            EngineApiTier::Vanilla => "Vanilla HLDS (standard API)",
        }
    }
}

/// Attempt to detect ReHLDS by looking for its exported interface symbol.
///
/// On Windows, ReHLDS exports `RehldsApi` from `swds.dll`.
/// On Linux, it exports it from `engine_i486.so`.
fn detect_rehlds() -> EngineApiTier {
    // Use platform-specific symbol lookup to check if ReHLDS is loaded.
    #[cfg(target_os = "windows")]
    {
        use std::ffi::CString;
        // SAFETY: We query an already-loaded DLL via its module handle.
        unsafe {
            let module_name = CString::new("swds.dll").unwrap();
            let module = windows_get_module_handle(module_name.as_ptr());
            if !module.is_null() {
                let sym_name = CString::new("RehldsApi").unwrap();
                let sym = windows_get_proc_address(module, sym_name.as_ptr());
                if !sym.is_null() {
                    return EngineApiTier::Rehlds;
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, use dlopen with RTLD_NOLOAD to check without loading.
        // SAFETY: We never actually open a new lib — RTLD_NOLOAD returns null if not loaded.
        unsafe {
            let handle = libc_dlopen(
                b"engine_i486.so\0".as_ptr() as *const std::os::raw::c_char,
                0x2 | 0x1000, // RTLD_NOW | RTLD_NOLOAD
            );
            if !handle.is_null() {
                let sym = libc_dlsym(
                    handle,
                    b"RehldsApi\0".as_ptr() as *const std::os::raw::c_char,
                );
                libc_dlclose(handle);
                if !sym.is_null() {
                    return EngineApiTier::Rehlds;
                }
            }
        }
    }

    EngineApiTier::Vanilla
}

// ============================================================================
// Platform-specific symbol lookup helpers
// ============================================================================

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    #[link_name = "GetModuleHandleA"]
    fn windows_get_module_handle(
        lp_module_name: *const std::os::raw::c_char,
    ) -> *mut std::ffi::c_void;
    #[link_name = "GetProcAddress"]
    fn windows_get_proc_address(
        h_module: *mut std::ffi::c_void,
        lp_proc_name: *const std::os::raw::c_char,
    ) -> *mut std::ffi::c_void;
}

#[cfg(target_os = "linux")]
extern "C" {
    #[link_name = "dlopen"]
    fn libc_dlopen(filename: *const std::os::raw::c_char, flags: i32) -> *mut std::ffi::c_void;
    #[link_name = "dlsym"]
    fn libc_dlsym(
        handle: *mut std::ffi::c_void,
        symbol: *const std::os::raw::c_char,
    ) -> *mut std::ffi::c_void;
    #[link_name = "dlclose"]
    fn libc_dlclose(handle: *mut std::ffi::c_void) -> i32;
}
