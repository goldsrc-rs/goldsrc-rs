//! Domain entity records for player sanctions and moderation history.

use serde::{Deserialize, Serialize};

/// Type of moderation sanction applied to a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Server ban (temporary or permanent).
    Ban,
    /// Mute of voice network transmission.
    MuteVoice,
    /// Mute of in-game chat and radio messages.
    MuteChat,
    /// Combined voice and chat communication block.
    Gag,
    /// Movement freeze for cheat/afk inspection.
    Freeze,
    /// Direct slay (instant death).
    Slay,
    /// Physical impulse slap with optional damage.
    Slap,
    /// Network connection disconnect.
    Kick,
}

impl ActionType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ban => "ban",
            Self::MuteVoice => "mute_voice",
            Self::MuteChat => "mute_chat",
            Self::Gag => "gag",
            Self::Freeze => "freeze",
            Self::Slay => "slay",
            Self::Slap => "slap",
            Self::Kick => "kick",
        }
    }
}

/// Audit and enforcement record representing a moderation action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerationRecord {
    /// Unique record ID.
    pub id: u64,
    /// Target SteamID or hardware hash (if resolved).
    pub target_auth: String,
    /// Target network IP address or subnet (if resolved).
    pub target_ip: String,
    /// Target display name at the time of punishment.
    pub target_name: String,
    /// AuthID or name of the moderator who issued the sanction.
    pub issuer_auth: String,
    /// Sanction category.
    pub action_type: ActionType,
    /// Administrative reason.
    pub reason: String,
    /// UNIX timestamp of creation.
    pub created_at: u64,
    /// UNIX timestamp of expiration (0 = permanent).
    pub expires_at: u64,
    /// Whether the sanction is currently active.
    pub active: bool,
}

impl ModerationRecord {
    /// Returns `true` if this record is currently in effect at `current_time`.
    pub fn is_currently_active(&self, current_time: u64) -> bool {
        if !self.active {
            return false;
        }
        if self.expires_at == 0 {
            return true; // Permanent
        }
        current_time < self.expires_at
    }
}
