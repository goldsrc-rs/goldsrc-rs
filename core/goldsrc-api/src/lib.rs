//! Pure Rust traits (interfaces) for GoldSrc engine interaction.
//!
//! This crate defines the abstract interface that plugin developers use.
//! It has no dependency on any specific backend (Metamod or Standalone).

/// Capability-based access control.
pub mod auth;
/// Generated WASM bindings (wasm32 only).
pub mod bindings;
/// Shared capability registry (host + native auth).
pub mod caps;
/// Global constants for the engine and framework.
pub mod consts;
/// Validated `edict_t` handle.
pub mod edict;
/// Modular engine sub-system traits.
pub mod engine;
/// Narrow object-safe engine bridge for the WASM host.
pub mod engine_ops;
/// Abstract interface for plugin runtime execution hosts.
pub mod plugin_host;

pub use caps::CapabilityRegistry;
pub use edict::EDict;
pub use engine::*;
pub use engine_ops::EngineOps;
pub use plugin_host::{HostError, HostResult, PluginHost};

/// Engine interface — provides access to engine functions.
pub trait Engine {
    /// Spawn an entity by classname.
    fn spawn_entity(&self, classname: &str) -> Option<Entity>;

    /// Get a player by index (1-based).
    fn get_player(&self, index: i32) -> Option<Player>;

    /// Print a message to the server console.
    fn server_print(&self, message: &str);

    /// Execute a server command.
    fn server_command(&self, command: &str);

    /// Get a cvar value as float.
    fn cvar_get_float(&self, name: &str) -> f32;

    /// Set a cvar value.
    fn cvar_set_float(&self, name: &str, value: f32);
}

/// Safe wrapper around `edict_t` (entity dictionary).
///
/// Delegates field access to [`EDict`] which validates the serial number on
/// every access, preventing use-after-free from stale handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    /// Entity index (0 = world, 1..=N = players).
    pub index: i32,
    #[cfg(not(target_arch = "wasm32"))]
    inner: EDict,
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
    pub fn index(&self) -> i32 {
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

    /// Returns the entity classname (e.g. `"player"`, `"func_door"`), if set.
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

    /// Prints a chat message to the player. On native hosts this is a no-op
    /// (chat printing is not yet part of the engine bridge surface).
    pub fn print_chat(&self, msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_log(&format!(
                "Print to player {}: {}",
                self.index, msg
            ));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = msg;
        }
    }

    /// Checks if the player has the specified capability.
    pub fn has_capability(&self, name: &str) -> bool {
        crate::auth::Auth::has_capability(self.index, name)
    }

    /// Grants a capability to the player dynamically.
    pub fn grant_capability(&self, name: &str) -> bool {
        crate::auth::Auth::grant_capability(self.index, name)
    }

    /// Returns the entity origin as a flat `[x, y, z]` array.
    pub fn origin(&self) -> [f32; 3] {
        #[cfg(target_arch = "wasm32")]
        {
            let v = crate::bindings::goldsrc::engine::api::host_entity_origin(self.index);
            [v.x, v.y, v.z]
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.origin().unwrap_or([0.0, 0.0, 0.0])
        }
    }

    /// Returns the entity health.
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

    /// Returns the entity's rotation angles (pitch, yaw, roll).
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

    /// Sets the entity's rotation angles.
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

/// 3D Vector type for GoldSrc positions and velocities.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector3 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
    /// Z component.
    pub z: f32,
}

impl From<[f32; 3]> for Vector3 {
    fn from(arr: [f32; 3]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
            z: arr[2],
        }
    }
}

impl From<Vector3> for [f32; 3] {
    fn from(v: Vector3) -> Self {
        [v.x, v.y, v.z]
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
    inner: EDict,
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

    /// Creates a `Player` handle for `index`.
    #[cfg(target_arch = "wasm32")]
    pub fn new(index: i32) -> Self {
        Self { index }
    }

    /// Creates a `Player` handle for `index` with an invalid backing edict
    /// (host-only placeholder; use [`Player::from_raw`] with a real pointer).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(index: i32) -> Self {
        Self {
            index,
            inner: EDict::invalid(),
        }
    }

    /// Returns the player index (1-based).
    pub fn index(&self) -> i32 {
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

    /// Prints a chat message to the player. On native hosts this is a no-op
    /// (chat printing is not yet part of the engine bridge surface).
    pub fn print_chat(&self, msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_log(&format!(
                "Print to player {}: {}",
                self.index, msg
            ));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = msg;
        }
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

    /// Returns `true` if the player is alive (health > 0).
    pub fn is_alive(&self) -> bool {
        self.health() > 0.0
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

impl From<Player> for Entity {
    fn from(player: Player) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Entity {
                index: player.index,
                inner: player.inner,
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Entity {
                index: player.index,
            }
        }
    }
}

/// Plugin trait — implement this for your plugin.
pub trait Plugin: Send + Sync {
    /// Called when the plugin is loaded.
    fn on_load(&mut self) {}

    /// Called when the plugin is unloaded.
    fn on_unload(&self) {}

    /// Called when a player connects.
    fn on_client_connect(&self, _player: &Player) {}

    /// Called when a player disconnects.
    fn on_client_disconnect(&self, _player: &Player) {}

    /// Called when a player spawns.
    fn on_client_spawn(&self, _player: &Player) {}

    /// Called when a player dies.
    fn on_client_killed(&self, _victim: &Player, _killer: &Player) {}

    /// Called every server frame.
    fn on_server_frame(&self) {}
}

// SAFETY: Entity and Player are just wrappers around raw pointers.
// The caller must ensure the pointer is valid when used.
unsafe impl Send for Entity {}
unsafe impl Sync for Entity {}
unsafe impl Send for Player {}
unsafe impl Sync for Player {}
