//! Capability authorization modification and query actions (CheckCapability, GrantCapability, RevokeCapability).

use crate::action::Action;
use crate::client::Player;

/// Dynamic authorization capability check action.
///
/// Returns `true` if the player possesses the specified capability flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckCapability<'a>(pub &'a str);

impl<'a> Action<Player> for CheckCapability<'a> {
    type Output = bool;

    #[inline(always)]
    fn execute(self, player: &Player) -> Self::Output {
        crate::auth::Auth::has_capability(player.index, self.0)
    }
}

/// Grants a dynamic authorization capability to the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantCapability {
    /// Capability name/tag (e.g. `"vip.double_jump"`).
    pub capability: String,
}

impl GrantCapability {
    /// Creates a new capability granting action.
    pub fn new(capability: impl Into<String>) -> Self {
        Self {
            capability: capability.into(),
        }
    }
}

impl Action<Player> for GrantCapability {
    type Output = bool;

    #[inline(always)]
    fn execute(self, player: &Player) -> Self::Output {
        crate::auth::Auth::grant_capability(player.index, &self.capability)
    }
}

/// Revokes a dynamic authorization capability from the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevokeCapability {
    /// Capability name/tag (e.g. `"vip.double_jump"`).
    pub capability: String,
}

impl RevokeCapability {
    /// Creates a new capability revocation action.
    pub fn new(capability: impl Into<String>) -> Self {
        Self {
            capability: capability.into(),
        }
    }
}

impl Action<Player> for RevokeCapability {
    type Output = bool;

    #[inline(always)]
    fn execute(self, player: &Player) -> Self::Output {
        crate::auth::Auth::revoke_capability(player.index, &self.capability)
    }
}
