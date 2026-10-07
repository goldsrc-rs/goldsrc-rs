//! Antiflood rate limiter service.

use std::collections::HashMap;

/// Rate limiter tracking message frequency per player.
#[derive(Default)]
pub struct AntifloodService {
    last_message_time: HashMap<i32, f32>,
}

impl AntifloodService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if the message is allowed under the rate limit.
    /// Returns `Ok(())` if allowed, or `Err(remaining_wait_seconds)` if flood detected.
    pub fn check_and_update(
        &mut self,
        sender: i32,
        now: f32,
        min_interval_secs: f32,
    ) -> Result<(), f32> {
        if let Some(&last) = self.last_message_time.get(&sender) {
            let elapsed = now - last;
            if elapsed < min_interval_secs {
                let wait = min_interval_secs - elapsed;
                return Err(wait);
            }
        }
        self.last_message_time.insert(sender, now);
        Ok(())
    }

    /// Cleans up state when a player disconnects.
    pub fn on_player_disconnect(&mut self, sender: i32) {
        self.last_message_time.remove(&sender);
    }
}
