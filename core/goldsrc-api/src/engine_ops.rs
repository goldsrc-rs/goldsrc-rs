//! Unified object-safe engine bridge.
//!
//! Combines modular sub-system traits (`EnginePrecache`, `EngineMessages`,
//! `EngineEntities`, `EngineCvars`, `EnginePhysics`, `EngineSound`) into a
//! unified, object-safe interface shared with plugin hosts as `Arc<dyn EngineOps>`.

use crate::engine::*;

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
