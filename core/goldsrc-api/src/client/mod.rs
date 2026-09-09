//! Core player and client domain abstractions, states, and typestate guards.

pub mod ext;
pub mod player;
pub mod property;
pub mod slot;
pub mod spec;
pub mod types;

pub use crate::entity::EntityExt;
pub use ext::{ClientExt, PlayerExt};
pub use player::Player;
pub use property::{Lang, Name};
pub use slot::PlayerSlot;
pub use spec::{
    Alive, Bot, Connected, ConnectedClient, Dead, DeadPlayer, Hltv, Human, HumanClient,
    LivingHuman, LivingPlayer, SpectatingPlayer, Spectator,
};
pub use types::{AsLangCode, ClientKind, ConnectionState, LifeState, PrintTarget, Team};

use crate::Entity;
use crate::action::Action;
use crate::property::{Prop, PropGet, PropSet};
#[cfg(not(target_arch = "wasm32"))]
use crate::types::EDict;

/// Validated handle to an active GoldSrc engine client slot (1..=32: Player, Bot, HLTV).
/// Strictly bound to the GoldSrc main thread (!Send, !Sync).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Client {
    /// Client slot index (1-based, 1..=32).
    pub index: i32,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) inner: EDict,
    pub(crate) _marker: std::marker::PhantomData<*const ()>,
}

impl Client {
    /// Creates a Client from a raw index and edict_t pointer.
    ///
    /// # Safety
    /// The caller must ensure that `edict` is a valid pointer to a client entity in the engine.
    #[cfg(all(not(target_arch = "wasm32"), feature = "unsafe-sys"))]
    pub unsafe fn from_raw(index: i32, edict: *mut goldsrc_sys::edict_t) -> Self {
        Self {
            index,
            inner: unsafe { EDict::from_raw(index, edict) },
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates a Client handle from a verified index on native host.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_index(index: i32) -> Self {
        Self {
            index,
            inner: EDict::invalid(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates a `Client` handle for `index`.
    #[cfg(target_arch = "wasm32")]
    pub fn new(index: i32) -> Self {
        Self {
            index,
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates a `Client` handle for `index` with backing edict resolved via host engine if available.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(index: i32) -> Self {
        if let Ok(lock) = player::PLAYER_RESOLVER_HOOK.read()
            && let Some(resolver) = *lock
            && let Some(player) = resolver(index)
        {
            return *player.client();
        }
        Self {
            index,
            inner: EDict::invalid(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the client slot index (1-based).
    #[inline(always)]
    pub const fn index(&self) -> i32 {
        self.index
    }

    /// Returns `true` if the underlying client slot is valid and connected.
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

    /// Returns the underlying raw `edict_t` pointer, or null if invalid.
    #[cfg(all(not(target_arch = "wasm32"), feature = "unsafe-sys"))]
    pub fn as_ptr(&self) -> *mut goldsrc_sys::edict_t {
        self.inner.as_ptr().unwrap_or(std::ptr::null_mut())
    }

    /// Access the underlying [`EDict`] handle directly.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn edict(&self) -> EDict {
        self.inner
    }

    /// Converts this client into a combatant [`Player`] handle, if not an HLTV proxy.
    #[inline(always)]
    pub fn as_player(&self) -> Option<Player> {
        if self.is_valid() && !self.is_hltv() {
            Some(Player {
                index: self.index,
                #[cfg(not(target_arch = "wasm32"))]
                inner: self.inner,
                _marker: std::marker::PhantomData,
            })
        } else {
            None
        }
    }

    /// Returns the thread-safe copyable slot for off-thread communication.
    #[inline(always)]
    pub const fn slot(&self) -> PlayerSlot {
        PlayerSlot(self.index)
    }

    /// Queries a property of type `T` from this client.
    #[inline(always)]
    pub fn get<T: PropGet<Client>>(&self) -> T {
        T::get_from(self)
    }

    /// Mutates a property of type `T` on this client.
    #[inline(always)]
    pub fn set<T: PropSet<Client>>(&mut self, val: T) {
        val.set_on(self);
    }

    /// In-place mutation of a property on this client.
    #[inline(always)]
    pub fn modify<T: Prop<Client>>(&mut self, f: impl FnOnce(&mut T)) {
        let mut val = self.get::<T>();
        f(&mut val);
        self.set(val);
    }

    /// Executes a strongly-typed action on this client.
    #[inline(always)]
    pub fn act<A: Action<Client>>(&self, action: A) -> A::Output {
        action.execute(self)
    }
}

const _: () = {
    assert!(std::mem::size_of::<Client>() == std::mem::size_of::<Entity>());
    assert!(std::mem::align_of::<Client>() == std::mem::align_of::<Entity>());
};

impl std::ops::Deref for Client {
    type Target = Entity;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        // SAFETY: Client and Entity have identical #[repr(C)] memory layout (index: i32, inner: EDict, _marker).
        unsafe { &*(self as *const Client as *const Entity) }
    }
}

impl std::ops::DerefMut for Client {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Client and Entity have identical #[repr(C)] memory layout (index: i32, inner: EDict, _marker).
        unsafe { &mut *(self as *mut Client as *mut Entity) }
    }
}

impl AsRef<Entity> for Client {
    #[inline(always)]
    fn as_ref(&self) -> &Entity {
        self
    }
}

impl AsMut<Entity> for Client {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut Entity {
        self
    }
}

impl From<Client> for Entity {
    #[inline(always)]
    fn from(c: Client) -> Self {
        *c
    }
}

impl From<i32> for Client {
    #[inline(always)]
    fn from(index: i32) -> Self {
        Client::new(index)
    }
}

// Client is strictly bound to the GoldSrc engine main thread.
// It contains PhantomData<*const ()>, making it !Send and !Sync by design.
// Cross-thread communication must use PlayerSlot instead.
