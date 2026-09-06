//! Safe wrapper around player entities with serial-validated edict access.

#[cfg(not(target_arch = "wasm32"))]
use crate::edict::EDict;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::RwLock;

#[cfg(not(target_arch = "wasm32"))]
pub type NativePrintHook = fn(i32, crate::client::PrintTarget, &str);

#[cfg(not(target_arch = "wasm32"))]
pub type PlayerResolverHook = fn(i32) -> Option<Player>;

#[cfg(not(target_arch = "wasm32"))]
pub type PlayerNameResolverHook = fn(i32) -> Option<String>;

#[cfg(not(target_arch = "wasm32"))]
pub type PlayerTeamResolverHook = fn(i32) -> i32;

#[cfg(not(target_arch = "wasm32"))]
pub type PlayerLangResolverHook = fn(i32) -> Option<String>;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) static NATIVE_PRINT_HOOK: RwLock<Option<NativePrintHook>> = RwLock::new(None);

#[cfg(not(target_arch = "wasm32"))]
pub(crate) static PLAYER_RESOLVER_HOOK: RwLock<Option<PlayerResolverHook>> = RwLock::new(None);

#[cfg(not(target_arch = "wasm32"))]
pub(crate) static PLAYER_NAME_RESOLVER_HOOK: RwLock<Option<PlayerNameResolverHook>> =
    RwLock::new(None);

#[cfg(not(target_arch = "wasm32"))]
pub(crate) static PLAYER_TEAM_RESOLVER_HOOK: RwLock<Option<PlayerTeamResolverHook>> =
    RwLock::new(None);

#[cfg(not(target_arch = "wasm32"))]
pub(crate) static PLAYER_LANG_RESOLVER_HOOK: RwLock<Option<PlayerLangResolverHook>> =
    RwLock::new(None);

/// Registers the native backend print dispatcher for host-side `Player::print_*` calls.
#[cfg(not(target_arch = "wasm32"))]
pub fn set_native_print_hook(hook: NativePrintHook) {
    if let Ok(mut lock) = NATIVE_PRINT_HOOK.write() {
        *lock = Some(hook);
    }
}

/// Registers the native engine player resolver for `Player::new(index)` on the host.
#[cfg(not(target_arch = "wasm32"))]
pub fn set_player_resolver_hook(hook: PlayerResolverHook) {
    if let Ok(mut lock) = PLAYER_RESOLVER_HOOK.write() {
        *lock = Some(hook);
    }
}

/// Registers the native engine player name resolver for `Player::name()` on the host.
#[cfg(not(target_arch = "wasm32"))]
pub fn set_player_name_hook(hook: PlayerNameResolverHook) {
    if let Ok(mut lock) = PLAYER_NAME_RESOLVER_HOOK.write() {
        *lock = Some(hook);
    }
}

/// Registers the native engine player team resolver for `Player::team()` on the host.
#[cfg(not(target_arch = "wasm32"))]
pub fn set_player_team_hook(hook: PlayerTeamResolverHook) {
    if let Ok(mut lock) = PLAYER_TEAM_RESOLVER_HOOK.write() {
        *lock = Some(hook);
    }
}

/// Registers the native engine player lang resolver for `Player::lang()` on the host.
#[cfg(not(target_arch = "wasm32"))]
pub fn set_player_lang_hook(hook: PlayerLangResolverHook) {
    if let Ok(mut lock) = PLAYER_LANG_RESOLVER_HOOK.write() {
        *lock = Some(hook);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub type OpenMenuHook = fn(i32, &crate::menu::Menu);

#[cfg(not(target_arch = "wasm32"))]
pub(crate) static OPEN_MENU_HOOK: RwLock<Option<OpenMenuHook>> = RwLock::new(None);

/// Registers the engine/runtime open menu handler for host-side `Player::open_menu` calls.
#[cfg(not(target_arch = "wasm32"))]
pub fn set_open_menu_hook(hook: OpenMenuHook) {
    if let Ok(mut lock) = OPEN_MENU_HOOK.write() {
        *lock = Some(hook);
    }
}

/// Safe wrapper around a player entity.
///
/// Delegates all edict field accesses to the underlying [`EDict`] handle,
/// which performs serial-number validation on every read/write.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Player {
    /// Player index (1-based).
    pub index: i32,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) inner: EDict,
}

