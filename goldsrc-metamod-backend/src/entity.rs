//! Safe abstractions over core GoldSrc engine entities and types.

use std::ffi::CStr;
use goldsrc_sys::{edict_t, entvars_t};

/// Safe wrapper around raw GoldSrc `edict_t`.
#[derive(Debug, Clone, Copy)]
pub struct EdictRef {
    ptr: *mut edict_t,
}

impl EdictRef {
    /// # Safety
    /// `ptr` must be a valid pointer to `edict_t` or null.
    pub unsafe fn from_raw(ptr: *mut edict_t) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub fn as_raw(&self) -> *mut edict_t {
        self.ptr
    }

    pub fn is_free(&self) -> bool {
        // SAFETY: `self.ptr` is guaranteed to be non-null and valid when EdictRef exists.
        unsafe { (*self.ptr).free != 0 }
    }

    pub fn vars(&self) -> Option<EntVarsRef> {
        // SAFETY: `self.ptr` is guaranteed to be non-null. Accessing `.v` entvars_t embedded structure.
        unsafe { EntVarsRef::from_raw(&mut (*self.ptr).v) }
    }
}

/// Safe wrapper around `entvars_t`.
#[derive(Debug, Clone, Copy)]
pub struct EntVarsRef {
    ptr: *mut entvars_t,
}

impl EntVarsRef {
    /// # Safety
    /// `ptr` must be a valid pointer to `entvars_t` or null.
    pub unsafe fn from_raw(ptr: *mut entvars_t) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub fn as_raw(&self) -> *mut entvars_t {
        self.ptr
    }

    pub fn health(&self) -> f32 {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe { (*self.ptr).health }
    }

    pub fn set_health(&mut self, health: f32) {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe {
            (*self.ptr).health = health;
        }
    }

    pub fn max_health(&self) -> f32 {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe { (*self.ptr).max_health }
    }

    pub fn set_max_health(&mut self, max_health: f32) {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe {
            (*self.ptr).max_health = max_health;
        }
    }

    pub fn flags(&self) -> i32 {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe { (*self.ptr).flags }
    }

    pub fn set_flags(&mut self, flags: i32) {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe {
            (*self.ptr).flags = flags;
        }
    }

    pub fn armorvalue(&self) -> f32 {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe { (*self.ptr).armorvalue }
    }

    pub fn set_armorvalue(&mut self, armorvalue: f32) {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe {
            (*self.ptr).armorvalue = armorvalue;
        }
    }

    pub fn origin(&self) -> [f32; 3] {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe { (*self.ptr).origin }
    }

    pub fn set_origin(&mut self, origin: [f32; 3]) {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe {
            (*self.ptr).origin = origin;
        }
    }

    pub fn velocity(&self) -> [f32; 3] {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe { (*self.ptr).velocity }
    }

    pub fn set_velocity(&mut self, velocity: [f32; 3]) {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe {
            (*self.ptr).velocity = velocity;
        }
    }

    pub fn angles(&self) -> [f32; 3] {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe { (*self.ptr).angles }
    }

    pub fn set_angles(&mut self, angles: [f32; 3]) {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe {
            (*self.ptr).angles = angles;
        }
    }

    pub fn movetype(&self) -> i32 {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe { (*self.ptr).movetype }
    }

    pub fn set_movetype(&mut self, movetype: i32) {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe {
            (*self.ptr).movetype = movetype;
        }
    }

    pub fn solid(&self) -> i32 {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe { (*self.ptr).solid }
    }

    pub fn set_solid(&mut self, solid: i32) {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe {
            (*self.ptr).solid = solid;
        }
    }

    pub fn deadflag(&self) -> i32 {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe { (*self.ptr).deadflag }
    }

    pub fn set_deadflag(&mut self, deadflag: i32) {
        // SAFETY: `self.ptr` is non-null entvars_t pointer.
        unsafe {
            (*self.ptr).deadflag = deadflag;
        }
    }

    pub fn classname(&self) -> Option<String> {
        // SAFETY: `self.ptr` is non-null entvars_t pointer. classname is string_t offset in engine string pool.
        unsafe {
            let offset = (*self.ptr).classname;
            if offset == 0 {
                return None;
            }
            let eng = crate::engfuncs();
            let ptr = (eng.pfnSzFromIndex)?(offset as i32);
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }

    pub fn netname(&self) -> Option<String> {
        // SAFETY: `self.ptr` is non-null entvars_t pointer. netname is string_t offset in engine string pool.
        unsafe {
            let offset = (*self.ptr).netname;
            if offset == 0 {
                return None;
            }
            let eng = crate::engfuncs();
            let ptr = (eng.pfnSzFromIndex)?(offset as i32);
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }
}

/// Safe wrapper around CBaseEntity pointer (private base class in HLSDK / ReHLDS).
#[derive(Debug, Clone, Copy)]
pub struct BaseEntityRef {
    ptr: *mut core::ffi::c_void,
}

impl BaseEntityRef {
    /// # Safety
    /// `ptr` must be a valid pointer to `CBaseEntity` or null.
    pub unsafe fn from_raw(ptr: *mut core::ffi::c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub fn as_raw(&self) -> *mut core::ffi::c_void {
        self.ptr
    }
}

