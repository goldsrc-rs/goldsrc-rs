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

    /// Player authentication ID / SteamID (e.g. "STEAM_0:1:12345678").
    fn player_auth_id(&self, _index: i32) -> Option<String> {
        None
    }

    /// Server-assigned unique user ID (`pfnGetPlayerUserId`).
    fn player_user_id(&self, _index: i32) -> u32 {
        0
    }

    /// Player IP address string without port (e.g. "192.168.1.50").
    fn player_ip(&self, _index: i32) -> Option<String> {
        None
    }

    /// Comprehensive player identity record.
    fn player_identity(&self, index: i32) -> crate::client::PlayerIdentity {
        let raw_auth = self
            .player_auth_id(index)
            .unwrap_or_else(|| "STEAM_ID_PENDING".to_string());
        let auth_state = if raw_auth == "STEAM_ID_PENDING" || raw_auth.is_empty() {
            crate::client::AuthState::Pending
        } else if let Some(steam_id) = crate::client::SteamId::parse(&raw_auth) {
            crate::client::AuthState::Authenticated(crate::client::AuthSubject::steam(steam_id))
        } else {
            crate::client::AuthState::Authenticated(crate::client::AuthSubject::external(
                "custom",
                raw_auth.clone(),
            ))
        };

        crate::client::PlayerIdentity {
            slot: index,
            user_id: self.player_user_id(index),
            raw_auth_id: raw_auth,
            auth_state,
            ip: self.player_ip(index),
            ping: 0,
            packet_loss: 0,
            is_bot: false,
            is_hltv: false,
        }
    }

    /// Player game team slot (0=Unassigned, 1=Terrorist, 2=CT, 3=Spectator).
    fn player_team(&self, _index: i32) -> i32 {
        0
    }

    /// Player preferred language (from `setinfo _lang` or server default).
    fn player_lang(&self, _index: i32) -> Option<String> {
        None
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

    /// Sets a key-value attribute on an entity (dispatched via `pfnKeyValue`).
    /// Returns `true` if handled by the GameDLL.
    fn entity_key_value(&self, _index: i32, _key: &str, _value: &str) -> bool {
        false
    }

    /// Forces the real GameDLL's Touch between two entities
    /// (`touched` delivered into `other`, e.g. weapon → player).
    fn dispatch_touch(&self, touched: i32, other: i32);

    /// Constructs a safe Player entity handle from a player slot index if valid.
    fn player_handle(&self, index: i32) -> Option<crate::client::Player> {
        if (1..=32).contains(&index) && self.entity_is_valid(index) {
            #[cfg(not(target_arch = "wasm32"))]
            {
                Some(crate::client::Player::from_index(index))
            }
            #[cfg(target_arch = "wasm32")]
            {
                Some(crate::client::Player::new(index))
            }
        } else {
            None
        }
    }
}
