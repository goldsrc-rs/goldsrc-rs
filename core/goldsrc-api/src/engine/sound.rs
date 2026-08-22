//! Engine audio and sound playback operations.

/// Sound playback operations.
pub trait EngineSound: Send + Sync {
    /// Emit a dynamic sound attached to an entity.
    #[allow(clippy::too_many_arguments)]
    fn emit_sound(
        &self,
        entity: i32,
        channel: i32,
        sample: &str,
        volume: f32,
        attenuation: f32,
        flags: i32,
        pitch: i32,
    );

    /// Emit a static ambient sound originating from a specific world position.
    #[allow(clippy::too_many_arguments)]
    fn emit_ambient_sound(
        &self,
        entity: i32,
        pos: [f32; 3],
        sample: &str,
        volume: f32,
        attenuation: f32,
        flags: i32,
        pitch: i32,
    );
}
