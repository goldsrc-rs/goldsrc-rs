//! Backend transport modular engine extensions.
//!
//! Exposes Metamod and Standalone proxy capabilities as discoverable `EngineExtension` instances
//! queryable by plugins via `ext:metamod` and `ext:standalone`.

use goldsrc_api::consts::BackendType;
use goldsrc_spi::EngineExtension;

/// Metamod plugin manager engine extension (`ext:metamod`).
pub struct MetamodExtension {
    active: bool,
}

impl MetamodExtension {
    /// Creates a new Metamod extension descriptor.
    pub const fn new(active: bool) -> Self {
        Self { active }
    }
}

impl EngineExtension for MetamodExtension {
    fn name(&self) -> &'static str {
        "metamod"
    }

    fn version(&self) -> &str {
        "1.21.0"
    }

    fn is_available(&self) -> bool {
        self.active
    }

    fn description(&self) -> &str {
        "Metamod Plugin Manager Transport Bridge"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Standalone GameDLL proxy engine extension (`ext:standalone`).
pub struct StandaloneExtension {
    active: bool,
}

impl StandaloneExtension {
    /// Creates a new Standalone proxy extension descriptor.
    pub const fn new(active: bool) -> Self {
        Self { active }
    }
}

impl EngineExtension for StandaloneExtension {
    fn name(&self) -> &'static str {
        "standalone"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn is_available(&self) -> bool {
        self.active
    }

    fn description(&self) -> &str {
        "Direct Proxy GameDLL Transport Bridge"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Helper to construct the active backend extension.
pub fn create_backend_extension(backend_type: BackendType) -> Box<dyn EngineExtension> {
    match backend_type {
        BackendType::Metamod => Box::new(MetamodExtension::new(true)),
        BackendType::Standalone => Box::new(StandaloneExtension::new(true)),
    }
}
