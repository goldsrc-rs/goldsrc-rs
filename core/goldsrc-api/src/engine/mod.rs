//! Modular engine sub-system traits, composite engine bridge, and API facade.

pub mod api;
pub mod codec;
pub mod console;
pub mod cvars;
pub mod entities;
pub mod messages;
pub mod physics;
pub mod precache;
pub mod sound;

pub use api as engine_api;
pub use codec::{format_center_text, format_say_text, utf8_to_cp1251};
pub use console::EngineConsole;
pub use cvars::EngineCvars;
pub use entities::EngineEntities;
pub use messages::{EngineMessages, MessageBuilder, MessageDest};
pub use physics::{EnginePhysics, TraceResult};
pub use precache::EnginePrecache;
pub use sound::EngineSound;

/// Maximum number of players supported by the GoldSrc engine.
pub const MAX_PLAYERS: u16 = 32;

/// Maximum number of entity edicts in GoldSrc engine.
pub const MAX_EDICTS: u16 = 2048;

/// Maximum payload size in bytes for a single user network message.
pub const MAX_USER_MSG_DATA_LEN: usize = 192;

/// Standard engine interface version (`DLL_FUNCTIONS`).
pub const ENGINE_INTERFACE_VERSION: i32 = 140;

/// Standard NEW_DLL_FUNCTIONS interface version.
pub const NEW_DLL_INTERFACE_VERSION: i32 = 1;

/// Client print destination: Console (HLSDK `print_console = 0`).
pub const PRINT_CONSOLE: i32 = 0;

/// Client print destination: Center message (HLSDK `print_center = 1`).
pub const PRINT_CENTER: i32 = 1;

/// Client print destination: Chat (HLSDK `print_chat = 2`).
pub const PRINT_CHAT: i32 = 2;

/// Client print destination: Notify / developer print.
pub const PRINT_NOTIFY: i32 = 0;

/// HUD / TextMsg print destination: Notify / developer print (HLSDK `HUD_PRINTNOTIFY = 1`).
pub const HUD_PRINTNOTIFY: i32 = 1;

/// HUD / TextMsg print destination: Console (HLSDK `HUD_PRINTCONSOLE = 2`).
pub const HUD_PRINTCONSOLE: i32 = 2;

/// HUD / TextMsg print destination: Chat (HLSDK `HUD_PRINTCHAT = 3`).
pub const HUD_PRINTCHAT: i32 = 3;

/// HUD / TextMsg print destination: Center message (HLSDK `HUD_PRINTCENTER = 4`).
pub const HUD_PRINTCENTER: i32 = 4;

/// HUD / TextMsg print destination: Radio message (HLSDK `HUD_PRINTRADIO = 5`).
pub const HUD_PRINTRADIO: i32 = 5;

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
