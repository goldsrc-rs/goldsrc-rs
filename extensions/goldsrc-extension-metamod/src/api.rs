//! Safe, ergonomic Metamod API abstractions.

use crate::types::*;
use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicPtr, Ordering};

static G_META_GLOBALS: AtomicPtr<meta_globals_t> = AtomicPtr::new(std::ptr::null_mut());
static G_META_UTIL: AtomicPtr<mutil_funcs_t> = AtomicPtr::new(std::ptr::null_mut());
static G_PLUGIN_INFO: AtomicPtr<plugin_info_t> = AtomicPtr::new(std::ptr::null_mut());

/// Safe, ergonomic API facade for Metamod operations.
pub struct MetamodApi;

impl MetamodApi {
    /// Returns `true` if Metamod is active and pointers have been initialized.
    #[inline]
    pub fn is_available() -> bool {
        !G_META_GLOBALS.load(Ordering::Relaxed).is_null()
    }

    /// Sets the Metamod return status code (e.g. `MRES_SUPERCEDE`, `MRES_OVERRIDE`, `MRES_HANDLED`, `MRES_IGNORED`).
    #[inline]
    pub fn set_result(mres: i32) {
        let ptr = G_META_GLOBALS.load(Ordering::Relaxed);
        if !ptr.is_null() {
            // SAFETY: Valid pointer supplied by Metamod host in Meta_Attach.
            unsafe {
                (*ptr).mres = mres;
            }
        }
    }

    /// Returns the current Metamod return status code.
    #[inline]
    pub fn get_result() -> i32 {
        let ptr = G_META_GLOBALS.load(Ordering::Relaxed);
        if !ptr.is_null() {
            // SAFETY: Valid pointer supplied by Metamod host in Meta_Attach.
            unsafe { (*ptr).mres }
        } else {
            MRES_UNSET
        }
    }

    /// Reads the original return value pointer from Metamod globals.
    #[inline]
    pub fn orig_ret<T>() -> Option<*const T> {
        let ptr = G_META_GLOBALS.load(Ordering::Relaxed);
        if !ptr.is_null() {
            // SAFETY: Valid pointer supplied by Metamod host.
            let ret = unsafe { (*ptr).orig_ret as *const T };
            if !ret.is_null() {
                return Some(ret);
            }
        }
        None
    }

    /// Reads and dereferences the original return value of type `T: Copy`.
    #[inline]
    pub fn orig_ret_val<T: Copy>() -> Option<T> {
        let ptr = Self::orig_ret::<T>()?;
        // SAFETY: Non-null pointer to return value supplied by Metamod hook post-call.
        Some(unsafe { *ptr })
    }

    /// Sets an override return value pointer into Metamod globals.
    ///
    /// # Safety
    /// `ret` must be a valid pointer or null.
    #[inline]
    pub unsafe fn set_override_ret<T>(ret: *mut T) {
        let ptr = G_META_GLOBALS.load(Ordering::Relaxed);
        if !ptr.is_null() {
            // SAFETY: Valid pointer supplied by Metamod host.
            unsafe {
                (*ptr).override_ret = ret as *mut _;
            }
        }
    }

    /// Resolves an engine user message ID by message name (e.g. "TextMsg", "ShowMenu").
    pub fn get_user_msg_id(name: &str) -> Option<i32> {
        let util_ptr = G_META_UTIL.load(Ordering::Relaxed);
        let pl_ptr = G_PLUGIN_INFO.load(Ordering::Relaxed);
        if util_ptr.is_null() || pl_ptr.is_null() {
            return None;
        }
        // SAFETY: Valid Metamod utility function pointer and plugin descriptor.
        unsafe {
            if let Some(get_msg_id) = (*util_ptr).pfnGetUserMsgID
                && let Ok(cname) = CString::new(name)
            {
                let mut size: i32 = 0;
                let id = get_msg_id(pl_ptr, cname.as_ptr(), &mut size as *mut _);
                if id > 0 && id != 255 {
                    return Some(id);
                }
            }
        }
        None
    }

