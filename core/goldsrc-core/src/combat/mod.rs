//! Dual-tier combat bridging and phased damage/killed interception.

pub mod bridge;

pub use bridge::{CombatBridge, CombatTier, KilledLayer, TakeDamageLayer};
