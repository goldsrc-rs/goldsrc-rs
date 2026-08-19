//! Abstract interface for plugin runtime execution hosts (WASM, Python, Lua, Native).

use std::path::Path;

/// Result type for plugin host operations.
pub type HostResult<T> = Result<T, HostError>;

/// Standard errors that can occur during plugin lifecycle and execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostError {
    /// Failed to load or compile plugin module.
    Load(String),
    /// Plugin with the specified name or ID was not found.
    NotFound(String),
    /// Error during execution of a plugin event or callback.
    Execution(String),
    /// Internal runtime host error.
    Runtime(String),
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(msg) => write!(f, "Failed to load plugin: {msg}"),
            Self::NotFound(name) => write!(f, "Plugin not found: {name}"),
            Self::Execution(msg) => write!(f, "Plugin execution error: {msg}"),
            Self::Runtime(msg) => write!(f, "Host runtime error: {msg}"),
        }
    }
}

impl std::error::Error for HostError {}

/// Abstract plugin host interface.
///
/// Any plugin runtime environment (Wasmtime, CPython, LuaJIT, Native DLLs)
/// implements this trait to plug into the GoldSrc.rs engine backend.
pub trait PluginHost: Send + Sync {
    /// Identifier for this host runtime (e.g., "wasm", "python", "lua").
    fn name(&self) -> &'static str;

    /// File extensions supported by this host (e.g., `&[".wasm"]`, `&[".py"]`).
    fn supported_extensions(&self) -> &'static [&'static str];

    /// Load a plugin from a filesystem path.
    fn load_plugin(&mut self, path: &Path) -> HostResult<String>;

    /// Unload an active plugin by name.
    fn unload_plugin(&mut self, name: &str) -> HostResult<()>;

    /// Hot-reload an active plugin.
    fn reload_plugin(&mut self, name: &str) -> HostResult<()>;

    /// Frame tick hook called by the engine loop.
    fn on_frame(&mut self);
}
