use crate::Player;

/// Event fired when a player takes damage.
#[derive(Debug, Clone)]
pub struct DamageEvent {
    pub victim_index: i32,
    pub attacker_index: i32,
    pub damage: f32,
    pub damage_type: i32,
}

impl DamageEvent {
    pub fn victim(&self) -> Player {
        Player::new(self.victim_index)
    }

    pub fn attacker(&self) -> Player {
        Player::new(self.attacker_index)
    }
}

/// Event fired when a player spawns.
#[derive(Debug, Clone)]
pub struct PlayerSpawnEvent {
    pub player_index: i32,
}

impl PlayerSpawnEvent {
    pub fn player(&self) -> Player {
        Player::new(self.player_index)
    }
}

/// Event fired when a client connects to the server.
#[derive(Debug, Clone)]
pub struct ClientPutInServerEvent {
    pub player_index: i32,
}

impl ClientPutInServerEvent {
    pub fn player(&self) -> Player {
        Player::new(self.player_index)
    }
}

/// Command and Event execution context.
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub executor_index: Option<i32>,
}

impl Context {
    pub fn executor(&self) -> Option<Player> {
        self.executor_index.map(Player::new)
    }
}
