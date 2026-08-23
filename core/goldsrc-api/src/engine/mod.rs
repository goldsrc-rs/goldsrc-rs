//! Modular engine sub-system traits, composite engine bridge, and API facade.

pub mod api;
pub mod console;
pub mod cvars;
pub mod entities;
pub mod messages;
pub mod physics;
pub mod precache;
pub mod sound;

pub use api as engine_api;
pub use console::EngineConsole;
pub use cvars::EngineCvars;
pub use entities::EngineEntities;
pub use messages::{EngineMessages, MessageDest};
pub use physics::{EnginePhysics, TraceResult};
pub use precache::EnginePrecache;
pub use sound::EngineSound;

/// Unified object-safe GoldSrc engine interface.
///
/// Combines modular sub-system traits into a composite bridge implemented
/// by backends and consumed by runtime hosts (`Arc<dyn Engine>`).
pub trait Engine:
    EnginePrecache
    + EngineMessages
    + EngineEntities
    + EngineCvars
    + EnginePhysics
    + EngineSound
    + EngineConsole
    + Send
    + Sync
{
}

// Blanket implementation for any type implementing all engine sub-traits.
impl<T> Engine for T where
    T: EnginePrecache
        + EngineMessages
        + EngineEntities
        + EngineCvars
        + EnginePhysics
        + EngineSound
        + EngineConsole
        + Send
        + Sync
{
}
