//! Safe wrapper around player entities with serial-validated edict access.

use crate::Vector3;
#[cfg(not(target_arch = "wasm32"))]
use crate::edict::EDict;

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

    /// Prints a chat message to the player.
    pub fn print_chat(&self, msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_print_chat(self.index, msg);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (self.index, msg);
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
