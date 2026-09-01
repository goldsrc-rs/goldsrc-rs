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

/// Style of multi-line continuation prefixes for split chat packets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WrapStyle {
    /// Single continuation prefix for all continued lines (e.g. "... " or "⤷ ").
    Single(String),
    /// Tree/hierarchical branching format:
    /// Line 1:   [Admin] Player : Initial line
    /// Line 2:   ├── second line...
    /// Line N-1: ├── next line...
    /// Line N:   └── last line
    Tree {
        middle_prefix: String,
        last_prefix: String,
    },
    /// No continuation prefix.
    None,
}

impl Default for WrapStyle {
    fn default() -> Self {
        WrapStyle::Tree {
            middle_prefix: "|-- ".to_string(),
            last_prefix: "\\-- ".to_string(),
        }
    }
}

/// Maximum safe payload size in bytes for a single `SayText` user message.
/// GoldSrc engine limit is 192 bytes; reserving bytes for sender ID and NUL-terminator.
pub const MAX_SAYTEXT_PAYLOAD_LEN: usize = 180;

/// Splits a long formatted chat message into safe packets (<= 180 bytes) using default Tree WrapStyle.
pub fn split_chat_chunks(message: &str) -> Vec<String> {
    split_chat_chunks_with_style(message, &WrapStyle::default())
}

/// Splits a long formatted chat message into safe packets (<= 180 bytes) with a customized WrapStyle,
/// supporting both byte limit splitting and explicit newline `\n` splitting.
pub fn split_chat_chunks_with_style(message: &str, style: &WrapStyle) -> Vec<String> {
    if message.is_empty() {
        return Vec::new();
    }

    // Determine maximum prefix overhead across continuation chunks
    let max_prefix_len = match style {
        WrapStyle::Single(p) => p.len() + 2, // +2 for ^X
        WrapStyle::Tree {
            middle_prefix,
            last_prefix,
        } => middle_prefix.len().max(last_prefix.len()) + 2,
        WrapStyle::None => 0,
    };
    let chunk_limit = MAX_SAYTEXT_PAYLOAD_LEN.saturating_sub(max_prefix_len);

    // 1. First split raw message by explicit newlines `\n` or `\r\n`
    let raw_lines: Vec<&str> = message.split('\n').collect();
    let mut raw_chunks = Vec::new();
    let mut active_color: char = '1';

    for line in raw_lines {
        let clean_line = line.trim_end_matches('\r');
        if clean_line.is_empty() {
            continue;
        }

        let mut current_chunk = String::new();
        let mut chars = clean_line.chars().peekable();

        while let Some(ch) = chars.next() {
            let mut seq = String::new();
            seq.push(ch);

            if ch == '^' {
                if let Some(&next_c) = chars.peek()
                    && ('1'..='4').contains(&next_c)
                {
                    active_color = next_c;
                    seq.push(chars.next().unwrap());
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

            if current_chunk.len() + seq.len() > chunk_limit && !current_chunk.is_empty() {
                raw_chunks.push((current_chunk.clone(), active_color));
                current_chunk.clear();
            }

            current_chunk.push_str(&seq);
        }

        if !current_chunk.is_empty() {
            raw_chunks.push((current_chunk, active_color));
        }
    }

    let total = raw_chunks.len();
    if total <= 1 {
        return raw_chunks.into_iter().map(|(c, _)| c).collect();
    }

    let mut formatted_chunks = Vec::with_capacity(total);
    for (i, (chunk_text, color)) in raw_chunks.into_iter().enumerate() {
        if i == 0 {
            formatted_chunks.push(chunk_text);
        } else {
            let prefix = match style {
                WrapStyle::Single(p) => p.as_str(),
                WrapStyle::Tree {
                    middle_prefix,
                    last_prefix,
                } => {
                    if i == total - 1 {
                        last_prefix.as_str()
                    } else {
                        middle_prefix.as_str()
                    }
                }
                WrapStyle::None => "",
            };

            let mut chunk = String::with_capacity(chunk_text.len() + prefix.len() + 4);
            if !prefix.is_empty() {
                chunk.push('^');
                chunk.push(color);
                chunk.push_str(prefix);
            }
            chunk.push_str(&chunk_text);
            formatted_chunks.push(chunk);
        }
    }

    formatted_chunks
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
    fn test_split_chat_chunks_tree_wrap() {
        let lines = "Line 1\nLine 2\nLine 3";
        let chunks = split_chat_chunks(lines);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], "Line 1");
        assert_eq!(chunks[1], "^1|-- Line 2");
        assert_eq!(chunks[2], "^1\\-- Line 3");
    }

    #[test]
    fn test_split_chat_chunks_long_preserves_color() {
        let long_msg = format!("^4[Server]^3 {}", "A".repeat(350));
        let chunks = split_chat_chunks(&long_msg);
        assert!(chunks.len() >= 2);
        assert!(chunks[0].starts_with("^4[Server]^3"));
        assert!(chunks[1].starts_with("^3|-- "));
        assert!(chunks.last().unwrap().starts_with("^3\\-- "));
    }
}
