//! Core player and client domain abstractions, states, and typestate guards.

pub mod ext;
pub mod player;
pub mod property;
pub mod types;

pub use crate::entity::EntityExt;
pub use crate::spec::{Alive, Bot, Connected, Dead, Hltv, Human, Spectator};
pub use ext::{ClientExt, PlayerExt};
pub use player::Player;
pub use property::{Lang, Name};
pub use types::{AsLangCode, ClientKind, ConnectionState, LifeState, PrintTarget, Team};
