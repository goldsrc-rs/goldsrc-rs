//! Engine console variable (CVAR) operations and dynamic registration.

use super::EngineBackend;
use crate::{call_engfunc, call_engfunc_ret};
use goldsrc_api::cvar::CvarEngine;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub type MapNameResolverFn = fn() -> Option<String>;

static MAP_NAME_RESOLVER_FN: std::sync::OnceLock<MapNameResolverFn> = std::sync::OnceLock::new();

/// Sets a backend-specific resolver for querying the active map name.
pub fn set_map_name_resolver(resolver: MapNameResolverFn) {
    let _ = MAP_NAME_RESOLVER_FN.set(resolver);
}

impl CvarEngine for EngineBackend {
    fn cvar_get_float(&self, name: &str) -> f32 {
        unsafe {
            let cname = std::ffi::CString::new(name).unwrap_or_default();
            call_engfunc_ret!((self.engfuncs)().pfnCVarGetFloat, cname.as_ptr())
        }
    }

    fn cvar_set_float(&self, name: &str, val: f32) {
        unsafe {
            let cname = std::ffi::CString::new(name).unwrap_or_default();
            call_engfunc!((self.engfuncs)().pfnCVarSetFloat, cname.as_ptr(), val);
        }
    }

    fn cvar_get_string(&self, name: &str) -> Option<String> {
        unsafe {
            let cname = std::ffi::CString::new(name).unwrap_or_default();
            let funcs = (self.engfuncs)();
            if let Some(pfn) = funcs.pfnCVarGetString {
                let ptr = pfn(cname.as_ptr());
                if let Some(val) = goldsrc_sys::ffi::cstr_to_string_bounded(ptr, 256) {
                    return Some(val);
                }
            }
            if let Some(pfn_ptr) = funcs.pfnCVarGetPointer {
                let cvar_ptr = pfn_ptr(cname.as_ptr());
                if !cvar_ptr.is_null()
                    && let Some(val) =
                        goldsrc_sys::ffi::cstr_to_string_bounded((*cvar_ptr).string, 256)
                {
                    return Some(val);
                }
            }
            if name == "mapname"
                && let Some(resolver) = MAP_NAME_RESOLVER_FN.get()
                && let Some(m) = resolver()
                && !m.is_empty()
            {
                return Some(m);
            }
            None
        }
    }

    fn cvar_set_string(&self, name: &str, val: &str) {
        unsafe {
            let cname = std::ffi::CString::new(name).unwrap_or_default();
            let cval = std::ffi::CString::new(val).unwrap_or_default();
            call_engfunc!(
                (self.engfuncs)().pfnCVarSetString,
                cname.as_ptr(),
                cval.as_ptr()
            );
        }
    }

    fn cvar_register(
        &self,
        name: &str,
        default_value: &str,
        flags: goldsrc_api::cvar::CvarFlags,
    ) -> bool {
        static REGISTERED_CVARS: LazyLock<Mutex<HashMap<String, RegisteredCvar>>> =
            LazyLock::new(|| Mutex::new(HashMap::new()));

        let mut registry = match REGISTERED_CVARS.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if registry.contains_key(name) {
            return true;
        }

        let cname = match std::ffi::CString::new(name) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let cval = match std::ffi::CString::new(default_value) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let float_val = default_value.trim().parse::<f32>().unwrap_or(0.0);
        let mut cvar_box = Box::new(goldsrc_sys::cvar_t {
            name: cname.as_ptr(),
            string: cval.as_ptr(),
            flags: (flags.bits() | goldsrc_api::cvar::CvarFlags::EXT_DLL.bits())
                as std::os::raw::c_int,
            value: float_val,
            next: std::ptr::null_mut(),
        });

        unsafe {
            let funcs = (self.engfuncs)();
            if let Some(pfn_reg) = funcs.pfnCVarRegister {
                pfn_reg(cvar_box.as_mut() as *mut goldsrc_sys::cvar_t);
            }
        }

        registry.insert(
            name.to_string(),
            RegisteredCvar {
                _cvar: cvar_box,
                _name: cname,
                _string: cval,
            },
        );

        true
    }
}

struct RegisteredCvar {
    _cvar: Box<goldsrc_sys::cvar_t>,
    _name: std::ffi::CString,
    _string: std::ffi::CString,
}

// SAFETY: Heap buffers in Box and CStrings are kept permanently alive and unmoved.
unsafe impl Send for RegisteredCvar {}
unsafe impl Sync for RegisteredCvar {}
