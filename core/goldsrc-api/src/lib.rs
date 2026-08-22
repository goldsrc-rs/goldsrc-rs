//! Pure Rust traits (interfaces) for GoldSrc engine interaction.
//!
//! This crate defines the abstract interface that plugin developers use.
//! It has no dependency on any specific backend (Metamod or Standalone).

/// Capability-based access control, registry, and hierarchical DSL.
pub mod auth;
/// Generated WASM bindings (wasm32 only).
pub mod bindings;
/// Core player and client domain abstractions, states, and typestate guards.
pub mod client;
/// Command routing targets, scope filters, programmatic builder, and errors.
pub mod command;
/// Global constants for the engine and framework.
pub mod consts;
/// Validated `edict_t` handle.
pub mod edict;
/// Modular engine sub-system traits, unified engine bridge, and API facade.
pub mod engine;
/// Abstract interface for plugin runtime execution hosts.
pub mod plugin_host;

pub use auth::{Auth, CapExpr, CapabilityRegistry};
pub use client::{
    Alive, Bot, ClientKind, ConnectionState, CounterTerrorist, Dead, HLTV, LifeState, Player,
    Spectator, Team, Terrorist,
};
pub use command::{
    ChatScope, Command, CommandBuilder, CommandContext, CommandError, CommandResult, CommandTarget,
    FromArg, PlayerStateFilter,
};
pub use edict::EDict;
pub use engine::{
    Engine, EngineCvars, EngineEntities, EngineMessages, EnginePhysics, EnginePrecache,
    EngineSound, MessageDest, TraceResult, engine_api,
};
pub use plugin_host::{HostError, HostResult, PluginHost};

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

impl Vector3 {
    /// Create a new 3D vector.
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
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

// SAFETY: Entity is just a wrapper around raw pointers / integer index.
// The caller must ensure the pointer is valid when used.
unsafe impl Send for Entity {}
unsafe impl Sync for Entity {}
