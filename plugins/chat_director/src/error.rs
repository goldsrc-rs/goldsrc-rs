//! Strongly typed domain errors for the chat director plugin.

#[derive(Debug, Clone, PartialEq)]
pub enum ChatDirectorError {
    /// Message rate exceeded (antiflood).
    FloodDetected { sender: i32, wait_seconds: f32 },
    /// Unauthorized attempt to broadcast or post in staff channel.
    Unauthorized(&'static str),
    /// Bad channel target or syntax.
    InvalidChannel(String),
    /// Host feature missing in WIT.
    FeatureUnsupported {
        feature: &'static str,
        missing_wit_binding: &'static str,
        reason: &'static str,
    },
    /// Storage failure.
    Storage(String),
}

impl std::fmt::Display for ChatDirectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FloodDetected {
                sender,
                wait_seconds,
            } => {
                write!(
                    f,
                    "Antiflood triggered for player #{sender}: wait {wait_seconds:.2}s"
                )
            }
            Self::Unauthorized(cap) => {
                write!(f, "Access denied: missing required capability '{cap}'")
            }
            Self::InvalidChannel(chan) => write!(f, "Invalid channel: {chan}"),
            Self::FeatureUnsupported {
                feature,
                missing_wit_binding,
                reason,
            } => {
                write!(
                    f,
                    "[HOST GAP] Chat feature '{feature}' cannot execute: missing WIT binding '{missing_wit_binding}'. Reason: {reason}"
                )
            }
            Self::Storage(err) => write!(f, "Storage failure: {err}"),
        }
    }
}

impl std::error::Error for ChatDirectorError {}
