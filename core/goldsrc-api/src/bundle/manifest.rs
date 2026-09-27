//! Declarative bundle manifest (`bundle.toml`) model and validation.

use crate::bundle::role::ComponentRole;
use core::fmt;
use std::collections::BTreeMap;

/// Top-level metadata for a GoldSrc.rs plugin bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BundleInfo {
    /// Unique canonical bundle name (e.g. `vip_system`, `admin_system`).
    pub name: String,
    /// Semantic version string (e.g. `1.0.0`).
    #[cfg_attr(feature = "serde", serde(default = "default_version"))]
    pub version: String,
    /// Author or organization name.
    #[cfg_attr(feature = "serde", serde(default))]
    pub author: String,
    /// Brief explanation of bundle functionality.
    #[cfg_attr(feature = "serde", serde(default))]
    pub description: String,
}

#[allow(dead_code)]
fn default_version() -> String {
    "1.0.0".to_string()
}

/// Specification for an individual WASM component declared in `bundle.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BundleComponentSpec {
    /// Role assigned to this component within the bundle.
    #[cfg_attr(feature = "serde", serde(default))]
    pub role: ComponentRole,
    /// Whether this component is optional for bundle operation.
    #[cfg_attr(feature = "serde", serde(default))]
    pub optional: bool,
    /// Optional component-specific description.
    #[cfg_attr(feature = "serde", serde(default))]
    pub description: Option<String>,
}

/// Declarative manifest loaded from `bundle.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BundleManifest {
    /// Bundle identity and metadata.
    pub bundle: BundleInfo,
    /// Components belonging to this bundle mapped by plugin name.
    #[cfg_attr(feature = "serde", serde(default))]
    pub components: BTreeMap<String, BundleComponentSpec>,
}

/// Validation errors encountered in a bundle manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleValidationError {
    /// The bundle name contains illegal characters or path traversal elements.
    InvalidBundleName(String),
    /// More than one coordinator component was declared in the bundle.
    MultipleCoordinators(Vec<String>),
    /// A component name is invalid or empty.
    InvalidComponentName(String),
}

impl fmt::Display for BundleValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBundleName(name) => {
                write!(
                    f,
                    "invalid bundle name '{name}'; must be alphanumeric with underscores/hyphens and without path traversal ('..')"
                )
            }
            Self::MultipleCoordinators(names) => {
                write!(
                    f,
                    "bundle declared {} coordinators ({:?}); maximum 1 allowed per bundle",
                    names.len(),
                    names
                )
            }
            Self::InvalidComponentName(name) => {
                write!(f, "invalid or empty component name '{name}'")
            }
        }
    }
}

impl std::error::Error for BundleValidationError {}

impl BundleManifest {
    /// Validates the manifest against architectural invariants.
    pub fn validate(&self) -> Result<(), BundleValidationError> {
        let name = &self.bundle.name;
        if name.is_empty()
            || name.contains("..")
            || name.starts_with('/')
            || name.starts_with('\\')
            || name.contains(':')
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(BundleValidationError::InvalidBundleName(name.clone()));
        }

        let mut coordinators = Vec::new();
        for (comp_name, spec) in &self.components {
            if comp_name.is_empty()
                || comp_name.contains("..")
                || comp_name.starts_with('/')
                || comp_name.starts_with('\\')
            {
                return Err(BundleValidationError::InvalidComponentName(
                    comp_name.clone(),
                ));
            }
            if spec.role.is_coordinator() {
                coordinators.push(comp_name.clone());
            }
        }

        if coordinators.len() > 1 {
            return Err(BundleValidationError::MultipleCoordinators(coordinators));
        }

        Ok(())
    }

    /// Finds the singular coordinator component if declared.
    pub fn coordinator(&self) -> Option<(&str, &BundleComponentSpec)> {
        self.components
            .iter()
            .find(|(_, spec)| spec.role.is_coordinator())
            .map(|(k, v)| (k.as_str(), v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_bundle_manifest() {
        let toml_str = r#"
[bundle]
name = "vip_system"
version = "1.2.0"
author = "GoldSrc Team"
description = "VIP player rewards"

[components.vip_core]
role = "coordinator"

[components.vip_menu]
role = "ui"

[components.vip_db]
role = "service"
"#;
        let manifest: BundleManifest = toml::from_str(toml_str).unwrap();
        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.bundle.name, "vip_system");
        assert_eq!(manifest.coordinator().map(|(n, _)| n), Some("vip_core"));
    }

    #[test]
    fn test_rejects_multiple_coordinators() {
        let toml_str = r#"
[bundle]
name = "bad_bundle"

[components.coord1]
role = "coordinator"

[components.coord2]
role = "coordinator"
"#;
        let manifest: BundleManifest = toml::from_str(toml_str).unwrap();
        let err = manifest.validate().unwrap_err();
        assert!(matches!(
            err,
            BundleValidationError::MultipleCoordinators(_)
        ));
    }

    #[test]
    fn test_rejects_path_traversal_bundle_name() {
        let manifest = BundleManifest {
            bundle: BundleInfo {
                name: "../escape".into(),
                version: "1.0.0".into(),
                author: "".into(),
                description: "".into(),
            },
            components: BTreeMap::new(),
        };
        let err = manifest.validate().unwrap_err();
        assert!(matches!(err, BundleValidationError::InvalidBundleName(_)));
    }
}
