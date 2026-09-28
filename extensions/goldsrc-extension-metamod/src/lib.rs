//! Modular Metamod plugin manager engine extension.
//!
//! Provides the `MetamodExtension` descriptor and runtime capabilities
//! for plugins to discover and interact with the Metamod plugin manager.

use goldsrc_spi::EngineExtension;
use std::sync::Arc;

/// Metamod plugin manager engine extension (`ext:metamod`).
pub struct MetamodExtension {
    active: bool,
    version: &'static str,
}

impl MetamodExtension {
    /// Creates a new Metamod extension descriptor.
    pub const fn new(active: bool) -> Self {
        Self {
            active,
            version: "1.21.0",
        }
    }

    /// Sets the discovered Metamod version.
    pub fn with_version(active: bool, version: &'static str) -> Self {
        Self { active, version }
    }
}

impl EngineExtension for MetamodExtension {
    fn name(&self) -> &'static str {
        "metamod"
    }

    fn version(&self) -> &str {
        self.version
    }

    fn is_available(&self) -> bool {
        self.active
    }

    fn description(&self) -> &str {
        "Metamod Plugin Manager Engine Extension"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Initializes and registers the Metamod engine extension into the global registry.
pub fn init(active: bool) {
    let registry = goldsrc_core::extension::extension_registry();
    registry.register(Arc::new(MetamodExtension::new(active)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metamod_extension() {
        let ext = MetamodExtension::new(true);
        assert_eq!(ext.name(), "metamod");
        assert!(ext.is_available());
        assert_eq!(ext.version(), "1.21.0");
    }
}
