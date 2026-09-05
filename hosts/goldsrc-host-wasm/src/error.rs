//! Error taxonomy for the WASM plugin host.

use std::path::PathBuf;

/// Errors raised while loading, embedding or instantiating a WASM plugin.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to embed component metadata: {0}")]
    Embed(String),
    #[error("failed to encode WASM component: {0}")]
    Encode(String),
    #[error("failed to compile component: {0}")]
    Compile(String),
    #[error("failed to link component: {0}")]
    Link(String),
    #[error("failed to instantiate component: {0}")]
    Instantiate(String),
    #[error("plugin metadata is not valid TOML: {0}")]
    Metadata(String),
    #[error("unmet dependency requirement: {0}")]
    DependencyMismatch(String),
    #[error("plugin on_load panicked: {0}")]
    LoadPanic(String),
    #[error("plugin '{0}' is already loaded")]
    AlreadyLoaded(String),
}

/// Errors raised by plugin management commands (load/unload/reload/pause).
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("plugin '{0}' not found")]
    NotFound(String),
    #[error("plugin index {index} out of bounds (total loaded plugins: {total})")]
    IndexOutOfBounds { index: usize, total: usize },
    #[error("failed to load plugin '{name}': {source}")]
    Load {
        name: String,
        #[source]
        source: LoadError,
    },
}

/// Errors raised by the host runtime during init.
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error("failed to create PluginManager: {0}")]
    Manager(String),
}
