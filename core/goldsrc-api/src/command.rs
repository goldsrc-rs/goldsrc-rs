//! Command routing targets, scope filters, teams, and typestate extractors.

use crate::Player;
use std::ops::{Deref, DerefMut};

/// Scope for in-game chat command execution (`say` vs `say_team`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatScope {
    /// Public chat (`say`).
    All,
    /// Team chat (`say_team`).
    Team,
    /// Both public and team chat.
    Both,
}

/// Filter for the player's life state when executing chat commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStateFilter {
    /// Any player state (alive or dead).
    Any,
    /// Only living players.
    AliveOnly,
    /// Only dead players / spectators.
    DeadOnly,
}

/// Game team identifiers (compatible with Counter-Strike 1.6 team slots).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Team {
    /// Unassigned / choosing team.
    Unassigned = 0,
    /// Terrorist team (T).
    Terrorist = 1,
    /// Counter-Terrorist team (CT).
    CounterTerrorist = 2,
    /// Spectator team (SPEC).
    Spectator = 3,
}

impl From<i32> for Team {
    fn from(val: i32) -> Self {
        match val {
            1 => Team::Terrorist,
            2 => Team::CounterTerrorist,
            3 => Team::Spectator,
            _ => Team::Unassigned,
        }
    }
}

/// Execution target and ingress channel for commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandTarget {
    /// Server console only (`HLDS`).
    Server,
    /// Client console only (`~` / `F10`).
    ClientConsole,
    /// In-game chat trigger (e.g. `/vip`, `!vip`, `rtv`).
    Chat {
        /// Applicable chat channels.
        scope: ChatScope,
        /// Life state requirement for the caller.
        filter: PlayerStateFilter,
        /// If `true`, suppresses the trigger message from broadcasting in public chat.
        silent: bool,
    },
    /// Custom `messagemode` prompt response.
    MessageMode(String),
    /// Callable from any target or channel.
    Any,
}

// ---------------------------------------------------------------------------
// Typestate Wrappers & Guards
// ---------------------------------------------------------------------------

/// Typestate extractor guaranteeing that the wrapped player/entity is currently alive (`health > 0`).
#[derive(Debug, Clone, PartialEq)]
pub struct Alive<T>(pub T);

impl<T> Deref for Alive<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Alive<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Typestate extractor guaranteeing that the wrapped player/entity is currently dead (`health <= 0`).
#[derive(Debug, Clone, PartialEq)]
pub struct Dead<T>(pub T);

impl<T> Deref for Dead<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Dead<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Typestate extractor guaranteeing that the caller is on the Terrorist team.
#[derive(Debug, Clone, PartialEq)]
pub struct Terrorist(pub Player);

impl Deref for Terrorist {
    type Target = Player;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Terrorist {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Typestate extractor guaranteeing that the caller is on the Counter-Terrorist team.
#[derive(Debug, Clone, PartialEq)]
pub struct CounterTerrorist(pub Player);

impl Deref for CounterTerrorist {
    type Target = Player;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CounterTerrorist {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Typestate extractor guaranteeing that the caller is on the Spectator team.
#[derive(Debug, Clone, PartialEq)]
pub struct Spectator(pub Player);

impl Deref for Spectator {
    type Target = Player;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Spectator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Typestate extractor representing an AI Bot client (`FL_FAKECLIENT`).
#[derive(Debug, Clone, PartialEq)]
pub struct Bot(pub Player);

impl Deref for Bot {
    type Target = Player;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Bot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Typestate extractor representing an HLTV relay proxy client.
#[derive(Debug, Clone, PartialEq)]
pub struct HLTV(pub Player);

impl Deref for HLTV {
    type Target = Player;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HLTV {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
