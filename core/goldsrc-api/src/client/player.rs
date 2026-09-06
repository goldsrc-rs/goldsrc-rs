//! Safe wrapper around player entities with serial-validated edict access.

use crate::Vector3;
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

    /// Queries a strongly-typed property on this player.
    ///
    /// Accepts both unit ZST markers (e.g. `player.get(prop::Health)`) and
    /// parameterized properties (e.g. `player.get(prop::Capability("admin"))`).
    #[inline(always)]
    pub fn get<P: crate::property::Property<Player>>(&self, prop: P) -> P::Value {
        prop.get(self)
    }

    /// Mutates a strongly-typed property on this player.
    ///
    /// Accepts both unit ZST markers (e.g. `player.set(prop::Health, 100.0)`) and
    /// parameterized properties (e.g. `player.set(prop::Capability("admin"), true)`).
    #[inline(always)]
    pub fn set<P: crate::property::MutProperty<Player>>(&mut self, prop: P, val: P::Value) {
        prop.set(self, val);
    }

    /// Executes a strongly-typed action or command on this player.
    #[inline(always)]
    pub fn act<A: crate::action::PlayerAction>(&self, action: A) -> A::Output {
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

/// Extension trait providing spatial, physics, vital, and identity queries on entities.
pub trait EntityExt {
    /// Returns the entity's 3D world origin.
    fn origin(&self) -> Vector3;
    /// Sets the entity's 3D world origin.
    fn set_origin(&mut self, pos: Vector3);
    /// Returns the entity's velocity vector.
    fn velocity(&self) -> Vector3;
    /// Sets the entity's velocity vector.
    fn set_velocity(&mut self, vel: Vector3);
    /// Returns the entity's rotation angles (pitch, yaw, roll).
    fn angles(&self) -> Vector3;
    /// Sets the entity's rotation angles.
    fn set_angles(&mut self, angles: Vector3);
    /// Returns the entity's current health.
    fn health(&self) -> f32;
    /// Sets the entity's health.
    fn set_health(&mut self, health: f32);
    /// Returns the entity's class name, if set.
    fn classname(&self) -> Option<String>;
    /// Returns `true` if the entity is alive (`health > 0.0`).
    fn is_alive(&self) -> bool;
    /// Returns `true` if the entity slot is currently valid.
    fn is_valid(&self) -> bool;
}

/// Extension trait providing client-specific queries and actions (slots 1..=32: Player, Bot, HLTV).
pub trait ClientExt: EntityExt {
    /// Returns the 1-based client slot index (1..=32).
    fn client_index(&self) -> i32;
    /// Returns the client display name, if set.
    fn name(&self) -> Option<String>;
    /// Returns the client's preferred language code.
    fn lang(&self) -> String;
    /// Returns the client kind (Player, Bot, HLTV).
    fn client_kind(&self) -> crate::client::ClientKind;
    /// Returns `true` if this client is an AI bot (`FL_FAKECLIENT`).
    fn is_bot(&self) -> bool;
    /// Returns `true` if this client is an HLTV proxy (`FL_PROXY`).
    fn is_hltv(&self) -> bool;
    /// Prints a message to client's console.
    fn print_console(&self, msg: impl Into<String>);
    /// Prints a top-left notification to client's screen.
    fn print_notify(&self, msg: impl Into<String>);
}

/// Extension trait providing gameplay combatant operations (Human Player, Bot).
pub trait PlayerExt: ClientExt {
    /// Returns the player's armor value.
    fn armorvalue(&self) -> f32;
    /// Sets the player's armor value.
    fn set_armorvalue(&mut self, armor: f32);
    /// Returns the player's current game team.
    fn team(&self) -> crate::client::Team;
    /// Returns the player's current life state.
    fn life_state(&self) -> crate::client::LifeState;
    /// Prints a message to the specified target.
    fn print(&self, target: crate::client::PrintTarget, msg: impl Into<String>);
    /// Prints a message to player's chat.
    fn print_chat(&self, msg: impl Into<String>);
    /// Prints a center notification message to player's screen.
    fn print_center(&self, msg: impl Into<String>);
    /// Prints a colorized chat message.
    fn print_color(&self, msg: impl Into<String>);
    /// Plays an audio sound effect for this player.
    fn play_sound(&self, sample: impl Into<String>);
    /// Opens an interactive declarative menu for this player.
    fn open_menu(&self, menu: &crate::menu::Menu);
    /// Displays a menu for the player.
    fn show_menu(&self, menu: &crate::menu::Menu);
    /// Displays a raw `ShowMenu` dialog to the player.
    fn show_raw_menu(&self, keys_mask: i32, timeout: i32, text: &str);
    /// Closes any currently displayed menu on the player's client.
    fn close_menu(&self);
    /// Sends a HUD or DHUD message to the player.
    fn send_hud(&self, msg: &crate::hud::HudMessage);
    /// Spawns an item or weapon entity and delivers it to the player.
    fn give_item(&self, item: impl Into<String>) -> Option<i32>;
    /// Checks if the player has the specified capability.
    fn has_capability(&self, name: &str) -> bool;
    /// Grants a capability to the player dynamically.
    fn grant_capability(&self, name: impl Into<String>) -> bool;
    /// Revokes a capability from the player dynamically.
    fn revoke_capability(&self, name: impl Into<String>) -> bool;
}

impl EntityExt for crate::Entity {
    #[inline(always)]
    fn origin(&self) -> Vector3 {
        self.get(crate::property::Origin)
    }

    #[inline(always)]
    fn set_origin(&mut self, pos: Vector3) {
        self.set(crate::property::Origin, pos);
    }

    #[inline(always)]
    fn velocity(&self) -> Vector3 {
        self.get(crate::property::Velocity)
    }

    #[inline(always)]
    fn set_velocity(&mut self, vel: Vector3) {
        self.set(crate::property::Velocity, vel);
    }

    #[inline(always)]
    fn angles(&self) -> Vector3 {
        self.get(crate::property::Angles)
    }

    #[inline(always)]
    fn set_angles(&mut self, angles: Vector3) {
        self.set(crate::property::Angles, angles);
    }

    #[inline(always)]
    fn health(&self) -> f32 {
        self.get(crate::property::Health)
    }

    #[inline(always)]
    fn set_health(&mut self, health: f32) {
        self.set(crate::property::Health, health);
    }

    #[inline(always)]
    fn classname(&self) -> Option<String> {
        self.get(crate::property::Classname)
    }

    #[inline(always)]
    fn is_alive(&self) -> bool {
        self.health() > 0.0
    }

    #[inline(always)]
    fn is_valid(&self) -> bool {
        crate::Entity::is_valid(self)
    }
}

impl EntityExt for Player {
    #[inline(always)]
    fn origin(&self) -> Vector3 {
        self.get(crate::property::Origin)
    }

    #[inline(always)]
    fn set_origin(&mut self, pos: Vector3) {
        self.set(crate::property::Origin, pos);
    }

    #[inline(always)]
    fn velocity(&self) -> Vector3 {
        self.get(crate::property::Velocity)
    }

    #[inline(always)]
    fn set_velocity(&mut self, vel: Vector3) {
        self.set(crate::property::Velocity, vel);
    }

    #[inline(always)]
    fn angles(&self) -> Vector3 {
        self.get(crate::property::Angles)
    }

    #[inline(always)]
    fn set_angles(&mut self, angles: Vector3) {
        self.set(crate::property::Angles, angles);
    }

    #[inline(always)]
    fn health(&self) -> f32 {
        self.get(crate::property::Health)
    }

    #[inline(always)]
    fn set_health(&mut self, health: f32) {
        self.set(crate::property::Health, health);
    }

    #[inline(always)]
    fn classname(&self) -> Option<String> {
        self.get(crate::property::Classname)
    }

    #[inline(always)]
    fn is_alive(&self) -> bool {
        self.health() > 0.0
    }

    #[inline(always)]
    fn is_valid(&self) -> bool {
        Player::is_valid(self)
    }
}

impl ClientExt for Player {
    #[inline(always)]
    fn client_index(&self) -> i32 {
        self.index
    }

    #[inline(always)]
    fn name(&self) -> Option<String> {
        self.get(crate::property::Name)
    }

    #[inline(always)]
    fn lang(&self) -> String {
        self.get(crate::property::Lang)
    }

    #[inline(always)]
    fn client_kind(&self) -> crate::client::ClientKind {
        if self.is_hltv() {
            crate::client::ClientKind::HLTV
        } else if self.is_bot() {
            crate::client::ClientKind::Bot
        } else {
            crate::client::ClientKind::Player
        }
    }

    #[inline(always)]
    fn is_bot(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(flags) = self.inner.flags() {
                return (flags & crate::consts::FL_FAKECLIENT) != 0;
            }
            false
        }
        #[cfg(target_arch = "wasm32")]
        {
            false
        }
    }

    #[inline(always)]
    fn is_hltv(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(flags) = self.inner.flags() {
                return (flags & crate::consts::FL_PROXY) != 0;
            }
            false
        }
        #[cfg(target_arch = "wasm32")]
        {
            false
        }
    }

    #[inline(always)]
    fn print_console(&self, msg: impl Into<String>) {
        self.act(crate::action::Print::console(msg));
    }

    #[inline(always)]
    fn print_notify(&self, msg: impl Into<String>) {
        self.act(crate::action::Print::notify(msg));
    }
}

