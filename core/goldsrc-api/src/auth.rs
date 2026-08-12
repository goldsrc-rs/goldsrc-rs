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
            let _ = (name, description);
            false
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
            let _ = (player_index, name);
            false
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
            let _ = (player_index, name);
            false
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
            let _ = (player_index, name);
            false
        }
    }
}
