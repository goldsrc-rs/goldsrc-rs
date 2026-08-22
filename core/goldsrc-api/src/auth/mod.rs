//! Capability-based access control, registry, and hierarchical DSL.

pub mod dsl;
pub mod registry;

pub use dsl::CapExpr;
pub use registry::{CAPS, CapabilityRegistry};

#[cfg(target_arch = "wasm32")]
use crate::bindings::goldsrc::engine::api;

/// Auth System: Capability-based access control for plugins.
pub struct Auth;

impl Auth {
    /// Registers a new capability in the global system.
    pub fn register_capability(name: &str, description: &str) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            api::host_register_capability(name, description)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut caps = CAPS.write().unwrap_or_else(|e| e.into_inner());
            if let std::collections::hash_map::Entry::Vacant(e) =
                caps.registered.entry(name.to_string())
            {
                e.insert(description.to_string());
                true
            } else {
                false
            }
        }
    }

    /// Checks if a player has a specific capability (with wildcard resolution).
    pub fn has_capability(player_index: i32, name: &str) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            api::host_has_capability(player_index, name)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let caps = CAPS.read().unwrap_or_else(|e| e.into_inner());
            caps.player_capabilities
                .get(&player_index)
                .is_some_and(|player_caps| {
                    if player_caps.contains(name) || player_caps.contains("*") {
                        return true;
                    }
                    for g in player_caps {
                        if let Some(prefix) = g.strip_suffix(".*")
                            && name.starts_with(prefix)
                            && name[prefix.len()..].starts_with('.')
                        {
                            return true;
                        }
                    }
                    false
                })
        }
    }

    /// Removes all capability grants for a player (e.g. on client disconnect).
    pub fn remove_player(player_index: i32) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut caps = CAPS.write().unwrap_or_else(|e| e.into_inner());
            caps.player_capabilities.remove(&player_index);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = player_index;
        }
    }

    /// Clears all player capability grants (e.g. on map change / server deactivate).
    pub fn clear_all_players() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut caps = CAPS.write().unwrap_or_else(|e| e.into_inner());
            caps.player_capabilities.clear();
        }
    }

    /// Grants a capability to a player dynamically.
    pub fn grant_capability(player_index: i32, name: &str) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            api::host_grant_capability(player_index, name)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut caps = CAPS.write().unwrap_or_else(|e| e.into_inner());
            if !caps.registered.contains_key(name) && !name.ends_with(".*") && name != "*" {
                return false;
            }
            caps.player_capabilities
                .entry(player_index)
                .or_default()
                .insert(name.to_string())
        }
    }

    /// Revokes a capability from a player dynamically.
    pub fn revoke_capability(player_index: i32, name: &str) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            api::host_revoke_capability(player_index, name)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut caps = CAPS.write().unwrap_or_else(|e| e.into_inner());
            if let Some(player_caps) = caps.player_capabilities.get_mut(&player_index) {
                player_caps.remove(name)
            } else {
                false
            }
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::Auth;

    #[test]
    fn capability_lifecycle() {
        Auth::register_capability("admin", "test capability");
        assert!(!Auth::has_capability(1, "admin"));
        assert!(Auth::grant_capability(1, "admin"));
        assert!(Auth::has_capability(1, "admin"));
        assert!(!Auth::grant_capability(1, "missing"));
        assert!(Auth::revoke_capability(1, "admin"));
        assert!(!Auth::has_capability(1, "admin"));
    }

    #[test]
    fn test_wildcard_and_eviction_lifecycle() {
        Auth::register_capability("vip.heal", "heal ability");
        Auth::grant_capability(2, "vip.*");

        // Wildcard match
        assert!(Auth::has_capability(2, "vip.heal"));
        assert!(!Auth::has_capability(2, "admin.slay"));

        // Eviction on disconnect
        Auth::remove_player(2);
        assert!(!Auth::has_capability(2, "vip.heal"));

        // Clear all on map change
        Auth::grant_capability(3, "vip.heal");
        assert!(Auth::has_capability(3, "vip.heal"));
        Auth::clear_all_players();
        assert!(!Auth::has_capability(3, "vip.heal"));
    }
}
