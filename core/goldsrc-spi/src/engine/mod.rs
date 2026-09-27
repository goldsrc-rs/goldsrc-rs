//! Modular engine Hardware Abstraction Layer (HAL) SPI traits and composite Engine interface.

pub mod console;
pub mod cvars;
pub mod entities;
pub mod messages;
pub mod physics;
pub mod precache;
pub mod sound;

pub use console::EngineConsole;
pub use cvars::EngineCvars;
pub use entities::EngineEntities;
pub use messages::{EngineMessages, MessageBuilder, MessageDest};
pub use physics::{EnginePhysics, TraceResult};
pub use precache::EnginePrecache;
pub use sound::EngineSound;

/// Composite engine Hardware Abstraction Layer (HAL) bridge interface.
///
/// Combines modular sub-system traits into a unified driver interface implemented
/// by backends (Metamod, Standalone) and consumed by runtime hosts (`Arc<dyn Engine>`).
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
