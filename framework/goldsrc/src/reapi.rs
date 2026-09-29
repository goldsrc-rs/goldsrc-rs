//! ReAPI engine extension helpers.

/// Returns `true` if ReAPI engine extension is loaded and active.
#[inline]
pub fn has_reapi() -> bool {
    crate::extension::is_available("reapi", None)
}

/// Retrieves the ReAPI version string if present.
#[inline]
pub fn reapi_version() -> Option<String> {
    crate::extension::version("reapi")
}
