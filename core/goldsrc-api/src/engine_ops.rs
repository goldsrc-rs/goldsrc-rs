//! Narrow object-safe engine bridge used by the WASM host.
//!
//! Unlike the full [`crate::Engine`] trait (which returns rich [`crate::Entity`]
//! / [`crate::Player`] handles), this trait only deals with primitive values
//! (`i32`, `f32`, `String`) so it can be shared across the WASM host boundary
//! as `Arc<dyn EngineOps>`.

/// Engine bridge for the WASM host: object-safe, `Send + Sync`.
///
/// Every method maps 1:1 to a `host-*` function in the WIT interface, so the
/// WASM host delegates real engine state instead of returning mock constants.
pub trait EngineOps: Send + Sync {
    /// Whether an entity index is valid (0 = world, 1..=N = players).
    fn entity_is_valid(&self, index: i32) -> bool;
    /// Entity classname, if the index is valid.
    fn entity_classname(&self, index: i32) -> Option<String>;
    /// Entity health (0.0 if invalid).
    fn entity_health(&self, index: i32) -> f32;
    /// Set an entity's health.
    fn entity_set_health(&self, index: i32, health: f32);
    /// Entity origin as `[x, y, z]`.
    fn entity_origin(&self, index: i32) -> [f32; 3];
    /// Set an entity's origin.
    fn entity_set_origin(&self, index: i32, pos: [f32; 3]);
    /// Entity velocity as `[x, y, z]`.
    fn entity_velocity(&self, index: i32) -> [f32; 3];
    /// Set an entity's velocity.
    fn entity_set_velocity(&self, index: i32, vel: [f32; 3]);
    /// Player name, if the index is a valid player.
    fn player_name(&self, index: i32) -> Option<String>;
    /// Player armor value (0.0 if invalid).
    fn player_armorvalue(&self, index: i32) -> f32;
    /// Set a player's armor value.
    fn player_set_armorvalue(&self, index: i32, armor: f32);
}
