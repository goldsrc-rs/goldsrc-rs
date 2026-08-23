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

/// Maximum number of players supported by the GoldSrc engine.
pub const MAX_PLAYERS: u16 = 32;

/// Maximum number of entity edicts in GoldSrc engine.
pub const MAX_EDICTS: u16 = 2048;

/// Maximum payload length for a single GoldSrc UserMessage buffer in bytes.
pub const MAX_USER_MSG_DATA_LEN: usize = 192;

/// Standard engine interface version (`DLL_FUNCTIONS`).
pub const ENGINE_INTERFACE_VERSION: i32 = 140;

/// Standard NEW_DLL_FUNCTIONS interface version.
pub const NEW_DLL_INTERFACE_VERSION: i32 = 1;

/// Client print destination: Notify.
pub const PRINT_NOTIFY: i32 = 1;

/// Client print destination: Console.
pub const PRINT_CONSOLE: i32 = 2;

/// Client print destination: Chat.
pub const PRINT_CHAT: i32 = 3;

/// Client print destination: Center message.
pub const PRINT_CENTER: i32 = 4;

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
