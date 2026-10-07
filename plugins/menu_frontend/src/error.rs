//! Strongly typed domain errors for the menu frontend plugin.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuFrontendError {
    /// Target player is not connected or valid.
    PlayerNotConnected(i32),
    /// Menu item is not found in registry.
    ItemNotFound(u32),
    /// Player session expired or invalidated.
    SessionExpired(i32),
    /// Missing required security capability to see or activate item.
    Unauthorized(&'static str),
    /// Host feature is not exposed in WIT.
    FeatureUnsupported {
        feature: &'static str,
        missing_wit_binding: &'static str,
        reason: &'static str,
    },
    /// Storage failure.
    Storage(String),
}

impl std::fmt::Display for MenuFrontendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlayerNotConnected(idx) => write!(f, "Player #{idx} is not connected"),
            Self::ItemNotFound(id) => write!(f, "Menu item #{id} not found in registry"),
            Self::SessionExpired(idx) => write!(f, "Menu session for player #{idx} has expired"),
            Self::Unauthorized(cap) => {
                write!(f, "Access denied: missing required capability '{cap}'")
            }
            Self::FeatureUnsupported {
                feature,
                missing_wit_binding,
                reason,
            } => {
                write!(
                    f,
                    "[HOST GAP] Menu feature '{feature}' cannot execute: missing WIT binding '{missing_wit_binding}'. Reason: {reason}"
                )
            }
            Self::Storage(err) => write!(f, "Storage failure: {err}"),
        }
    }
}

impl MenuFrontendError {
    /// Formats the domain error using the localization dictionary for the specified language.
    pub fn format_localized(&self, lang: &str) -> String {
        match self {
            Self::PlayerNotConnected(idx) => {
                goldsrc::tr!(
                    "menu_frontend",
                    lang,
                    "errors.player_not_connected",
                    idx = idx
                )
            }
            Self::ItemNotFound(id) => {
                goldsrc::tr!("menu_frontend", lang, "errors.item_not_found", id = id)
            }
            Self::SessionExpired(idx) => {
                goldsrc::tr!("menu_frontend", lang, "errors.session_expired", idx = idx)
            }
            Self::Unauthorized(cap) => {
                goldsrc::tr!("menu_frontend", lang, "errors.unauthorized", cap = cap)
            }
            Self::FeatureUnsupported {
                feature,
                missing_wit_binding,
                reason,
            } => {
                format!(
                    "[HOST GAP] Menu feature '{feature}' cannot execute: missing WIT binding '{missing_wit_binding}'. Reason: {reason}"
                )
            }
            Self::Storage(err) => {
                goldsrc::tr!("menu_frontend", lang, "errors.storage", err = err)
            }
        }
    }
}

impl std::error::Error for MenuFrontendError {}
