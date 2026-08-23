//! Typestate zero-cost wrappers and state-verified guards.

use crate::client::Player;
use std::ops::{Deref, DerefMut};

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

/// Typestate extractor representing an HLTV relay proxy client (`FL_PROXY`).
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
