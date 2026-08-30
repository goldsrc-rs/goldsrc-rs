//! Engine entity management operations.

/// Operations for querying and manipulating entities and players.
pub trait EngineEntities: Send + Sync {
    /// Whether an entity index is valid (0 = world, 1..=N = players, >N = entities).
    fn entity_is_valid(&self, index: i32) -> bool;

    /// Entity classname (e.g. "info_player_start", "hostage_entity").
    fn entity_classname(&self, index: i32) -> Option<String>;

    /// Entity health value.
    fn entity_health(&self, index: i32) -> f32;

    /// Set an entity's health.
    fn entity_set_health(&self, index: i32, health: f32);

    /// Entity origin coordinates as `[x, y, z]`.
    fn entity_origin(&self, index: i32) -> [f32; 3];

    /// Set an entity's world position.
    fn entity_set_origin(&self, index: i32, pos: [f32; 3]);

    /// Entity velocity vector as `[x, y, z]`.
    fn entity_velocity(&self, index: i32) -> [f32; 3];

    /// Set an entity's velocity.
    fn entity_set_velocity(&self, index: i32, vel: [f32; 3]);

    /// Entity Euler angles as `[pitch, yaw, roll]`.
    fn entity_angles(&self, index: i32) -> [f32; 3];

    /// Set an entity's rotation angles.
    fn entity_set_angles(&self, index: i32, angles: [f32; 3]);

    /// Player display name (e.g. "Player").
    fn player_name(&self, index: i32) -> Option<String>;

    /// Player game team slot (0=Unassigned, 1=Terrorist, 2=CT, 3=Spectator).
    fn player_team(&self, _index: i32) -> i32 {
        0
    }

    /// Player armor value.
    fn player_armorvalue(&self, index: i32) -> f32;

    /// Set a player's armor value.
    fn player_set_armorvalue(&self, index: i32, armor: f32);

    /// Create a new named entity (e.g. "env_sprite", "info_target").
    /// Returns the newly allocated entity index.
    fn create_named_entity(&self, classname: &str) -> Option<i32>;

    /// Remove an entity from the world.
    fn remove_entity(&self, index: i32);

    /// Drop an entity to the floor beneath it.
    /// Returns 1 if grounded, 0 if stuck/freefall.
    fn drop_to_floor(&self, index: i32) -> i32;

    /// Runs the real GameDLL's DispatchSpawn for an entity by index.
    /// Returns the GameDLL result (0 when no GameDLL bridge is available).
    fn dispatch_spawn(&self, index: i32) -> i32;

    /// Forces the real GameDLL's Touch between two entities
    /// (`touched` delivered into `other`, e.g. weapon → player).
    fn dispatch_touch(&self, touched: i32, other: i32);
}
