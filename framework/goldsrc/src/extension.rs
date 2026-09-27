//! Engine extension discovery and querying subsystem.

/// Checks whether an engine extension with `name` is registered and available.
/// Optionally matches a SemVer constraint (e.g. `Some(">=1.2.0")`).
#[inline]
pub fn is_available(name: &str, version_req: Option<&str>) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_is_extension_available(name, version_req)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (name, version_req);
        false
    }
}

/// Retrieves the semantic version string of a named extension if registered and active.
#[inline]
pub fn version(name: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_get_extension_version(name)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = name;
        None
    }
}
