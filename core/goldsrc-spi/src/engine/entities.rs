//! Engine entity management operations.

use goldsrc_api::client::{AuthState, AuthSubject, Player, PlayerIdentity, SteamId};
use goldsrc_api::entity::EntitySpawner;

/// Operations for querying and manipulating entities and players.
pub trait EngineEntities: EntitySpawner + Send + Sync {
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

    /// Entity velocity vector as `[x, y, z]`.
    fn entity_velocity(&self, index: i32) -> [f32; 3];

    /// Set an entity's velocity.
    fn entity_set_velocity(&self, index: i32, vel: [f32; 3]);

    /// Entity Euler angles as `[pitch, yaw, roll]`.
    fn entity_angles(&self, index: i32) -> [f32; 3];

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
    fn player_identity(&self, index: i32) -> PlayerIdentity {
        let raw_auth = self
            .player_auth_id(index)
            .unwrap_or_else(|| "STEAM_ID_PENDING".to_string());
        let auth_state = if raw_auth == "STEAM_ID_PENDING" || raw_auth.is_empty() {
            AuthState::Pending
        } else if let Some(steam_id) = SteamId::parse(&raw_auth) {
            AuthState::Authenticated(AuthSubject::steam(steam_id))
        } else {
            AuthState::Authenticated(AuthSubject::external("custom", raw_auth.clone()))
        };

        PlayerIdentity {
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

    /// Remove an entity from the world.
    fn remove_entity(&self, index: i32);

    /// Drop an entity to the floor beneath it.
    /// Returns 1 if grounded, 0 if stuck/freefall.
    fn drop_to_floor(&self, index: i32) -> i32;

    /// Forces the real GameDLL's Touch between two entities
    /// (`touched` delivered into `other`, e.g. weapon → player).
    fn dispatch_touch(&self, touched: i32, other: i32);

    /// Constructs a safe Player entity handle from a player slot index if valid.
    fn player_handle(&self, index: i32) -> Option<Player> {
        if (1..=32).contains(&index) && self.entity_is_valid(index) {
            #[cfg(not(target_arch = "wasm32"))]
            {
                Some(Player::from_index(index))
            }
            #[cfg(target_arch = "wasm32")]
            {
                Some(Player::new(index))
            }
        } else {
            None
        }
    }
}
