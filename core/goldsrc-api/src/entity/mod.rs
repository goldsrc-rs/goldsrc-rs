//! Core Entity handle and extension trait for GoldSrc world and dynamic entities.

pub mod ext;
pub use ext::EntityExt;

#[cfg(not(target_arch = "wasm32"))]
use crate::types::EDict;

/// Validated handle to an active GoldSrc engine entity (world, items, physics, monsters, players).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    /// Entity index (0 = world, 1..=N = players, N+1.. = entities).
    pub index: i32,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) inner: EDict,
}

impl Entity {
    /// Creates an Entity from a raw index and edict_t pointer.
    ///
    /// # Safety
    /// The caller must ensure that `edict` is a valid pointer to an entity in the engine.
    #[cfg(all(not(target_arch = "wasm32"), feature = "unsafe-sys"))]
    pub unsafe fn from_raw(index: i32, edict: *mut goldsrc_sys::edict_t) -> Self {
        // SAFETY: propagated from caller.
        Self {
            index,
            inner: unsafe { EDict::from_raw(index, edict) },
        }
    }

    /// Creates an `Entity` handle for `index`.
    #[cfg(target_arch = "wasm32")]
    pub fn new(index: i32) -> Self {
        Self { index }
    }

    /// Creates an `Entity` handle for `index` with an invalid backing edict
    /// (host-only placeholder; use [`Entity::from_raw`] with a real pointer).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(index: i32) -> Self {
        Self {
            index,
            inner: EDict::invalid(),
        }
    }

    /// Returns the entity index.
    #[inline(always)]
    pub const fn index(&self) -> i32 {
        self.index
    }

    /// Returns `true` if the underlying edict slot is still valid.
    pub fn is_valid(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_is_valid(self.index)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.is_valid()
        }
    }

    /// Queries a strongly-typed property on this entity.
    #[inline(always)]
    pub fn get<P: crate::property::Property<Entity>>(&self, prop: P) -> P::Value {
        prop.get(self)
    }

    /// Mutates a strongly-typed property on this entity.
    #[inline(always)]
    pub fn set<P: crate::property::MutProperty<Entity>>(&mut self, prop: P, val: P::Value) {
        prop.set(self, val);
    }

    /// Executes a strongly-typed action against this entity.
    #[inline(always)]
    pub fn act<A: crate::action::Action<Entity>>(&self, action: A) -> A::Output {
        action.execute(self)
    }

    /// Returns the raw `edict_t` pointer, or null if the handle is stale.
    #[cfg(all(not(target_arch = "wasm32"), feature = "unsafe-sys"))]
    pub fn as_ptr(&self) -> *mut goldsrc_sys::edict_t {
        self.inner.as_ptr().unwrap_or(std::ptr::null_mut())
    }

    /// Access the underlying [`EDict`] handle directly.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn edict(&self) -> EDict {
        self.inner
    }
}

// SAFETY: Entity is just a wrapper around raw pointers / integer index.
// The caller must ensure the pointer is valid when used.
unsafe impl Send for Entity {}
unsafe impl Sync for Entity {}
