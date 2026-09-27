//! Component roles defining architectural responsibilities within a GoldSrc.rs bundle.

use core::fmt;
use core::str::FromStr;

/// Architectural role of a WASM component within its parent bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ComponentRole {
    /// Root public facade, API gateway, and coordinator. Maximum 1 per bundle.
    Coordinator,
    /// Backend service responsible for persistence, database storage, and shared state.
    Service,
    /// Gameplay feature providing hooks, rules, and game logic modifications.
    Feature,
    /// User interface component managing client menus, HUD elements, or chat commands.
    Ui,
    /// Symmetric participant with no special structural hierarchy.
    #[default]
    Peer,
}

impl ComponentRole {
    /// Returns the canonical string identifier for this component role.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Coordinator => "coordinator",
            Self::Service => "service",
            Self::Feature => "feature",
            Self::Ui => "ui",
            Self::Peer => "peer",
        }
    }

    /// Whether this component serves as the singular coordinator for its bundle.
    #[inline]
    pub const fn is_coordinator(&self) -> bool {
        matches!(self, Self::Coordinator)
    }

    /// Maximum allowed count of components with this role per bundle (`Some(1)` for Coordinator).
    #[inline]
    pub const fn max_per_bundle(&self) -> Option<usize> {
        match self {
            Self::Coordinator => Some(1),
            _ => None,
        }
    }
}

impl fmt::Display for ComponentRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error returned when parsing an invalid component role string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseComponentRoleError(pub String);

impl fmt::Display for ParseComponentRoleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid component role '{}'; expected 'coordinator', 'service', 'feature', 'ui', or 'peer'",
            self.0
        )
    }
}

impl std::error::Error for ParseComponentRoleError {}

impl FromStr for ComponentRole {
    type Err = ParseComponentRoleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "coordinator" => Ok(Self::Coordinator),
            "service" => Ok(Self::Service),
            "feature" => Ok(Self::Feature),
            "ui" => Ok(Self::Ui),
            "peer" => Ok(Self::Peer),
            other => Err(ParseComponentRoleError(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_serialization_and_display() {
        assert_eq!(ComponentRole::Coordinator.as_str(), "coordinator");
        assert_eq!(ComponentRole::Service.to_string(), "service");
        assert_eq!(ComponentRole::Feature.to_string(), "feature");
        assert_eq!(ComponentRole::Ui.to_string(), "ui");
        assert_eq!(ComponentRole::Peer.to_string(), "peer");

        assert_eq!(
            "coordinator".parse::<ComponentRole>().unwrap(),
            ComponentRole::Coordinator
        );
        assert_eq!(
            "SERVICE".parse::<ComponentRole>().unwrap(),
            ComponentRole::Service
        );
        assert_eq!("ui".parse::<ComponentRole>().unwrap(), ComponentRole::Ui);
        assert_eq!(
            "PEER".parse::<ComponentRole>().unwrap(),
            ComponentRole::Peer
        );
        assert!("invalid".parse::<ComponentRole>().is_err());
    }

    #[test]
    fn test_role_invariants() {
        assert!(ComponentRole::Coordinator.is_coordinator());
        assert!(!ComponentRole::Feature.is_coordinator());
        assert_eq!(ComponentRole::Coordinator.max_per_bundle(), Some(1));
        assert_eq!(ComponentRole::Service.max_per_bundle(), None);
    }
}
