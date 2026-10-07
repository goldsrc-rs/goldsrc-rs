//! Strongly typed domain errors for the moderation plugin.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModerationError {
    /// Target player was not found by index, userid, or display name.
    PlayerNotFound(String),
    /// Target player is connected but in an invalid or dead state for this action.
    InvalidPlayerState(i32, &'static str),
    /// Caller does not possess the required security capability.
    PermissionDenied(&'static str),
    /// Bad command arguments supplied by the caller.
    InvalidArgument(String),
    /// The requested operation requires an engine or WIT host feature that is not yet exposed.
    /// This is used instead of brittle ad-hoc workarounds.
    FeatureUnsupported {
        feature: &'static str,
        missing_wit_binding: &'static str,
        reason: &'static str,
    },
    /// An error occurred in the storage subsystem.
    Storage(String),
}

impl std::fmt::Display for ModerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlayerNotFound(id) => write!(f, "Player '{id}' not found on server"),
            Self::InvalidPlayerState(idx, state) => {
                write!(f, "Player #{idx} is in invalid state for action: {state}")
            }
            Self::PermissionDenied(cap) => {
                write!(f, "Access denied: missing required capability '{cap}'")
            }
            Self::InvalidArgument(msg) => write!(f, "Invalid argument: {msg}"),
            Self::FeatureUnsupported {
                feature,
                missing_wit_binding,
                reason,
            } => {
                write!(
                    f,
                    "[HOST GAP] Feature '{feature}' cannot execute: missing WIT binding '{missing_wit_binding}'. Reason: {reason}"
                )
            }
            Self::Storage(err) => write!(f, "Storage failure: {err}"),
        }
    }
}

impl std::error::Error for ModerationError {}
