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
static NATIVE_PRINT_HOOK: RwLock<Option<NativePrintHook>> = RwLock::new(None);

#[cfg(not(target_arch = "wasm32"))]
static PLAYER_RESOLVER_HOOK: RwLock<Option<PlayerResolverHook>> = RwLock::new(None);

#[cfg(not(target_arch = "wasm32"))]
static PLAYER_NAME_RESOLVER_HOOK: RwLock<Option<PlayerNameResolverHook>> = RwLock::new(None);

#[cfg(not(target_arch = "wasm32"))]
static PLAYER_TEAM_RESOLVER_HOOK: RwLock<Option<PlayerTeamResolverHook>> = RwLock::new(None);

#[cfg(not(target_arch = "wasm32"))]
static PLAYER_LANG_RESOLVER_HOOK: RwLock<Option<PlayerLangResolverHook>> = RwLock::new(None);

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
static OPEN_MENU_HOOK: RwLock<Option<OpenMenuHook>> = RwLock::new(None);

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
    pub fn is_alive(&self) -> bool {
        self.is_valid() && self.health() > 0.0
    }

    /// Returns the player's display name, if set.
    pub fn name(&self) -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_player_name(self.index)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(lock) = PLAYER_NAME_RESOLVER_HOOK.read()
                && let Some(resolver) = *lock
                && let Some(name) = resolver(self.index)
            {
                return Some(name);
            }
            self.inner.netname()
        }
    }

    /// Returns the player's preferred language code (e.g. `"ru"`, `"en"`).
    pub fn lang(&self) -> String {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_player_lang(self.index)
                .unwrap_or_else(|| "en".to_string())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(lock) = PLAYER_LANG_RESOLVER_HOOK.read()
                && let Some(resolver) = *lock
                && let Some(lang) = resolver(self.index)
            {
                return lang;
            }
            "en".to_string()
        }
    }

    /// Returns the entity's class name, if set.
    pub fn classname(&self) -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_classname(self.index)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.classname()
        }
    }

    /// Returns the player's world origin.
    pub fn origin(&self) -> Vector3 {
        #[cfg(target_arch = "wasm32")]
        {
            let v = crate::bindings::goldsrc::engine::api::host_entity_origin(self.index);
            Vector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.origin().unwrap_or([0.0, 0.0, 0.0]).into()
        }
    }

    /// Sets the player's world origin.
    pub fn set_origin(&mut self, pos: Vector3) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_set_origin(
                self.index,
                crate::bindings::goldsrc::engine::api::Vector3 {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                },
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.set_origin(pos.into());
        }
    }

    /// Returns the player's velocity.
    pub fn velocity(&self) -> Vector3 {
        #[cfg(target_arch = "wasm32")]
        {
            let v = crate::bindings::goldsrc::engine::api::host_entity_velocity(self.index);
            Vector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.velocity().unwrap_or([0.0, 0.0, 0.0]).into()
        }
    }

    /// Sets the player's velocity.
    pub fn set_velocity(&mut self, vel: Vector3) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_set_velocity(
                self.index,
                crate::bindings::goldsrc::engine::api::Vector3 {
                    x: vel.x,
                    y: vel.y,
                    z: vel.z,
                },
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.set_velocity(vel.into());
        }
    }

    /// Returns the player's rotation angles (pitch, yaw, roll).
    pub fn angles(&self) -> Vector3 {
        #[cfg(target_arch = "wasm32")]
        {
            let v = crate::bindings::goldsrc::engine::api::host_entity_angles(self.index);
            Vector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.angles().unwrap_or([0.0, 0.0, 0.0]).into()
        }
    }

    /// Sets the player's rotation angles.
    pub fn set_angles(&mut self, angles: Vector3) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_set_angles(
                self.index,
                crate::bindings::goldsrc::engine::api::Vector3 {
                    x: angles.x,
                    y: angles.y,
                    z: angles.z,
                },
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.set_angles(angles.into());
        }
    }

    /// Returns the player's current health.
    pub fn health(&self) -> f32 {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_health(self.index)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.health().unwrap_or(0.0)
        }
    }

    /// Sets the player's health.
    pub fn set_health(&mut self, health: f32) {
        if !health.is_finite() {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_set_health(self.index, health);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.set_health(health);
        }
    }

    /// Returns the player's armor value.
    pub fn armorvalue(&self) -> f32 {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_player_armorvalue(self.index)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.armorvalue().unwrap_or(0.0)
        }
    }

    /// Sets the player's armor value.
    pub fn set_armorvalue(&mut self, armor: f32) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_player_set_armorvalue(self.index, armor);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.set_armorvalue(armor);
        }
    }

    /// Returns the player's current game team.
    pub fn team(&self) -> crate::client::Team {
        #[cfg(target_arch = "wasm32")]
        {
            crate::client::Team::from(crate::bindings::goldsrc::engine::api::host_player_team(
                self.index,
            ))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(lock) = PLAYER_TEAM_RESOLVER_HOOK.read()
                && let Some(resolver) = *lock
            {
                return resolver(self.index).into();
            }
            self.inner.team().unwrap_or(0).into()
        }
    }

    /// Returns the player's current life state.
    pub fn life_state(&self) -> crate::client::LifeState {
        if !self.is_valid() {
            return crate::client::LifeState::Dead;
        }
        if self.health() > 0.0 {
            crate::client::LifeState::Alive
        } else {
            crate::client::LifeState::Dead
        }
    }

    /// Prints a message to the specified target (console / center / chat).
    ///
    /// This is the single dispatch point; the `print_*` helpers below are
    /// convenience wrappers. On native hosts printing is currently a no-op
    /// (the engine bridge surface is WASM-first).
    pub fn print(&self, target: crate::client::PrintTarget, msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            use crate::bindings::goldsrc::engine::api as host;
            match target {
                crate::client::PrintTarget::Console => host::host_print_console(self.index, msg),
                crate::client::PrintTarget::Center => host::host_print_center(self.index, msg),
                crate::client::PrintTarget::Notify => host::host_print_notify(self.index, msg),
                // Chat and ColoredChat share the SayText transport; the colored
                // variant only documents that ^1/^3/^4 escapes are meaningful.
                crate::client::PrintTarget::Chat | crate::client::PrintTarget::ColoredChat => {
                    host::host_print_chat(self.index, msg)
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(lock) = NATIVE_PRINT_HOOK.read()
                && let Some(hook) = *lock
            {
                hook(self.index, target, msg);
            }
        }
    }

    /// Prints a message to the player's game console.
    pub fn print_console(&self, msg: &str) {
        self.print(crate::client::PrintTarget::Console, msg);
    }

    /// Prints a developer notification (top-left screen con_notify area) to the player.
    pub fn print_notify(&self, msg: &str) {
        self.print(crate::client::PrintTarget::Notify, msg);
    }

    /// Prints a chat message to the player.
    pub fn print_chat(&self, msg: &str) {
        self.print(crate::client::PrintTarget::Chat, msg);
    }

    /// Prints a center notification message to the player.
    pub fn print_center(&self, msg: &str) {
        self.print(crate::client::PrintTarget::Center, msg);
    }

    /// Prints a colorized chat message (`^1` default, `^3` team, `^4` green).
    /// Color escapes render in CS 1.6 / CZ clients only.
    pub fn print_color(&self, msg: &str) {
        self.print(crate::client::PrintTarget::ColoredChat, msg);
    }

    /// Plays a dynamic sound effect to the player (e.g. `"buttons/button10.wav"`).
    pub fn play_sound(&self, sample: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_emit_sound(
                self.index, 0, // CHAN_AUTO
                sample, 1.0, // VOL_NORM
                1.0, // ATTN_NORM
                0, 100, // PITCH_NORM
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = sample;
        }
    }

    /// Spawns an item/weapon entity by classname (e.g. `"weapon_m4a1"`) and
    /// delivers it to this player via the real GameDLL's spawn + touch flow,
    /// mirroring AMX Mod X's `give_item`: create → position at player →
    /// DispatchSpawn → force Touch.
    ///
    /// Returns the new entity index. Requires a backend with GameDLL access
    /// (standalone proxy); on backends without it the entity is not created.
    pub fn give_item(&self, item: &str) -> Option<i32> {
        #[cfg(target_arch = "wasm32")]
        {
            use crate::bindings::goldsrc::engine::api as host;
            let ent = host::host_create_named_entity(item)?;
            let o = host::host_entity_origin(self.index);
            host::host_entity_set_origin(
                ent,
                crate::bindings::goldsrc::engine::api::Vector3 {
                    x: o.x,
                    y: o.y,
                    z: o.z,
                },
            );
            host::host_dispatch_spawn(ent);
            host::host_dispatch_touch(ent, self.index);
            Some(ent)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = item;
            None
        }
    }

    /// Displays a raw `ShowMenu` dialog to the player.
    pub fn show_raw_menu(&self, keys_mask: i32, timeout: i32, text: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_show_menu(
                self.index, keys_mask, timeout, text,
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (self.index, keys_mask, timeout, text);
        }
    }

    /// Sends a screen HUD / DHUD message to the player.
    pub fn send_hud(&self, msg: &crate::hud::HudMessage) {
        let (effect_val, fade_in, fade_out, hold_time) = match msg.effect {
            crate::hud::HudEffect::FadeInOut {
                fade_in,
                fade_out,
                hold_time,
            } => (0, fade_in, fade_out, hold_time),
            crate::hud::HudEffect::Flicker {
                fx_time: _,
                hold_time,
            } => (1, 0.0, 0.0, hold_time),
            crate::hud::HudEffect::Typewriter {
                char_time: _,
                fade_out,
                hold_time,
            } => (2, 0.05, fade_out, hold_time),
        };

        match msg.kind {
            crate::hud::HudKind::Classic { channel } => {
                #[cfg(target_arch = "wasm32")]
                {
                    crate::bindings::goldsrc::engine::api::host_send_hud_message(
                        self.index,
                        channel as i32,
                        msg.position.x,
                        msg.position.y,
                        msg.color.r as i32,
                        msg.color.g as i32,
                        msg.color.b as i32,
                        msg.color.a as i32,
                        effect_val,
                        fade_in,
                        fade_out,
                        hold_time,
                        &msg.text,
                    );
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let _ = (channel, effect_val, fade_in, fade_out, hold_time);
                }
            }
            crate::hud::HudKind::Dhud => {
                #[cfg(target_arch = "wasm32")]
                {
                    crate::bindings::goldsrc::engine::api::host_send_dhud_message(
                        self.index,
                        msg.position.x,
                        msg.position.y,
                        msg.color.r as i32,
                        msg.color.g as i32,
                        msg.color.b as i32,
                        msg.color.a as i32,
                        effect_val,
                        fade_in,
                        fade_out,
                        hold_time,
                        &msg.text,
                    );
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let _ = (effect_val, fade_in, fade_out, hold_time);
                }
            }
        }
    }

    /// Renders and opens a declarative `Menu` for this player.
    pub fn open_menu(&self, menu: &crate::menu::Menu) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(lock) = OPEN_MENU_HOOK.read()
                && let Some(hook) = *lock
            {
                hook(self.index, menu);
                return;
            }
        }
        let total_players = crate::auth::Auth::total_players();
        let ctx = crate::menu::MenuContext {
            player_index: self.index,
            round_number: 1,
            round_time_elapsed: 0.0,
            is_alive: self.health() > 0.0,
            players_count: if total_players > 0 {
                total_players as u32
            } else {
                1
            },
        };
        if let Some(rendered) = menu.render_page(&ctx, 0) {
            match rendered.renderer {
                crate::menu::MenuRendererKind::Text => {
                    self.show_raw_menu(rendered.keys_mask as i32, rendered.timeout, &rendered.text);
                }
                crate::menu::MenuRendererKind::Dhud {
                    position,
                    color,
                    effect,
                } => {
                    let hud_msg = crate::hud::HudMessage {
                        text: rendered.text.clone(),
                        kind: crate::hud::HudKind::Dhud,
                        color,
                        color2: color,
                        position,
                        effect,
                    };
                    self.send_hud(&hud_msg);
                    self.show_raw_menu(rendered.keys_mask as i32, rendered.timeout, "");
                }
            }
        }
    }

    /// Closes any currently displayed menu on the player's client.
    pub fn close_menu(&self) {
        self.show_raw_menu(0, 0, "");
    }

    /// Checks if the player has the specified capability.
    pub fn has_capability(&self, name: &str) -> bool {
        crate::auth::Auth::has_capability(self.index, name)
    }

    /// Grants a capability to the player dynamically.
    pub fn grant_capability(&self, name: &str) -> bool {
        crate::auth::Auth::grant_capability(self.index, name)
    }

    /// Revokes a capability from the player dynamically.
    pub fn revoke_capability(&self, name: &str) -> bool {
        crate::auth::Auth::revoke_capability(self.index, name)
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
    /// Prints a message to player's chat.
    fn print_chat(&self, msg: impl Into<String>);
    /// Prints a center notification message to player's screen.
    fn print_center(&self, msg: impl Into<String>);
    /// Prints a message to player's console.
    fn print_console(&self, msg: impl Into<String>);
    /// Prints a top-left notification to player's screen.
    fn print_notify(&self, msg: impl Into<String>);
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
        crate::auth::Auth::has_capability(self.index, name)
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
