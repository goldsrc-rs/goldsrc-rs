use crate::Player;

/// Event fired when a player takes damage.
#[derive(Debug, Clone)]
pub struct DamageEvent {
    /// Index of the damaged entity.
    pub victim_index: i32,
    /// Index of the attacker.
    pub attacker_index: i32,
    /// Amount of damage dealt.
    pub damage: f32,
    /// Damage type bitmask (e.g. `DMG_BULLET`).
    pub damage_type: i32,
}

impl DamageEvent {
    /// Returns the damaged player.
    pub fn victim(&self) -> Player {
        Player::new(self.victim_index)
    }

    /// Returns the attacking player.
    pub fn attacker(&self) -> Player {
        Player::new(self.attacker_index)
    }
}

/// Event fired when a player spawns.
#[derive(Debug, Clone)]
pub struct PlayerSpawnEvent {
    /// Index of the spawning player.
    pub player_index: i32,
}

impl PlayerSpawnEvent {
    /// Returns the spawning player.
    pub fn player(&self) -> Player {
        Player::new(self.player_index)
    }
}

/// Event fired when a client connects to the server.
#[derive(Debug, Clone)]
pub struct ClientPutInServerEvent {
    /// Index of the connecting player.
    pub player_index: i32,
}

impl ClientPutInServerEvent {
    /// Returns the connecting player.
    pub fn player(&self) -> Player {
        Player::new(self.player_index)
    }
}

/// Command and Event execution context.
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// Index of the player who executed the command, if any.
    pub executor_index: Option<i32>,
}

impl Context {
    /// Returns the executing player, if any.
    pub fn executor(&self) -> Option<Player> {
        self.executor_index.map(Player::new)
    }
}
