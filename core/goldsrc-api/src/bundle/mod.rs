//! Autonomous Bundle Component Model and Metadata Abstractions.

pub mod manifest;
pub mod role;

pub use manifest::{BundleComponentSpec, BundleInfo, BundleManifest, BundleValidationError};
pub use role::{ComponentRole, ParseComponentRoleError};
