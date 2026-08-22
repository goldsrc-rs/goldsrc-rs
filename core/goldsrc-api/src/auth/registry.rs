//! Capability registry shared between the WASM host and native auth.

use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, RwLock};

/// Thread-safe registry of capabilities and per-player grants, shared
/// between the host and WASM plugin host functions.
#[derive(Default)]
pub struct CapabilityRegistry {
    /// Registered capability name -> description.
    pub registered: HashMap<String, String>,
    /// player_index -> set of granted capability names.
    pub player_capabilities: HashMap<i32, HashSet<String>>,
}

/// Global capability registry. Both the WASM host and the native `Auth`
/// facade operate on this single instance, so grants made by a plugin are
/// visible to native code and vice versa.
pub static CAPS: LazyLock<RwLock<CapabilityRegistry>> =
    LazyLock::new(|| RwLock::new(CapabilityRegistry::default()));
