//! Public framework (SDK) for GoldSrc.rs plugin developers.
//!
//! This is the main entry point for plugin developers. It re-exports
//! everything you need from the other crates.

pub use goldsrc_api::{Engine, Entity, Player, Plugin};
pub use goldsrc_macros::{command, plugin};
pub use goldsrc_sys;

/// Initialize the GoldSrc.rs framework.
pub fn init() {
    // TODO: Initialize logging, signal handlers, etc.
}