impl PlayerExt for Player {
    #[inline(always)]
    fn armorvalue(&self) -> f32 {
        self.get(crate::property::Armor)
    }

    #[inline(always)]
    fn set_armorvalue(&mut self, armor: f32) {
        self.set(crate::property::Armor, armor);
    }

    #[inline(always)]
    fn team(&self) -> crate::client::Team {
        self.get(crate::property::PlayerTeam)
    }

    #[inline(always)]
    fn life_state(&self) -> crate::client::LifeState {
        self.get(crate::property::PlayerLifeState)
    }

    #[inline(always)]
    fn print(&self, target: crate::client::PrintTarget, msg: impl Into<String>) {
        self.act(crate::action::Print {
            target,
            message: msg.into(),
        });
    }

    #[inline(always)]
    fn print_chat(&self, msg: impl Into<String>) {
        self.act(crate::action::Print::chat(msg));
    }

    #[inline(always)]
    fn print_center(&self, msg: impl Into<String>) {
        self.act(crate::action::Print::center(msg));
    }

    #[inline(always)]
    fn print_color(&self, msg: impl Into<String>) {
        self.act(crate::action::Print::colored_chat(msg));
    }

    #[inline(always)]
    fn play_sound(&self, sample: impl Into<String>) {
        self.act(crate::action::PlaySound::new(sample));
    }

