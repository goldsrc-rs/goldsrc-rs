//! Modular engine sub-system traits.

pub mod cvars;
pub mod entities;
pub mod messages;
pub mod physics;
pub mod precache;
pub mod sound;

pub use cvars::EngineCvars;
pub use entities::EngineEntities;
pub use messages::{EngineMessages, MessageDest};
pub use physics::{EnginePhysics, TraceResult};
pub use precache::EnginePrecache;
pub use sound::EngineSound;
