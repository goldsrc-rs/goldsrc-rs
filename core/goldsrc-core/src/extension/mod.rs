//! Modular Engine Extensions subsystem.
//!
//! Provides a discoverable extension registry, standard extensions (ReAPI, ReHLDS,
//! ReGameDLL, Metamod, Standalone), and lifecycle coordination.

pub mod backend;
pub mod registry;

pub use backend::{MetamodExtension, StandaloneExtension};
pub use registry::{ExtensionRegistry, extension_registry};

use goldsrc_api::consts::BackendType;
use std::sync::Arc;

/// Initializes default engine extensions and registers the active backend extension.
pub fn init_default_extensions(backend_type: BackendType) {
    let registry = extension_registry();

    // Register active backend extension
    match backend_type {
        BackendType::Metamod => {
            registry.register(Arc::new(MetamodExtension::new(true)));
            registry.register(Arc::new(StandaloneExtension::new(false)));
        }
        BackendType::Standalone => {
            registry.register(Arc::new(StandaloneExtension::new(true)));
            registry.register(Arc::new(MetamodExtension::new(false)));
        }
    }
}
