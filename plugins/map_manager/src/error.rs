//! Strongly typed domain errors for the map manager plugin.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapManagerError {
    /// Requested map is not in the server cycle.
    UnknownMap(String),
    /// Map was recently played and is blocked by anti-repeat policy.
    MapRecentlyPlayed { map: String, remaining_rounds: u32 },
    /// Vote is already in progress or already completed.
    VoteAlreadyActive,
    /// Player already voted in this session.
    AlreadyVoted,
    /// Host feature missing in WIT.
    FeatureUnsupported {
        feature: &'static str,
        missing_wit_binding: &'static str,
        reason: &'static str,
    },
    /// Storage failure.
    Storage(String),
}

impl std::fmt::Display for MapManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownMap(map) => write!(f, "Map '{map}' is not in the rotation pool"),
            Self::MapRecentlyPlayed {
                map,
                remaining_rounds,
            } => {
                write!(
                    f,
                    "Map '{map}' was recently played: blocked for {remaining_rounds} more maps"
                )
            }
            Self::VoteAlreadyActive => write!(f, "A map vote is already active or concluded"),
            Self::AlreadyVoted => write!(f, "You have already cast your vote"),
            Self::FeatureUnsupported {
                feature,
                missing_wit_binding,
                reason,
            } => {
                write!(
                    f,
                    "[HOST GAP] Map manager feature '{feature}' cannot execute: missing WIT binding '{missing_wit_binding}'. Reason: {reason}"
                )
            }
            Self::Storage(err) => write!(f, "Storage failure: {err}"),
        }
    }
}

impl std::error::Error for MapManagerError {}
