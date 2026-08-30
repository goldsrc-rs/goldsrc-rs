//! Safe wrapper around player entities with serial-validated edict access.

use crate::Vector3;
use crate::client::types::AsLangCode;
#[cfg(not(target_arch = "wasm32"))]
use crate::edict::EDict;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::RwLock;

#[cfg(not(target_arch = "wasm32"))]
pub type NativePrintHook = fn(i32, crate::client::PrintTarget, &str);

#[cfg(not(target_arch = "wasm32"))]
pub type PlayerResolverHook = fn(i32) -> Option<Player>;

#[cfg(not(target_arch = "wasm32"))]
static NATIVE_PRINT_HOOK: RwLock<Option<NativePrintHook>> = RwLock::new(None);

#[cfg(not(target_arch = "wasm32"))]
static PLAYER_RESOLVER_HOOK: RwLock<Option<PlayerResolverHook>> = RwLock::new(None);

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
            self.inner.netname()
        }
    }

    /// Returns the player's preferred language code (e.g. `"ru"`, `"en"`).
    pub fn lang(&self) -> String {
        self.as_lang_code().to_string()
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
        crate::menu::session::open_menu(self.index, menu.clone());
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

// SAFETY: Player is just a wrapper around raw pointers / integer index.
// The caller must ensure the pointer is valid when used.
unsafe impl Send for Player {}
unsafe impl Sync for Player {}