    #[inline(always)]
    fn open_menu(&self, menu: &crate::menu::Menu) {
        self.act(crate::action::ShowMenu::new(menu));
    }

    #[inline(always)]
    fn show_menu(&self, menu: &crate::menu::Menu) {
        self.open_menu(menu);
    }

    #[inline(always)]
    fn show_raw_menu(&self, keys_mask: i32, timeout: i32, text: &str) {
        self.act(crate::action::ShowRawMenu {
            keys_mask,
            timeout,
            text,
        });
    }

    #[inline(always)]
    fn close_menu(&self) {
        self.act(crate::action::CloseMenu);
    }

    #[inline(always)]
    fn send_hud(&self, msg: &crate::hud::HudMessage) {
        self.act(crate::action::SendHud::new(msg));
    }

    #[inline(always)]
    fn give_item(&self, item: impl Into<String>) -> Option<i32> {
        self.act(crate::action::GiveItem::new(item))
    }

    #[inline(always)]
    fn has_capability(&self, name: &str) -> bool {
        self.get(crate::property::Capability(name))
    }

    #[inline(always)]
    fn grant_capability(&self, name: impl Into<String>) -> bool {
        self.act(crate::action::GrantCapability::new(name))
    }

    #[inline(always)]
    fn revoke_capability(&self, name: impl Into<String>) -> bool {
        self.act(crate::action::RevokeCapability::new(name))
    }
}
