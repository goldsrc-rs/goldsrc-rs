//! Service Provider Interface (SPI) traits, extension points, and driver contracts for GoldSrc.rs.
//!
//! This crate contains provider-facing interfaces implemented by external drivers,
//! engines, and infrastructure adapters (e.g. Auth providers, Storage backends, Engine HAL).

pub mod auth;
pub mod cvar;
pub mod engine;
pub mod entity;
pub mod extension;
pub mod identity;
pub mod storage;

pub use auth::{AuthProvider, HandshakeContext, HandshakeDecision};
pub use cvar::{CvarEngine, CvarFlags};
pub use engine::{
    Engine, EngineConsole, EngineCvars, EngineEntities, EngineExtensions, EngineMessages,
    EnginePhysics, EnginePrecache, EngineSound, MessageBuilder, MessageDest, TraceResult,
};
pub use entity::EntitySpawner;
pub use extension::EngineExtension;
pub use identity::{
    AuthIdentity, AuthState, AuthSubject, PlayerGuid, PlayerIdentity, PlayerSessionToken, SteamId,
    murmur3_128,
};
pub use storage::{SqlDatabase, StorageError, StorageProvider};
