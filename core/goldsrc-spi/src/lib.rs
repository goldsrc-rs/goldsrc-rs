//! Service Provider Interface (SPI) traits, extension points, and driver contracts for GoldSrc.rs.
//!
//! This crate contains provider-facing interfaces implemented by external drivers,
//! engines, and infrastructure adapters (e.g. Auth providers, Storage backends, Engine HAL).

pub mod auth;
pub mod engine;
pub mod extension;
pub mod storage;

pub use auth::{AuthProvider, HandshakeContext, HandshakeDecision};
pub use engine::{
    Engine, EngineConsole, EngineCvars, EngineEntities, EngineExtensions, EngineMessages,
    EnginePhysics, EnginePrecache, EngineSound, MessageBuilder, MessageDest, TraceResult,
};
pub use extension::EngineExtension;
pub use storage::{SqlDatabase, StorageError, StorageProvider};
