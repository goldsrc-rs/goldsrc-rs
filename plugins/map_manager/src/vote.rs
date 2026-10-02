//! Interactive map voting session and result tallying.

use std::collections::HashMap;

/// Active map voting session.
#[derive(Debug)]
pub struct MapVoteSession {
    pub options: Vec<String>,
    pub votes: HashMap<i32, usize>, // player_index -> option_index
    pub start_time: f32,
    pub duration_secs: f32,
    pub is_active: bool,
    pub is_concluded: bool,
    pub winning_map: Option<String>,
}

impl Default for MapVoteSession {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            votes: HashMap::new(),
            start_time: 0.0,
            duration_secs: 15.0,
            is_active: false,
            is_concluded: false,
            winning_map: None,
        }
    }
}

impl MapVoteSession {
    pub fn start(&mut self, options: Vec<String>, now: f32, duration_secs: f32) {
        self.options = options;
        self.votes.clear();
        self.start_time = now;
        self.duration_secs = duration_secs;
        self.is_active = true;
        self.is_concluded = false;
        self.winning_map = None;
    }

    pub fn cast_vote(&mut self, player_index: i32, option_idx: usize) -> bool {
        if !self.is_active || option_idx >= self.options.len() {
            return false;
        }
        self.votes.insert(player_index, option_idx);
        true
    }

    /// Evaluates if vote deadline has been reached and selects winner.
    pub fn check_conclude(&mut self, now: f32) -> Option<String> {
        if !self.is_active || self.is_concluded {
            return None;
        }

        if now - self.start_time >= self.duration_secs {
            self.is_active = false;
            self.is_concluded = true;

            let mut counts = vec![0u32; self.options.len()];
            for &opt in self.votes.values() {
                if opt < counts.len() {
                    counts[opt] += 1;
                }
            }

            let mut max_idx = 0;
            let mut max_votes = 0;
            for (idx, &cnt) in counts.iter().enumerate() {
                if cnt > max_votes {
                    max_votes = cnt;
                    max_idx = idx;
                }
            }

            let winner = self
                .options
                .get(max_idx)
                .cloned()
                .unwrap_or_else(|| "crossfire".to_string());
            self.winning_map = Some(winner.clone());
            Some(winner)
        } else {
            None
        }
    }
}
