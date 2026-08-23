//! Safe, validated wrapper around `edict_t` — the GoldSrc entity dictionary entry.
//!
//! # Design
//!
//! GoldSrc engine internally tracks entity validity via `edict_t.serialnumber`:
//! when an entity slot is freed and re-used, the `serialnumber` is incremented.
//! By storing the `serialnumber` we observed at creation time, we can detect
//! stale handles without dereferencing potentially freed memory.
//!
//! ## Lifetime contract
//!
//! All methods that read from or write to the underlying `edict_t` are gated
//! behind an [`is_valid`](EDict::is_valid) check. If the serial number does not
//! match (slot was recycled), all methods that return data return `None` / the
//! default value; setters are no-ops.
//!
//! The raw pointer is never stored as `*mut` in a `pub` field, preventing
//! accidental unsynchronised access from plugin code.

use std::sync::atomic::{AtomicU64, Ordering};

static MAP_GENERATION: AtomicU64 = AtomicU64::new(1);

/// Returns the current map session generation counter.
pub fn current_map_generation() -> u64 {
    MAP_GENERATION.load(Ordering::Relaxed)
}

/// Advances the map session generation counter, invalidating all handles from prior maps.
pub fn bump_map_generation() {
    MAP_GENERATION.fetch_add(1, Ordering::SeqCst);
}

/// Validated handle to a GoldSrc engine entity.
///
/// Cheap to copy (two words) — safe to store across frames, checked on access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EDict {
    /// Index of this edict in the engine's global array (0-based for world,
    /// 1-based for players, up to `gpGlobals->maxEntities`).
    index: i32,
    /// Raw pointer. Stored as `usize` to avoid compiler aliasing assumptions.
    /// Re-cast to `*mut edict_t` only inside `unsafe` methods after validation.
    ptr: usize,
    /// Serial number at the time this handle was created. If the edict slot
    /// has since been recycled, `edict_t.serialnumber` will differ.
    serial: i32,
    /// Map session generation at creation time to prevent cross-map UAF.
    generation: u64,
}

impl EDict {
    /// Constructs an `EDict` from raw engine data.
    ///
    /// # Safety
    /// `edict` must be a valid, non-null `edict_t` pointer owned by the engine.
    ///
    /// Host-only: this is the bridge between engine raw pointers and the safe
    /// handle type, so it lives behind the `unsafe-sys` feature that only the
    /// backends enable.
    #[cfg(all(not(target_arch = "wasm32"), feature = "unsafe-sys"))]
    pub unsafe fn from_raw(index: i32, edict: *mut goldsrc_sys::edict_t) -> Self {
        debug_assert!(!edict.is_null(), "EDict::from_raw called with null pointer");
        let serial = if edict.is_null() {
            0
        } else {
            // SAFETY: edict is verified non-null above
            unsafe { (*edict).serialnumber }
        };
        Self {
            index,
            ptr: edict as usize,
            serial,
            generation: current_map_generation(),
        }
    }

    /// Creates a placeholder `EDict` that is always invalid.
    pub const fn invalid() -> Self {
        Self {
            index: -1,
            ptr: 0,
            serial: -1,
            generation: 0,
        }
    }

    /// Returns the entity index.
    pub fn index(self) -> i32 {
        self.index
    }

    /// Returns `true` if the underlying edict slot is still assigned to the
    /// same logical entity (serial number unchanged and pointer non-null).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn is_valid(self) -> bool {
        if self.ptr == 0 || self.generation != current_map_generation() {
            return false;
        }
        // SAFETY: We check ptr != 0 and generation match above.
        // We only read `serialnumber` and `free` — plain i32 fields.
        let current_serial = unsafe { (*(self.ptr as *const goldsrc_sys::edict_t)).serialnumber };
        current_serial == self.serial
            && !(unsafe { (*(self.ptr as *const goldsrc_sys::edict_t)).free } != 0)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn is_valid(self) -> bool {
        self.index >= 0
    }

    /// Returns the raw edict pointer, or `None` if no longer valid.
    ///
    /// The returned pointer is only valid until the next server frame or any
    /// engine call that may free entities.
    #[cfg(all(not(target_arch = "wasm32"), feature = "unsafe-sys"))]
    pub fn as_ptr(self) -> Option<*mut goldsrc_sys::edict_t> {
        self.raw_ptr()
    }

    /// Returns the raw edict pointer, or `None` if no longer valid.
    ///
    /// Internal variant used by the safe accessors below; callers must treat
    /// the pointer as valid only for the current engine call.
    #[cfg(not(target_arch = "wasm32"))]
    fn raw_ptr(self) -> Option<*mut goldsrc_sys::edict_t> {
        if self.is_valid() {
            Some(self.ptr as *mut goldsrc_sys::edict_t)
        } else {
            None
        }
    }

