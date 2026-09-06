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

    /// Queries a strongly-typed property on this player via ZST marker.
    #[inline(always)]
    pub fn get<P: crate::property::Property<Player> + Default>(&self) -> P::Value {
        P::default().get(self)
    }

    /// Queries a strongly-typed property on this player via explicit property instance.
    #[inline(always)]
    pub fn get_prop<P: crate::property::Property<Player>>(&self, prop: P) -> P::Value {
        prop.get(self)
    }

    /// Mutates a strongly-typed property on this player via ZST marker.
    #[inline(always)]
    pub fn set<P: crate::property::MutProperty<Player> + Default>(&mut self, val: P::Value) {
        P::default().set(self, val);
    }

    /// Mutates a strongly-typed property on this player via explicit property instance.
    #[inline(always)]
    pub fn set_prop<P: crate::property::MutProperty<Player>>(&mut self, prop: P, val: P::Value) {
        prop.set(self, val);
    }

    /// Executes a strongly-typed action or command on this player.
    #[inline(always)]
    pub fn act<A: crate::action::PlayerAction>(&self, action: A) -> A::Output {
        action.execute(self)
    }

    /// Returns `true` if the player is currently alive (`health > 0`).
    #[inline(always)]
    pub fn is_alive(&self) -> bool {
        self.is_valid() && self.health() > 0.0
    }

    /// Returns the player's display name, if set.
    #[inline(always)]
    pub fn name(&self) -> Option<String> {
        self.get::<crate::property::prop::Name>()
    }

    /// Returns the player's preferred language code (e.g. `"ru"`, `"en"`).
    #[inline(always)]
    pub fn lang(&self) -> String {
        self.get::<crate::property::prop::Lang>()
    }

    /// Returns the entity's class name, if set.
    #[inline(always)]
    pub fn classname(&self) -> Option<String> {
        self.get::<crate::property::prop::Classname>()
    }

    /// Returns the player's world origin.
    #[inline(always)]
    pub fn origin(&self) -> Vector3 {
        self.get::<crate::property::prop::Origin>()
    }

    /// Sets the player's world origin.
    #[inline(always)]
    pub fn set_origin(&mut self, pos: Vector3) {
        self.set::<crate::property::prop::Origin>(pos);
    }

    /// Returns the player's velocity.
    #[inline(always)]
    pub fn velocity(&self) -> Vector3 {
        self.get::<crate::property::prop::Velocity>()
    }

    /// Sets the player's velocity.
    #[inline(always)]
    pub fn set_velocity(&mut self, vel: Vector3) {
        self.set::<crate::property::prop::Velocity>(vel);
    }

    /// Returns the player's rotation angles (pitch, yaw, roll).
    #[inline(always)]
    pub fn angles(&self) -> Vector3 {
        self.get::<crate::property::prop::Angles>()
    }

    /// Sets the player's rotation angles.
    #[inline(always)]
    pub fn set_angles(&mut self, angles: Vector3) {
        self.set::<crate::property::prop::Angles>(angles);
    }

    /// Returns the player's current health.
    #[inline(always)]
    pub fn health(&self) -> f32 {
        self.get::<crate::property::prop::Health>()
    }

    /// Sets the player's health.
    #[inline(always)]
    pub fn set_health(&mut self, health: f32) {
        self.set::<crate::property::prop::Health>(health);
    }

    /// Returns the player's armor value.
    #[inline(always)]
    pub fn armorvalue(&self) -> f32 {
        self.get::<crate::property::prop::Armor>()
    }

    /// Sets the player's armor value.
    #[inline(always)]
    pub fn set_armorvalue(&mut self, armor: f32) {
        self.set::<crate::property::prop::Armor>(armor);
    }

    /// Returns the player's current game team.
    #[inline(always)]
    pub fn team(&self) -> crate::client::Team {
        self.get::<crate::property::prop::PlayerTeam>()
    }

    /// Returns the player's current life state.
    #[inline(always)]
    pub fn life_state(&self) -> crate::client::LifeState {
        self.get::<crate::property::prop::PlayerLifeState>()
    }

    /// Prints a message to the specified target (console / center / chat).
    #[inline(always)]
    pub fn print(&self, target: crate::client::PrintTarget, msg: &str) {
        self.act(crate::action::action::Print {
            target,
            message: msg.to_string(),
        });
    }

    /// Prints a message to the player's game console.
    #[inline(always)]
    pub fn print_console(&self, msg: &str) {
        self.act(crate::action::action::Print::console(msg));
    }

    /// Prints a developer notification (top-left screen con_notify area) to the player.
    #[inline(always)]
    pub fn print_notify(&self, msg: &str) {
        self.act(crate::action::action::Print::notify(msg));
    }

    /// Prints a chat message to the player.
    #[inline(always)]
    pub fn print_chat(&self, msg: &str) {
        self.act(crate::action::action::Print::chat(msg));
    }

    /// Prints a center notification message to the player.
    #[inline(always)]
    pub fn print_center(&self, msg: &str) {
        self.act(crate::action::action::Print::center(msg));
    }

    /// Prints a colorized chat message (`^1` default, `^3` team, `^4` green).
    #[inline(always)]
    pub fn print_color(&self, msg: &str) {
        self.act(crate::action::action::Print::colored_chat(msg));
    }

    /// Plays a dynamic sound effect to the player (e.g. `"buttons/button10.wav"`).
    #[inline(always)]
    pub fn play_sound(&self, sample: &str) {
        self.act(crate::action::action::PlaySound::new(sample));
    }

    /// Spawns an item/weapon entity by classname and delivers it to this player.
    #[inline(always)]
    pub fn give_item(&self, item: &str) -> Option<i32> {
        self.act(crate::action::action::GiveItem::new(item))
    }

    /// Displays a raw `ShowMenu` dialog to the player.
    #[inline(always)]
    pub fn show_raw_menu(&self, keys_mask: i32, timeout: i32, text: &str) {
        self.act(crate::action::action::ShowRawMenu {
            keys_mask,
            timeout,
            text,
        });
    }

    /// Sends a screen HUD / DHUD message to the player.
    #[inline(always)]
    pub fn send_hud(&self, msg: &crate::hud::HudMessage) {
        self.act(crate::action::action::SendHud::new(msg));
    }

    /// Renders and opens a declarative `Menu` for this player.
    #[inline(always)]
    pub fn open_menu(&self, menu: &crate::menu::Menu) {
        self.act(crate::action::action::ShowMenu::new(menu));
    }

    /// Displays a menu for the player.
    #[inline(always)]
    pub fn show_menu(&self, menu: &crate::menu::Menu) {
        self.open_menu(menu);
    }

    /// Closes any currently displayed menu on the player's client.
    #[inline(always)]
    pub fn close_menu(&self) {
        self.act(crate::action::action::CloseMenu);
    }

    /// Checks if the player has the specified capability.
    #[inline(always)]
    pub fn has_capability(&self, name: &str) -> bool {
        self.get_prop(crate::property::prop::Capability(name))
    }

    /// Grants a capability to the player dynamically.
    #[inline(always)]
    pub fn grant_capability(&self, name: &str) -> bool {
        self.act(crate::action::action::GrantCapability::new(name))
    }

    /// Revokes a capability from the player dynamically.
    #[inline(always)]
    pub fn revoke_capability(&self, name: &str) -> bool {
        self.act(crate::action::action::RevokeCapability::new(name))
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

impl From<Player> for crate::Entity {
    fn from(player: Player) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            crate::Entity {
                index: player.index,
                inner: player.inner,
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            crate::Entity {
                index: player.index,
            }
        }
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

/// Extension trait providing ergonomic shortcuts over `get`, `set`, and `act`.
pub trait PlayerExt {
    /// Returns the player's current health.
    fn health(&self) -> f32;
    /// Sets the player's health.
    fn set_health(&mut self, health: f32);
    /// Returns the player's armor value.
    fn armorvalue(&self) -> f32;
    /// Sets the player's armor value.
    fn set_armorvalue(&mut self, armor: f32);
    /// Returns the player's world origin.
    fn origin(&self) -> Vector3;
    /// Sets the player's world origin.
    fn set_origin(&mut self, pos: Vector3);
    /// Returns the player's velocity.
    fn velocity(&self) -> Vector3;
    /// Sets the player's velocity.
    fn set_velocity(&mut self, vel: Vector3);
    /// Returns the player's rotation angles (pitch, yaw, roll).
    fn angles(&self) -> Vector3;
    /// Sets the player's rotation angles.
    fn set_angles(&mut self, angles: Vector3);
    /// Returns the player's current game team.
    fn team(&self) -> crate::client::Team;
    /// Returns the player's current life state.
    fn life_state(&self) -> crate::client::LifeState;
    /// Returns `true` if the player is currently alive.
    fn is_alive(&self) -> bool;
    /// Returns the player's display name, if set.
    fn name(&self) -> Option<String>;
    /// Returns the player's preferred language code.
    fn lang(&self) -> String;
    /// Prints a message to player's chat.
    fn print_chat(&self, msg: impl Into<String>);
    /// Prints a center notification message to player's screen.
    fn print_center(&self, msg: impl Into<String>);
    /// Prints a message to player's console.
    fn print_console(&self, msg: impl Into<String>);
    /// Prints a top-left notification to player's screen.
    fn print_notify(&self, msg: impl Into<String>);
    /// Prints a colorized chat message.
    fn print_color(&self, msg: impl Into<String>);
    /// Plays an audio sound effect for this player.
    fn play_sound(&self, sample: impl Into<String>);
    /// Opens an interactive declarative menu for this player.
    fn open_menu(&self, menu: &crate::menu::Menu);
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

impl PlayerExt for Player {
    #[inline(always)]
    fn health(&self) -> f32 {
        self.get::<crate::property::prop::Health>()
    }

    #[inline(always)]
    fn set_health(&mut self, health: f32) {
        self.set::<crate::property::prop::Health>(health);
    }

    #[inline(always)]
    fn armorvalue(&self) -> f32 {
        self.get::<crate::property::prop::Armor>()
    }

    #[inline(always)]
    fn set_armorvalue(&mut self, armor: f32) {
        self.set::<crate::property::prop::Armor>(armor);
    }

    #[inline(always)]
    fn origin(&self) -> Vector3 {
        self.get::<crate::property::prop::Origin>()
    }

    #[inline(always)]
    fn set_origin(&mut self, pos: Vector3) {
        self.set::<crate::property::prop::Origin>(pos);
    }

    #[inline(always)]
    fn velocity(&self) -> Vector3 {
        self.get::<crate::property::prop::Velocity>()
    }

    #[inline(always)]
    fn set_velocity(&mut self, vel: Vector3) {
        self.set::<crate::property::prop::Velocity>(vel);
    }

    #[inline(always)]
    fn angles(&self) -> Vector3 {
        self.get::<crate::property::prop::Angles>()
    }

    #[inline(always)]
    fn set_angles(&mut self, angles: Vector3) {
        self.set::<crate::property::prop::Angles>(angles);
    }

    #[inline(always)]
    fn team(&self) -> crate::client::Team {
        self.get::<crate::property::prop::PlayerTeam>()
    }

    #[inline(always)]
    fn life_state(&self) -> crate::client::LifeState {
        self.get::<crate::property::prop::PlayerLifeState>()
    }

    #[inline(always)]
    fn is_alive(&self) -> bool {
        self.health() > 0.0
    }

    #[inline(always)]
    fn name(&self) -> Option<String> {
        self.get::<crate::property::prop::Name>()
    }

    #[inline(always)]
    fn lang(&self) -> String {
        self.get::<crate::property::prop::Lang>()
    }

    #[inline(always)]
    fn print_chat(&self, msg: impl Into<String>) {
        self.act(crate::action::action::Print::chat(msg));
    }

    #[inline(always)]
    fn print_center(&self, msg: impl Into<String>) {
        self.act(crate::action::action::Print::center(msg));
    }

    #[inline(always)]
    fn print_console(&self, msg: impl Into<String>) {
        self.act(crate::action::action::Print::console(msg));
    }

    #[inline(always)]
    fn print_notify(&self, msg: impl Into<String>) {
        self.act(crate::action::action::Print::notify(msg));
    }

    #[inline(always)]
    fn print_color(&self, msg: impl Into<String>) {
        self.act(crate::action::action::Print::colored_chat(msg));
    }

    #[inline(always)]
    fn play_sound(&self, sample: impl Into<String>) {
        self.act(crate::action::action::PlaySound::new(sample));
    }

    #[inline(always)]
    fn open_menu(&self, menu: &crate::menu::Menu) {
        self.act(crate::action::action::ShowMenu::new(menu));
    }

    #[inline(always)]
    fn close_menu(&self) {
        self.act(crate::action::action::CloseMenu);
    }

    #[inline(always)]
    fn send_hud(&self, msg: &crate::hud::HudMessage) {
        self.act(crate::action::action::SendHud::new(msg));
    }

    #[inline(always)]
    fn give_item(&self, item: impl Into<String>) -> Option<i32> {
        self.act(crate::action::action::GiveItem::new(item))
    }

    #[inline(always)]
    fn has_capability(&self, name: &str) -> bool {
        self.get_prop(crate::property::prop::Capability(name))
    }

    #[inline(always)]
    fn grant_capability(&self, name: impl Into<String>) -> bool {
        self.act(crate::action::action::GrantCapability::new(name))
    }

    #[inline(always)]
    fn revoke_capability(&self, name: impl Into<String>) -> bool {
        self.act(crate::action::action::RevokeCapability::new(name))
    }
}
