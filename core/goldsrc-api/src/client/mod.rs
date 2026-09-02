//! Core player and client domain abstractions, states, and typestate guards.

pub mod guards;
pub mod player;
pub mod types;

pub use guards::{Alive, Bot, Dead, HLTV, Spectator};
pub use player::Player;
pub use types::{AsLangCode, ClientKind, ConnectionState, LifeState, PrintTarget, Team};
