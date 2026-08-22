//! Modular engine sub-system traits, unified engine bridge, and API facade.

pub mod api;
pub mod cvars;
pub mod entities;
pub mod messages;
pub mod physics;
pub mod precache;
pub mod sound;

pub use api as engine_api;
pub use cvars::EngineCvars;
pub use entities::EngineEntities;
pub use messages::{EngineMessages, MessageDest};
pub use physics::{EnginePhysics, TraceResult};
pub use precache::EnginePrecache;
pub use sound::EngineSound;

use crate::Entity;
use crate::client::Player;

/// Engine interface — provides access to low-level engine functions.
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

/// Unified engine bridge for plugin hosts: object-safe, `Send + Sync`.
pub trait EngineOps:
    EnginePrecache
    + EngineMessages
    + EngineEntities
    + EngineCvars
    + EnginePhysics
    + EngineSound
    + Send
    + Sync
{
}

// Blanket implementation for any type implementing all engine sub-traits.
impl<T> EngineOps for T where
    T: EnginePrecache
        + EngineMessages
        + EngineEntities
        + EngineCvars
        + EnginePhysics
        + EngineSound
        + Send
        + Sync
{
}
