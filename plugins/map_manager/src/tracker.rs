//! Map lifecycle and timelimit tracker.

use goldsrc::prelude::*;

/// Tracks elapsed server map time and evaluates timelimit conditions.
#[derive(Debug)]
pub struct MapTracker {
    pub map_start_time: f32,
    pub current_map: String,
    pub next_map: Option<String>,
}

impl Default for MapTracker {
    fn default() -> Self {
        Self {
            map_start_time: 0.0,
            current_map: "crossfire".to_string(), // generic Half-Life map
            next_map: None,
        }
    }
}

impl MapTracker {
    pub fn new(now: f32) -> Self {
        Self {
            map_start_time: now,
            current_map: "crossfire".to_string(),
            next_map: None,
        }
    }

    /// Evaluates seconds remaining based on engine `mp_timelimit` cvar.
    pub fn time_left(&self, now: f32) -> Option<f32> {
        self.time_left_with_limit(now, cvar::cvar_get_float("mp_timelimit"))
    }

    /// Evaluates seconds remaining with explicit timelimit in minutes.
    pub fn time_left_with_limit(&self, now: f32, timelimit_mins: f32) -> Option<f32> {
        if timelimit_mins <= 0.0 {
            return None; // No timelimit set
        }
        let total_secs = timelimit_mins * 60.0;
        let elapsed = (now - self.map_start_time).max(0.0);
        let remaining = (total_secs - elapsed).max(0.0);
        Some(remaining)
    }

    /// Formats remaining time into mm:ss human string.
    pub fn format_time_left(&self, now: f32) -> String {
        self.format_time_left_with_limit(now, cvar::cvar_get_float("mp_timelimit"))
    }

    /// Formats remaining time with explicit timelimit in minutes.
    pub fn format_time_left_with_limit(&self, now: f32, timelimit_mins: f32) -> String {
        if let Some(left) = self.time_left_with_limit(now, timelimit_mins) {
            let mins = (left / 60.0) as u32;
            let secs = (left % 60.0) as u32;
            format!("{mins:02}:{secs:02}")
        } else {
            "Без ограничений".to_string()
        }
    }
}
