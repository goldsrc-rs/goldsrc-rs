//! Dynamic Contextual Placeholder Engine abstractions and metadata definitions.

use crate::client::Player;
use crate::dsl::PlaceholderCall;

/// Target player resolution strategy for placeholder function calls (e.g. `{ip(target='PlayerName')}`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerTarget {
    /// Resolved by 1-based slot index (1..32).
    Slot(i32),
    /// Resolved by UserID (e.g. `#12`).
    UserId(i32),
    /// Resolved by player display name or substring match.
    Name(String),
    /// Resolved by AuthID / SteamID (e.g. `STEAM_0:0:12345`).
    AuthId(String),
}

impl PlayerTarget {
    /// Parses target argument string into a `PlayerTarget` variant.
    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }

        if let Some(rest) = trimmed.strip_prefix('#')
            && let Ok(uid) = rest.parse::<i32>()
        {
            return Some(PlayerTarget::UserId(uid));
        }

        if let Ok(slot) = trimmed.parse::<i32>()
            && (1..=32).contains(&slot)
        {
            return Some(PlayerTarget::Slot(slot));
        }

        if trimmed.starts_with("STEAM_")
            || trimmed.starts_with("VALVE_")
            || trimmed.starts_with("BOT")
        {
            return Some(PlayerTarget::AuthId(trimmed.to_string()));
        }

        Some(PlayerTarget::Name(trimmed.to_string()))
    }
}

/// Metadata exported by a plugin for registered placeholders.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct PlaceholderMetadata {
    /// Primary placeholder identifier name (e.g. `rank`, `ip`, `kills`).
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Usage example / parameter signature (e.g. `{rank(format='short')}`).
    #[serde(default)]
    pub usage: String,
    /// List of alternative names or aliases.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Optional required capability for callers to resolve this placeholder.
    #[serde(default)]
    pub capability: Option<String>,
}

/// Trait implemented by placeholder provider callbacks.
pub trait PlaceholderHandler: Send + Sync {
    /// Evaluates the placeholder function for a given caller and parsed call arguments.
    fn evaluate(&self, caller: Player, call: &PlaceholderCall) -> String;
}

impl<F> PlaceholderHandler for F
where
    F: Fn(Player, &PlaceholderCall) -> String + Send + Sync,
{
    fn evaluate(&self, caller: Player, call: &PlaceholderCall) -> String {
        self(caller, call)
    }
}
