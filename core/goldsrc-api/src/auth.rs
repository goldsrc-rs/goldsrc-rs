#[cfg(not(target_arch = "wasm32"))]
use crate::caps::CAPS;

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

    /// Checks if a player has a specific capability.
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
                .is_some_and(|player_caps| player_caps.contains(name))
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
            if !caps.registered.contains_key(name) {
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
}