    /// Resolves user message name by message ID.
    pub fn get_user_msg_name(msg_id: i32) -> Option<&'static str> {
        let util_ptr = G_META_UTIL.load(Ordering::Relaxed);
        let pl_ptr = G_PLUGIN_INFO.load(Ordering::Relaxed);
        if util_ptr.is_null() || pl_ptr.is_null() {
            return None;
        }
        // SAFETY: Valid Metamod utility function pointer.
        unsafe {
            if let Some(get_msg_name) = (*util_ptr).pfnGetUserMsgName {
                let mut size: i32 = 0;
                let ptr = get_msg_name(pl_ptr, msg_id, &mut size as *mut _);
                if !ptr.is_null() {
                    return CStr::from_ptr(ptr).to_str().ok();
                }
            }
        }
        None
    }

    /// Dispatches entity instantiation/thinking to GameDLL entity tables via Metamod.
    ///
    /// # Safety
    /// `pev` must point to an initialized `entvars_t` structure or null.
    pub unsafe fn call_game_entity(
        ent_type: &str,
        pev: *mut goldsrc_sys::entvars_t,
    ) -> Result<i32, &'static str> {
        let util_ptr = G_META_UTIL.load(Ordering::Relaxed);
        let pl_ptr = G_PLUGIN_INFO.load(Ordering::Relaxed);
        if util_ptr.is_null() || pl_ptr.is_null() {
            return Err("Metamod utility functions not initialized");
        }
        // SAFETY: Caller guarantees pev is valid; pointers were verified non-null.
        unsafe {
            if let Some(call_entity) = (*util_ptr).pfnCallGameEntity
                && let Ok(c_type) = CString::new(ent_type)
            {
                Ok(call_entity(pl_ptr, c_type.as_ptr(), pev))
            } else {
                Err("pfnCallGameEntity not available or invalid entity classname")
            }
        }
    }

    /// Returns the filesystem path to the current plugin DLL.
    pub fn get_plugin_path() -> Option<&'static str> {
        let util_ptr = G_META_UTIL.load(Ordering::Relaxed);
        let pl_ptr = G_PLUGIN_INFO.load(Ordering::Relaxed);
        if util_ptr.is_null() || pl_ptr.is_null() {
            return None;
        }
        // SAFETY: Valid Metamod utility function pointer returning null-terminated static C string.
        unsafe {
            if let Some(get_path) = (*util_ptr).pfnGetPluginPath {
                let ptr = get_path(pl_ptr);
                if !ptr.is_null() {
                    return CStr::from_ptr(ptr).to_str().ok();
                }
            }
        }
        None
    }

    /// Returns game information string (e.g. gamedir, game description).
    pub fn get_game_info(tag: i32) -> Option<&'static str> {
        let util_ptr = G_META_UTIL.load(Ordering::Relaxed);
        let pl_ptr = G_PLUGIN_INFO.load(Ordering::Relaxed);
        if util_ptr.is_null() || pl_ptr.is_null() {
            return None;
        }
        // SAFETY: Valid Metamod utility function pointer returning static C string.
        unsafe {
            if let Some(get_info) = (*util_ptr).pfnGetGameInfo {
                let ptr = get_info(pl_ptr, tag);
                if !ptr.is_null() {
                    return CStr::from_ptr(ptr).to_str().ok();
                }
            }
        }
        None
    }

    /// Directly accesses raw Metamod globals if low-level manipulation is required.
    pub fn raw_meta_globals() -> Option<&'static mut meta_globals_t> {
        let ptr = G_META_GLOBALS.load(Ordering::Relaxed);
        if !ptr.is_null() {
            // SAFETY: Valid pointer supplied by Metamod host.
            Some(unsafe { &mut *ptr })
        } else {
            None
        }
    }

    /// Directly accesses raw Metamod utility table if low-level manipulation is required.
    pub fn raw_meta_util() -> Option<&'static mut mutil_funcs_t> {
        let ptr = G_META_UTIL.load(Ordering::Relaxed);
        if !ptr.is_null() {
            // SAFETY: Valid pointer supplied by Metamod host.
            Some(unsafe { &mut *ptr })
        } else {
            None
        }
    }
}

/// Sets the Metamod utility functions table and plugin info descriptor.
pub fn set_meta_util_funcs(util: *mut mutil_funcs_t, plinfo: *const plugin_info_t) {
    G_META_UTIL.store(util, Ordering::Relaxed);
    G_PLUGIN_INFO.store(plinfo as *mut _, Ordering::Relaxed);
}

/// Sets the Metamod globals structure pointer.
pub fn set_meta_globals(globals: *mut meta_globals_t) {
    G_META_GLOBALS.store(globals, Ordering::Relaxed);
}
