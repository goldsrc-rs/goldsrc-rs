//! VIP player perks session tracking and equipment delivery logic.

use std::collections::HashMap;

/// Per-player VIP session state.
#[derive(Debug, Clone)]
pub struct VipPlayerState {
    pub claimed_this_round: bool,
    pub regen_enabled: bool,
    pub prefix: String,
}

impl Default for VipPlayerState {
    fn default() -> Self {
        Self {
            claimed_this_round: false,
            regen_enabled: true,
            prefix: "[VIP] ".to_string(),
        }
    }
}

/// Service managing round cycles and equipment perks.
#[derive(Default)]
pub struct PerkService {
    current_round: u32,
    player_states: HashMap<i32, VipPlayerState>,
}

impl PerkService {
    pub fn new() -> Self {
        Self {
            current_round: 1,
            player_states: HashMap::new(),
        }
    }

    pub fn current_round(&self) -> u32 {
        self.current_round
    }

    pub fn advance_round(&mut self) {
        self.current_round += 1;
        for state in self.player_states.values_mut() {
            state.claimed_this_round = false;
        }
        goldsrc::log_info!(
            "[Privileges] Round {} started, reset equipment claim locks",
            self.current_round
        );
    }

    pub fn get_or_create(&mut self, player_index: i32) -> &mut VipPlayerState {
        self.player_states.entry(player_index).or_default()
    }

    pub fn is_regen_enabled(&self, player_index: i32) -> bool {
        self.player_states
            .get(&player_index)
            .map(|s| s.regen_enabled)
            .unwrap_or(true)
    }

    pub fn toggle_regen(&mut self, player_index: i32) -> bool {
        let state = self.get_or_create(player_index);
        state.regen_enabled = !state.regen_enabled;
        state.regen_enabled
    }

    pub fn has_claimed(&self, player_index: i32) -> bool {
        self.player_states
            .get(&player_index)
            .map(|s| s.claimed_this_round)
            .unwrap_or(false)
    }

    pub fn mark_claimed(&mut self, player_index: i32) {
        self.get_or_create(player_index).claimed_this_round = true;
    }
}
