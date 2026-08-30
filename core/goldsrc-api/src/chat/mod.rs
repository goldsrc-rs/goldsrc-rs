//! In-game Chat Interception, Filtering, and Safe Packet Chunking.

use crate::client::Player;

/// Team targeting filter for chat messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TeamTarget {
    /// Broadcast to all teams / public.
    #[default]
    All,
    /// Sent only to teammates of the sender.
    SameTeam,
    /// Sent only to players of the opposite team.
    OppositeTeam,
    /// Sent to a specific player slot (1..32).
    Direct(i32),
}

/// Life state filter for recipient players.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LifeStateFilter {
    /// Delivered to players in any life state (alive or dead).
    #[default]
    Any,
    /// Delivered only to alive players.
    AliveOnly,
    /// Delivered only to dead players and spectators.
    DeadOnly,
}

/// Structured visibility scope for dispatched chat messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChatScope {
    /// Team routing target.
    pub team: TeamTarget,
    /// Player life state filter.
    pub state: LifeStateFilter,
}

impl ChatScope {
    /// Creates a default public scope (all players, any life state).
    pub const fn all() -> Self {
        Self {
            team: TeamTarget::All,
            state: LifeStateFilter::Any,
        }
    }

    /// Creates a team-only chat scope for teammates.
    pub const fn same_team() -> Self {
        Self {
            team: TeamTarget::SameTeam,
            state: LifeStateFilter::Any,
        }
    }

    /// Creates an opposite-team chat scope.
    pub const fn opposite_team() -> Self {
        Self {
            team: TeamTarget::OppositeTeam,
            state: LifeStateFilter::Any,
        }
    }

    /// Creates a direct private chat scope to a specific player.
    pub const fn direct(slot: i32) -> Self {
        Self {
            team: TeamTarget::Direct(slot),
            state: LifeStateFilter::Any,
        }
    }

    /// Constrains this chat scope to alive recipients only.
    pub const fn alive_only(mut self) -> Self {
        self.state = LifeStateFilter::AliveOnly;
        self
    }

    /// Constrains this chat scope to dead recipients and spectators only.
    pub const fn dead_only(mut self) -> Self {
        self.state = LifeStateFilter::DeadOnly;
        self
    }
}

/// Structured chat message in the interceptor middleware pipeline.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// Sender player entity.
    pub sender: Player,
    /// Target visibility scope.
    pub scope: ChatScope,
    /// Raw unformatted message typed by player.
    pub raw_text: String,
    /// Prefix tag (e.g. `^3[VIP]^1 `, `^4[Admin]^1 `).
    pub prefix: Option<String>,
    /// Formatted content with active color codes and placeholder expansions.
    pub formatted_text: String,
    /// Whether the message has been blocked by an interceptor (e.g. anti-spam / mute).
    pub is_blocked: bool,
}

impl ChatMessage {
    /// Creates a new `ChatMessage` instance from incoming player say command.
    pub fn new(sender: Player, raw_text: &str, scope: ChatScope) -> Self {
        Self {
            sender,
            scope,
            raw_text: raw_text.to_string(),
            prefix: None,
            formatted_text: raw_text.to_string(),
            is_blocked: false,
        }
    }

    /// Appends a prefix tag to the formatted output.
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    /// Blocks this message from being broadcast to other clients.
    pub fn block(&mut self) {
        self.is_blocked = true;
    }
}

/// Maximum safe payload size in bytes for a single `SayText` user message.
/// GoldSrc engine limit is 192 bytes; reserving bytes for sender ID and NUL-terminator.
pub const MAX_SAYTEXT_PAYLOAD_LEN: usize = 180;

/// Splits a long formatted chat message into safe packets (<= 180 bytes),
/// preserving active color tags across multi-line splits.
pub fn split_chat_chunks(message: &str) -> Vec<String> {
    if message.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut active_color: char = '1'; // Default GoldSrc yellow/white
    let mut chars = message.chars().peekable();

    while let Some(ch) = chars.next() {
        // Track current color code (^1..^4 or \x01..\x04)
        if ch == '^' {
            if let Some(&next_c) = chars.peek()
                && ('1'..='4').contains(&next_c)
            {
                active_color = next_c;
            }
        } else if ('\x01'..='\x04').contains(&ch) {
            active_color = match ch {
                '\x01' => '1',
                '\x02' => '2',
                '\x03' => '3',
                '\x04' => '4',
                _ => '1',
            };
        }

        // Test if adding character exceeds byte budget (180 bytes)
        if current_chunk.len() + ch.len_utf8() > MAX_SAYTEXT_PAYLOAD_LEN {
            chunks.push(current_chunk.clone());
            current_chunk.clear();
            // Prefix continuation chunk with active color
            current_chunk.push('^');
            current_chunk.push(active_color);
            current_chunk.push_str("... ");
        }

        current_chunk.push(ch);
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_scope_composition() {
        let team_dead = ChatScope::same_team().dead_only();
        assert_eq!(team_dead.team, TeamTarget::SameTeam);
        assert_eq!(team_dead.state, LifeStateFilter::DeadOnly);

        let opp_alive = ChatScope::opposite_team().alive_only();
        assert_eq!(opp_alive.team, TeamTarget::OppositeTeam);
        assert_eq!(opp_alive.state, LifeStateFilter::AliveOnly);
    }

    #[test]
    fn test_split_chat_chunks_short() {
        let msg = "^3[GoldSrc.rs]^1 Hello, world!";
        let chunks = split_chat_chunks(msg);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], msg);
    }

    #[test]
    fn test_split_chat_chunks_long_preserves_color() {
        let long_msg = format!("^4[Server]^3 {}", "A".repeat(350));
        let chunks = split_chat_chunks(&long_msg);
        assert!(chunks.len() >= 2);
        assert!(chunks[0].starts_with("^4[Server]^3"));
        assert!(chunks[1].starts_with("^3... "));
    }
}
