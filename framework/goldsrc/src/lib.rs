//! Public framework (SDK) for GoldSrc.rs plugin developers.
//!
//! This is the main entry point for plugin developers. It re-exports
//! everything you need from the other crates.

/// Flat ECS for plugin state storage.
pub mod ecs;

/// Shared host CLI (`meta-rs`/`mrs`). Enabled by the `host-cli` feature.
#[cfg(feature = "host-cli")]
pub mod cli;

/// Host runtime orchestrator. Enabled by the `host` feature.
#[cfg(feature = "host")]
pub mod host;

/// Shared backend plumbing (engine access, print queue, engfunc macros).
/// Enabled by the `host` feature.
#[cfg(feature = "host")]
pub mod backend;

/// `goldsrc.toml` configuration types and loader.
pub mod config;
/// Centralized hook dispatching helpers.
#[cfg(feature = "host")]
pub mod hooks;
/// Unified structured logger for backends.
pub mod logging;
/// Filesystem path resolution helpers.
pub mod paths;

pub use ::log;
pub use ::log::{
    debug as log_debug, error as log_err, info as log_info, trace as log_trace, warn as log_warn,
};
pub use config::*;
pub use ecs::*;
pub use goldsrc_api as api;
pub use goldsrc_api;
pub use goldsrc_api::{auth::Auth, Engine, Entity, Player, Plugin, Vector3};
pub use goldsrc_macros as macros;
pub use goldsrc_macros::{command, event, on_load, plugin};

/// Convenient prelude module for plugin authors.
pub mod prelude {
    pub use crate::ecs::*;
    pub use crate::Auth;
    pub use crate::{command, event, on_load, plugin};
    pub use crate::{Engine, Entity, Player, Plugin, Vector3};
    pub use ::log::{
        debug, debug as log_debug, error, error as log_err, info, info as log_info, trace,
        trace as log_trace, warn, warn as log_warn,
    };
}

/// Direct console print helper for WASM plugins.
pub mod wasm_log {
    /// Forwards `msg` to the host logger (WASM) or `println!` (native).
    pub fn print(msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::goldsrc_api::bindings::goldsrc::engine::api::host_log(msg);
        }
        #[cfg(not(target_arch = "wasm32"))]
        println!("{}", msg);
    }
}
