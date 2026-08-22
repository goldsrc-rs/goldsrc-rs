//! Command routing targets, scope filters, and typed argument extractors.

use crate::client::{Alive, Dead, Player};

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
// FromArg Trait & Parsers
// ---------------------------------------------------------------------------

/// Trait for types that can be parsed from a command argument string token.
pub trait FromArg: Sized {
    /// Attempts to parse `Self` from a command argument string token.
    fn from_arg(token: &str) -> Result<Self, String>;
}

macro_rules! impl_from_arg_from_str {
    ($($t:ty),*) => {
        $(
            impl FromArg for $t {
                fn from_arg(token: &str) -> Result<Self, String> {
                    token.parse::<$t>().map_err(|e| format!("invalid {}: {e}", stringify!($t)))
                }
            }
        )*
    };
}

impl_from_arg_from_str!(
    i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64, bool
);

impl FromArg for String {
    fn from_arg(token: &str) -> Result<Self, String> {
        Ok(token.to_string())
    }
}

impl FromArg for Player {
    fn from_arg(token: &str) -> Result<Self, String> {
        if let Ok(idx) = token.parse::<i32>() {
            let p = Player::new(idx);
            if p.is_valid() {
                return Ok(p);
            }
        }
        Err(format!("player with index '{token}' is not connected"))
    }
}

impl FromArg for Alive<Player> {
    fn from_arg(token: &str) -> Result<Self, String> {
        let p = Player::from_arg(token)?;
        if p.is_alive() {
            Ok(Alive(p))
        } else {
            Err(format!(
                "player '{}' is dead (expected living player)",
                token
            ))
        }
    }
}

impl FromArg for Dead<Player> {
    fn from_arg(token: &str) -> Result<Self, String> {
        let p = Player::from_arg(token)?;
        if !p.is_alive() {
            Ok(Dead(p))
        } else {
            Err(format!(
                "player '{}' is alive (expected dead player)",
                token
            ))
        }
    }
}
