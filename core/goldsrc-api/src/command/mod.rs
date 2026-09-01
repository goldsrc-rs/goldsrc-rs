//! Command routing targets, scope filters, and typed argument extractors.

pub mod builder;
pub mod error;

pub use builder::{Command, CommandBuilder};
pub use error::{CommandContext, CommandError, CommandResult};

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

/// Splits a command argument line into tokens respecting quotes (`"..."`).
/// If quotes are unclosed, takes the remaining content without trailing quote.
pub fn split_command_args(args: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = args.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }

        if c == '"' {
            chars.next(); // Consume opening quote
            let mut current = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == '"' {
                    chars.next(); // Consume closing quote
                    break;
                }
                current.push(ch);
                chars.next();
            }
            tokens.push(current);
        } else {
            let mut current = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() {
                    break;
                }
                current.push(ch);
                chars.next();
            }
            tokens.push(current);
        }
    }

    tokens
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_command_args_quoted_and_unquoted() {
        let args = r#"hello "world test" 123 "another string""#;
        let tokens = split_command_args(args);
        assert_eq!(tokens, vec!["hello", "world test", "123", "another string"]);
    }

    #[test]
    fn test_split_command_args_cyrillic_quotes() {
        let args = r#""Привет в developer область!""#;
        let tokens = split_command_args(args);
        assert_eq!(tokens, vec!["Привет в developer область!"]);
    }
}
