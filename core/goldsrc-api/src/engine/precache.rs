//! Engine precache operations.

/// Operations for precaching models, sounds, and generic assets.
pub trait EnginePrecache: Send + Sync {
    /// Precache a model file (e.g. "models/player/vip/vip.mdl").
    /// Returns model index.
    fn precache_model(&self, path: &str) -> i32;

    /// Precache a sound file (e.g. "weapons/c4_beep1.wav").
    /// Returns sound index.
    fn precache_sound(&self, path: &str) -> i32;

    /// Precache a generic asset (e.g. sprites, soundscapes).
    /// Returns asset index.
    fn precache_generic(&self, path: &str) -> i32;
}