    // =========================================================================
    // Field accessors (safe wrappers)
    // =========================================================================

    #[cfg(not(target_arch = "wasm32"))]
    pub fn classname(self) -> Option<String> {
        let _ptr = self.raw_ptr()?;
        // String offset requires engine pfnSzFromIndex / pStringBase resolver;
        // fallback returns None if called directly on raw handle without engine context.
        None
    }

    /// Entity origin (world position).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn origin(self) -> Option<[f32; 3]> {
        let ptr = self.raw_ptr()?;
        // SAFETY: ptr validated; origin is a plain [f32; 3] field.
        Some(unsafe { (*ptr).v.origin })
    }

    /// Set entity origin.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_origin(self, origin: [f32; 3]) -> bool {
        match self.raw_ptr() {
            Some(ptr) => {
                // SAFETY: ptr validated above.
                unsafe { (*ptr).v.origin = origin };
                true
            }
            None => false,
        }
    }

    /// Entity health points.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn health(self) -> Option<f32> {
        let ptr = self.raw_ptr()?;
        Some(unsafe { (*ptr).v.health })
    }

    /// Set entity health. Returns `false` if entity is no longer valid.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_health(self, health: f32) -> bool {
        match self.raw_ptr() {
            Some(ptr) => {
                unsafe { (*ptr).v.health = health };
                true
            }
            None => false,
        }
    }

    /// Returns `true` if health > 0 and entity is still valid.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn is_alive(self) -> bool {
        self.health().map(|h| h > 0.0).unwrap_or(false)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn netname(self) -> Option<String> {
        let _ptr = self.raw_ptr()?;
        // String offset requires engine pfnSzFromIndex / pStringBase resolver;
        // fallback returns None if called directly on raw handle without engine context.
        None
    }

    /// Player armor value. Only meaningful for player entities.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn armorvalue(self) -> Option<f32> {
        let ptr = self.raw_ptr()?;
        Some(unsafe { (*ptr).v.armorvalue })
    }

    /// Set player armor. Returns `false` if entity no longer valid.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_armorvalue(self, armor: f32) -> bool {
        match self.raw_ptr() {
            Some(ptr) => {
                unsafe { (*ptr).v.armorvalue = armor };
                true
            }
            None => false,
        }
    }

    /// Entity velocity.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn velocity(self) -> Option<[f32; 3]> {
        let ptr = self.raw_ptr()?;
        Some(unsafe { (*ptr).v.velocity })
    }

    /// Set entity velocity.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_velocity(self, velocity: [f32; 3]) -> bool {
        match self.raw_ptr() {
            Some(ptr) => {
                unsafe { (*ptr).v.velocity = velocity };
                true
            }
            None => false,
        }
    }

    /// Entity angles (pitch, yaw, roll).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn angles(self) -> Option<[f32; 3]> {
        let ptr = self.raw_ptr()?;
        Some(unsafe { (*ptr).v.angles })
    }

    /// Set entity angles.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_angles(self, angles: [f32; 3]) -> bool {
        match self.raw_ptr() {
            Some(ptr) => {
                unsafe { (*ptr).v.angles = angles };
                true
            }
            None => false,
        }
    }
}

// EDict is just two integers + a pointer-width integer. Thread-safety is
// the caller's responsibility (same as all engine interaction).
// SAFETY: See module-level safety contract.
unsafe impl Send for EDict {}
unsafe impl Sync for EDict {}

// ============================================================================
// Unit tests (host-only)
// ============================================================================

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn invalid_edict_is_not_valid() {
        let e = EDict::invalid();
        assert!(!e.is_valid());
        assert_eq!(e.index(), -1);
    }

    #[cfg(feature = "unsafe-sys")]
    #[test]
    fn invalid_edict_returns_none() {
        let e = EDict::invalid();
        assert!(e.raw_ptr().is_none());
        assert!(e.classname().is_none());
        assert!(e.health().is_none());
        assert!(e.origin().is_none());
        assert!(e.netname().is_none());
    }

    #[test]
    fn set_on_invalid_edict_returns_false() {
        let e = EDict::invalid();
        assert!(!e.set_health(100.0));
        assert!(!e.set_origin([0.0, 0.0, 0.0]));
        assert!(!e.set_velocity([0.0, 0.0, 0.0]));
        assert!(!e.set_armorvalue(0.0));
    }

    #[cfg(feature = "unsafe-sys")]
    #[test]
    fn map_generation_invalidation() {
        let mut raw_edict: goldsrc_sys::edict_t = unsafe { std::mem::zeroed() };
        raw_edict.serialnumber = 42;
        raw_edict.free = 0;

        let handle = unsafe { EDict::from_raw(1, &mut raw_edict as *mut _) };
        assert!(handle.is_valid());

        bump_map_generation();

        assert!(!handle.is_valid());
        assert!(handle.raw_ptr().is_none());
    }
}
