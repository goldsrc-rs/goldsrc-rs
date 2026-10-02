//! Player menu session tracking and debounce enforcement.

use std::collections::HashMap;

/// Active menu session for a player.
#[derive(Debug, Clone)]
pub struct PlayerMenuSession {
    pub current_page: usize,
    pub total_pages: usize,
    pub last_interaction_time: f32,
}

/// Thread-safe session tracker.
#[derive(Default)]
pub struct SessionManager {
    sessions: HashMap<i32, PlayerMenuSession>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_session(&mut self, player_index: i32, total_pages: usize, now: f32) {
        self.sessions.insert(
            player_index,
            PlayerMenuSession {
                current_page: 0,
                total_pages,
                last_interaction_time: now,
            },
        );
    }

    pub fn check_debounce(&mut self, player_index: i32, now: f32, min_interval_secs: f32) -> bool {
        if let Some(session) = self.sessions.get_mut(&player_index) {
            if now - session.last_interaction_time < min_interval_secs {
                return false; // Debounced
            }
            session.last_interaction_time = now;
            true
        } else {
            true
        }
    }

    pub fn end_session(&mut self, player_index: i32) {
        self.sessions.remove(&player_index);
    }
}
