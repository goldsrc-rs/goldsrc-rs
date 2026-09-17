//! High-level ReAPI capability flags, version info, and query traits.
//!
//! Provides type-safe abstractions for detecting and leveraging advanced ReHLDS
//! and ReGameDLL engine extensions with zero-overhead and transparent fallbacks.

/// Status of ReAPI components detected at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReApiStatus {
    /// Whether ReHLDS engine is active.
    pub rehlds_active: bool,
    /// ReHLDS major version (if active).
    pub rehlds_major: i32,
    /// ReHLDS minor version (if active).
    pub rehlds_minor: i32,
    /// Whether ReGameDLL is active.
    pub regamedll_active: bool,
    /// ReGameDLL major version (if active).
    pub regamedll_major: i32,
    /// ReGameDLL minor version (if active).
    pub regamedll_minor: i32,
}

impl ReApiStatus {
    /// Returns true if either ReHLDS or ReGameDLL is available.
    pub const fn is_available(&self) -> bool {
        self.rehlds_active || self.regamedll_active
    }
}

/// Abstract high-level queries for ReHLDS engine capabilities.
pub trait RehldsCapabilities: Send + Sync {
    /// Checks if ReHLDS is currently active.
    fn is_rehlds(&self) -> bool;

    /// Gets ReHLDS build number if running on ReHLDS.
    fn get_build_number(&self) -> Option<i32>;

    /// Gets real engine uptime (double precision).
    fn get_real_time(&self) -> Option<f64>;
}

/// Abstract high-level queries for ReGameDLL game rules & game features.
pub trait ReGameCapabilities: Send + Sync {
    /// Checks if ReGameDLL is currently active.
    fn is_regamedll(&self) -> bool;

    /// Requests team auto-balancing.
    fn balance_teams(&self) -> bool {
        false
    }

    /// Triggers immediate round restart with given delay.
    fn restart_round(&self, _delay: f32) -> bool {
        false
    }
}
