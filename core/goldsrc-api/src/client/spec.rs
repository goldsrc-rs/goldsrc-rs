//! Client and Player domain specifications and compile-time typestate witness tokens.

use crate::client::{Client, ClientExt, LifeState, Player, PlayerExt};
use crate::entity::EntityExt;
use crate::spec::{Refined, Spec, SpecError};

/// Typestate marker indicating a connected client slot (1..=32).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Connected;

/// Typestate marker indicating an AI bot client (`FL_FAKECLIENT`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bot;

/// Typestate marker indicating a human player (non-bot, non-HLTV).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Human;

/// Typestate marker indicating an HLTV relay proxy client (`FL_PROXY`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hltv;

/// Typestate marker indicating a living player character (`health > 0.0` and `life_state == ALIVE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Alive;

/// Typestate marker indicating a dead player character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Dead;

/// Typestate marker indicating a spectator client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Spectator;

// --- Spec Implementations for Client ---

impl Spec<Client> for Connected {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Client) -> Result<(), Self::Error> {
        if target.is_valid() {
            Ok(())
        } else {
            Err(SpecError::NotConnected)
        }
    }
}

impl Spec<Client> for Bot {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Client) -> Result<(), Self::Error> {
        if target.is_valid() && target.is_bot() {
            Ok(())
        } else {
            Err(SpecError::NotBot)
        }
    }
}

impl Spec<Client> for Human {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Client) -> Result<(), Self::Error> {
        if target.is_valid() && !target.is_bot() && !target.is_hltv() {
            Ok(())
        } else {
            Err(SpecError::NotHuman)
        }
    }
}

impl Spec<Client> for Hltv {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Client) -> Result<(), Self::Error> {
        if !target.is_valid() {
            return Err(SpecError::NotConnected);
        }
        if target.is_hltv() {
            Ok(())
        } else {
            Err(SpecError::NotHltv)
        }
    }
}

// --- Spec Implementations for Player ---

impl Spec<Player> for Connected {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Player) -> Result<(), Self::Error> {
        if target.is_valid() {
            Ok(())
        } else {
            Err(SpecError::NotConnected)
        }
    }
}

impl Spec<Player> for Alive {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Player) -> Result<(), Self::Error> {
        if target.is_valid() && target.is_alive() {
            Ok(())
        } else {
            Err(SpecError::NotAlive)
        }
    }
}

impl Spec<Player> for Dead {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Player) -> Result<(), Self::Error> {
        if target.is_valid() && !target.is_alive() {
            Ok(())
        } else {
            Err(SpecError::NotDead)
        }
    }
}

impl Spec<Player> for Hltv {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Player) -> Result<(), Self::Error> {
        if !target.is_valid() {
            return Err(SpecError::NotConnected);
        }
        if target.is_hltv() {
            Ok(())
        } else {
            Err(SpecError::NotHltv)
        }
    }
}

impl Spec<Player> for Bot {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Player) -> Result<(), Self::Error> {
        if target.is_valid() && target.is_bot() {
            Ok(())
        } else {
            Err(SpecError::NotBot)
        }
    }
}

impl Spec<Player> for Human {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Player) -> Result<(), Self::Error> {
        if target.is_valid() && !target.is_bot() && !target.is_hltv() {
            Ok(())
        } else {
            Err(SpecError::NotHuman)
        }
    }
}

impl Spec<Player> for Spectator {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Player) -> Result<(), Self::Error> {
        if target.is_valid() && target.life_state() == LifeState::Spectating {
            Ok(())
        } else {
            Err(SpecError::NotSpectator)
        }
    }
}

// --- Semantic Typestate Aliases ---

/// A frame-scoped, validated client slot guaranteed to be connected.
pub type ConnectedClient<'a> = Refined<'a, Client, Connected>;

/// A frame-scoped, validated client slot guaranteed to be a human player.
pub type HumanClient<'a> = Refined<'a, Client, Human>;

/// A frame-scoped, validated player character guaranteed to be alive.
pub type LivingPlayer<'a> = Refined<'a, Player, Alive>;

/// A frame-scoped, validated player character guaranteed to be a living human (non-bot).
pub type LivingHuman<'a> = Refined<'a, Player, (Alive, Human)>;

/// A frame-scoped, validated player character guaranteed to be dead.
pub type DeadPlayer<'a> = Refined<'a, Player, Dead>;

/// A frame-scoped, validated player character guaranteed to be in spectator mode.
pub type SpectatingPlayer<'a> = Refined<'a, Player, Spectator>;