impl Player {
    /// Creates a Player from a raw index and edict_t pointer.
    ///
    /// # Safety
    /// The caller must ensure that `edict` is a valid pointer to a player entity in the engine.
    #[cfg(all(not(target_arch = "wasm32"), feature = "unsafe-sys"))]
    pub unsafe fn from_raw(index: i32, edict: *mut goldsrc_sys::edict_t) -> Self {
        Self {
            index,
            inner: unsafe { EDict::from_raw(index, edict) },
        }
    }

    /// Creates a Player handle from a verified index on native host.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_index(index: i32) -> Self {
        Self {
            index,
            inner: EDict::invalid(),
        }
    }

    /// Creates a `Player` handle for `index`.
    #[cfg(target_arch = "wasm32")]
    pub fn new(index: i32) -> Self {
        Self { index }
    }

    /// Creates a `Player` handle for `index` with backing edict resolved via host engine if available.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(index: i32) -> Self {
        if let Ok(lock) = PLAYER_RESOLVER_HOOK.read()
            && let Some(resolver) = *lock
            && let Some(player) = resolver(index)
        {
            return player;
        }
        Self {
            index,
            inner: EDict::invalid(),
        }
    }

    /// Returns the player index (1-based).
    pub const fn index(&self) -> i32 {
        self.index
    }

    /// Returns `true` if the underlying player entity is valid and connected.
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

    /// Queries a property of type `T` from this player.
    #[inline(always)]
    pub fn get<T: crate::property::PropertyGetter<Player>>(&self) -> T {
        T::get_from(self)
    }

    /// Mutates a property of type `T` on this player.
    #[inline(always)]
    pub fn set<T: crate::property::PropertySetter<Player>>(&mut self, val: T) {
        val.set_on(self);
    }

    /// In-place mutation of a property on this player.
    #[inline(always)]
    pub fn modify<T>(&mut self, f: impl FnOnce(&mut T))
    where
        T: crate::property::PropertyGetter<Player> + crate::property::PropertySetter<Player>,
    {
        let mut val = self.get::<T>();
        f(&mut val);
        self.set(val);
    }

    /// Executes a strongly-typed action or command on this player.
    #[inline(always)]
    pub fn act<A: crate::action::Action<Player>>(&self, action: A) -> A::Output {
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

impl std::ops::Deref for Player {
    type Target = crate::Entity;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        // SAFETY: Player and Entity have identical #[repr(C)] memory layout (index: i32, inner: EDict).
        unsafe { &*(self as *const Player as *const crate::Entity) }
    }
}

impl std::ops::DerefMut for Player {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Player and Entity have identical #[repr(C)] memory layout (index: i32, inner: EDict).
        unsafe { &mut *(self as *mut Player as *mut crate::Entity) }
    }
}

impl AsRef<crate::Entity> for Player {
    #[inline(always)]
    fn as_ref(&self) -> &crate::Entity {
        self
    }
}

impl AsMut<crate::Entity> for Player {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut crate::Entity {
        self
    }
}

impl From<Player> for crate::Entity {
    fn from(player: Player) -> Self {
        *player
    }
}

impl From<i32> for Player {
    fn from(index: i32) -> Self {
        Player::new(index)
    }
}

impl From<&Player> for Player {
    fn from(p: &Player) -> Self {
        *p
    }
}

impl From<&mut Player> for Player {
    fn from(p: &mut Player) -> Self {
        *p
    }
}

// SAFETY: Player is just a wrapper around raw pointers / integer index.
// The caller must ensure the pointer is valid when used.
unsafe impl Send for Player {}
unsafe impl Sync for Player {}
