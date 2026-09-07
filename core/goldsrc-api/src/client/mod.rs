//! Core player and client domain abstractions, states, and typestate guards.

pub mod ext;
pub mod guards;
pub mod player;
pub mod property;
pub mod types;

pub use crate::entity::EntityExt;
pub use ext::{ClientExt, PlayerExt};
pub use guards::{Alive, Bot, Dead, Hltv, Spectator};
pub use player::Player;
pub use property::{Lang, Name};
pub use types::{AsLangCode, ClientKind, ConnectionState, LifeState, PrintTarget, Team};
