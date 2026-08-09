//! WASM plugin host for GoldSrc.rs.
//!
//! Uses `wasm3` as the WASM runtime for maximum compatibility with 32-bit HLDS.
//! The runtime is abstracted behind the `PluginRuntime` trait, allowing future
//! migration to `wasmer` or `wasmtime`.

use goldsrc_api::Plugin;
use std::path::Path;

/// Plugin runtime trait — abstracts over the WASM engine.
pub trait PluginRuntime: Send + Sync {
    /// Load a WASM plugin from a file.
    fn load(&mut self, path: &Path) -> Result<(), RuntimeError>;

    /// Unload the currently loaded plugin.
    fn unload(&mut self);

    /// Call a function in the WASM plugin.
    fn call(&mut self, name: &str) -> Result<(), RuntimeError>;
}

/// Errors that can occur in the WASM runtime.
#[derive(Debug)]
pub enum RuntimeError {
    /// Failed to load the WASM module.
    LoadError(String),
    /// Function not found in the module.
    FunctionNotFound(String),
    /// Execution error.
    ExecutionError(String),
    /// IO error.
    IoError(std::io::Error),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::LoadError(msg) => write!(f, "WASM load error: {}", msg),
            RuntimeError::FunctionNotFound(name) => write!(f, "Function not found: {}", name),
            RuntimeError::ExecutionError(msg) => write!(f, "WASM execution error: {}", msg),
            RuntimeError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<std::io::Error> for RuntimeError {
    fn from(e: std::io::Error) -> Self {
        RuntimeError::IoError(e)
    }
}

/// Plugin manager — handles loading, unloading, and hot-reloading of WASM plugins.
pub struct PluginManager {
    runtime: Box<dyn PluginRuntime>,
}

impl PluginManager {
    /// Create a new plugin manager with the given runtime.
    pub fn new(runtime: Box<dyn PluginRuntime>) -> Self {
        Self { runtime }
    }

    /// Load a WASM plugin from a file.
    pub fn load(&mut self, path: &Path) -> Result<(), RuntimeError> {
        self.runtime.load(path)
    }

    /// Unload the currently loaded plugin.
    pub fn unload(&mut self) {
        self.runtime.unload()
    }

    /// Call a function in the loaded plugin.
    pub fn call(&mut self, name: &str) -> Result<(), RuntimeError> {
        self.runtime.call(name)
    }
}
