//! Core Entity handle and extension trait for GoldSrc world and dynamic entities.

pub mod ext;
pub mod property;
pub mod spec;

pub use crate::client::slot::EntityId;
pub use ext::EntityExt;
pub use property::Classname;
pub use spec::{Dormant, Solid, SolidEntity, Spawned, SpawnedEntity};

#[cfg(not(target_arch = "wasm32"))]
use crate::types::EDict;

use crate::action::Action;
#[cfg(target_arch = "wasm32")]
use crate::bindings::goldsrc::engine::api as host_api;
use crate::property::{Prop, PropGet, PropSet};

/// Validated handle to an active GoldSrc engine entity (world, items, physics, monsters, players).
/// Strictly bound to the GoldSrc main thread (!Send, !Sync).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    /// Entity index (0 = world, 1..=N = players, N+1.. = entities).
    pub index: i32,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) inner: EDict,
    pub(crate) _marker: std::marker::PhantomData<*const ()>,
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
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates an `Entity` handle for `index`.
    #[cfg(target_arch = "wasm32")]
    pub fn new(index: i32) -> Self {
        Self {
            index,
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates an `Entity` handle for `index` with an invalid backing edict
    /// (host-only placeholder; use [`Entity::from_raw`] with a real pointer).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(index: i32) -> Self {
        Self {
            index,
            inner: EDict::invalid(),
            _marker: std::marker::PhantomData,
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
            host_api::host_entity_is_valid(self.index)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.is_valid()
        }
    }

    /// Queries a property of type `T` from this entity.
    #[inline(always)]
    pub fn get<T: PropGet<Entity>>(&self) -> T {
        T::get_from(self)
    }

    /// Mutates a property of type `T` on this entity.
    #[inline(always)]
    pub fn set<T: PropSet<Entity>>(&mut self, val: T) {
        val.set_on(self);
    }

    /// In-place mutation of a property on this entity.
    #[inline(always)]
    pub fn modify<T: Prop<Entity>>(&mut self, f: impl FnOnce(&mut T)) {
        let mut val = self.get::<T>();
        f(&mut val);
        self.set(val);
    }

    /// Executes a strongly-typed action against this entity.
    #[inline(always)]
    pub fn act<A: Action<Entity>>(&self, action: A) -> A::Output {
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

    /// Returns the thread-safe copyable entity ID for off-thread communication.
    #[inline(always)]
    pub const fn id(&self) -> EntityId {
        EntityId(self.index)
    }
}

// Entity is strictly bound to the GoldSrc engine main thread.
// It contains PhantomData<*const ()>, making it !Send and !Sync by design.
// Cross-thread communication must use EntityId instead.
