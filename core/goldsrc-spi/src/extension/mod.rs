//! Engine extension SPI traits and contracts.
//!
//! Provides a uniform abstraction for modular engine-level extensions (e.g. ReAPI,
//! Metamod, Xash3D, Steamworks) that can be dynamically registered, inspected,
//! and queried by plugins via the requirements DSL (`ext:<name>@<version>`).

use std::any::Any;

/// Core trait implemented by engine-level extension providers.
pub trait EngineExtension: Send + Sync + 'static {
    /// Canonical identifier of the extension (e.g. "reapi", "metamod", "rehlds", "regamedll").
    fn name(&self) -> &'static str;

    /// Semantic version of the extension (e.g. "5.26.0").
    fn version(&self) -> &str;

    /// Whether the extension is active and available at runtime.
    fn is_available(&self) -> bool;

    /// Human-readable description of the extension.
    fn description(&self) -> &str {
        ""
    }

    /// Downcasting helper for accessing concrete extension-specific interfaces.
    fn as_any(&self) -> &dyn Any;

    /// Lifecycle hook: called on each server physics frame tick.
    fn on_server_frame(&self) {}

    /// Lifecycle hook: called when server transitions to a new map.
    fn on_change_level(&self, _map_name: &str) {}

    /// Lifecycle hook: called when server is shutting down.
    fn on_shutdown(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockExtension {
        active: bool,
    }

    impl EngineExtension for MockExtension {
        fn name(&self) -> &'static str {
            "mock_ext"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn is_available(&self) -> bool {
            self.active
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_engine_extension_contract() {
        let ext = MockExtension { active: true };
        assert_eq!(ext.name(), "mock_ext");
        assert_eq!(ext.version(), "1.0.0");
        assert!(ext.is_available());
        assert_eq!(ext.description(), "");
        assert!(ext.as_any().is::<MockExtension>());
    }
}
