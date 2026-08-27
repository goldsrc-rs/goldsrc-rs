pub mod effects;
pub mod types;

pub use effects::{FadeFlags, ScreenFade, ScreenFadeBuilder, ScreenShake, ScreenShakeBuilder};
pub use types::{HudColor, HudCoord, HudEffect, HudKind, HudMessage, HudMessageBuilder};

/// Maximum number of distinct HUD channels in GoldSrc (1..=4).
pub const MAX_HUD_CHANNELS: usize = 4;

/// Normalized coordinate representing center screen alignment (-1.0).
pub const HUD_COORD_CENTER: f32 = -1.0;

/// Network message opcode for Director HUD messages (`SVC_DIRECTOR`).
pub const SVC_DIRECTOR: i32 = 51;

/// Director command sub-opcode for screen text messages (`DRC_CMD_MESSAGE`).
pub const DRC_CMD_MESSAGE: u8 = 2;

/// Network message opcode for temporary entities (`SVC_TEMPENTITY`).
pub const SVC_TEMPENTITY: i32 = 23;

/// TempEntity sub-type for screen text messages (`TE_TEXTMESSAGE`).
pub const TE_TEXTMESSAGE: u8 = 29;
