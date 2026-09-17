//! Common entity keyvalue constant names used by GoldSrc entities.

/// Entity target name (identifier used for triggering).
pub const TARGET_NAME: &str = "targetname";

/// Entity target (target entity name triggered by this entity).
pub const TARGET: &str = "target";

/// Entity model asset path (e.g. "models/player.mdl" or "*1" for BSP brush).
pub const MODEL: &str = "model";

/// Entity origin in world coordinates as string "x y z".
pub const ORIGIN: &str = "origin";

/// Entity Euler angles as string "pitch yaw roll".
pub const ANGLES: &str = "angles";

/// Spawn flags bitmask.
pub const SPAWN_FLAGS: &str = "spawnflags";

/// Visual rendering mode (see [`crate::entity::RenderMode`]).
pub const RENDER_MODE: &str = "rendermode";

/// Visual rendering amount / opacity (0..=255).
pub const RENDER_AMT: &str = "renderamt";

/// Visual rendering color as string "r g b" (0..=255).
pub const RENDER_COLOR: &str = "rendercolor";

/// Visual rendering special effect (see [`crate::entity::RenderFx`]).
pub const RENDER_FX: &str = "renderfx";

/// Physical solidity type (see [`crate::entity::SolidType`]).
pub const SOLID: &str = "solid";

/// Entity health points.
pub const HEALTH: &str = "health";

/// Maximum health points.
pub const MAX_HEALTH: &str = "max_health";

/// Movement speed or door/train transit speed.
pub const SPEED: &str = "speed";

/// Visual rendering scale factor.
pub const SCALE: &str = "scale";

/// Animation frame rate multiplier.
pub const FRAMERATE: &str = "framerate";

/// Master entity name (must be active for this entity to function).
pub const MASTER: &str = "master";

/// Display / network name.
pub const NETNAME: &str = "netname";

/// Message text or sound sample name.
pub const MESSAGE: &str = "message";

/// Entity class name.
pub const CLASSNAME: &str = "classname";
