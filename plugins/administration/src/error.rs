//! Strongly typed domain errors for the administration plugin.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdministrationError {
    /// Target staff member or player was not found.
    TargetNotFound(String),
    /// Caller does not possess the required administration capability.
    PermissionDenied(&'static str),
    /// Invalid command argument.
    InvalidArgument(String),
    /// Feature requires an engine host call not present in WIT.
    FeatureUnsupported {
        feature: &'static str,
        missing_wit_binding: &'static str,
        reason: &'static str,
    },
    /// Storage failure.
    Storage(String),
}

impl std::fmt::Display for AdministrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TargetNotFound(id) => write!(f, "Staff member or target '{id}' not found"),
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
                    "[HOST GAP] Administration feature '{feature}' cannot execute: missing WIT binding '{missing_wit_binding}'. Reason: {reason}"
                )
            }
            Self::Storage(err) => write!(f, "Storage failure: {err}"),
        }
    }
}

impl std::error::Error for AdministrationError {}
