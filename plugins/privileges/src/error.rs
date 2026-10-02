//! Strongly typed domain errors for the privileges plugin.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrivilegesError {
    /// Target player is not valid or connected.
    PlayerNotConnected(i32),
    /// Target player does not have VIP/Privilege capability.
    Unauthorized(String),
    /// Weapon/equipment already claimed in the current round.
    AlreadyClaimedThisRound,
    /// Weapon restricted in the current round.
    RoundRestricted {
        current_round: u32,
        allowed_from_round: u32,
    },
    /// Host capability not present in WIT.
    FeatureUnsupported {
        feature: &'static str,
        missing_wit_binding: &'static str,
        reason: &'static str,
    },
    /// Storage failure.
    Storage(String),
}

impl std::fmt::Display for PrivilegesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlayerNotConnected(idx) => write!(f, "Player #{idx} is not connected"),
            Self::Unauthorized(cap) => {
                write!(f, "Access denied: missing required privilege '{cap}'")
            }
            Self::AlreadyClaimedThisRound => {
                write!(f, "Equipment perk already claimed for this round")
            }
            Self::RoundRestricted {
                current_round,
                allowed_from_round,
            } => {
                write!(
                    f,
                    "Equipment restricted: round {current_round} < allowed round {allowed_from_round}"
                )
            }
            Self::FeatureUnsupported {
                feature,
                missing_wit_binding,
                reason,
            } => {
                write!(
                    f,
                    "[HOST GAP] Privilege feature '{feature}' cannot execute: missing WIT binding '{missing_wit_binding}'. Reason: {reason}"
                )
            }
            Self::Storage(err) => write!(f, "Storage failure: {err}"),
        }
    }
}

impl std::error::Error for PrivilegesError {}
