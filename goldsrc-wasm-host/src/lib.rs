//! WASM plugin host for GoldSrc.rs.
//!
//! Uses `wasmi` as the pure-Rust WASM runtime for maximum compatibility and safety
//! with 32-bit HLDS without C/C++ build system dependencies.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use wasmi::{Engine, Linker, Module, Store, TypedFunc};

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
    /// Notify watcher error.
    NotifyError(notify::Error),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::LoadError(msg) => write!(f, "WASM load error: {}", msg),
            RuntimeError::FunctionNotFound(name) => write!(f, "Function not found: {}", name),
            RuntimeError::ExecutionError(msg) => write!(f, "WASM execution error: {}", msg),
            RuntimeError::IoError(e) => write!(f, "IO error: {}", e),
            RuntimeError::NotifyError(e) => write!(f, "Notify error: {}", e),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<std::io::Error> for RuntimeError {
    fn from(e: std::io::Error) -> Self {
        RuntimeError::IoError(e)
    }
}

impl From<notify::Error> for RuntimeError {
    fn from(e: notify::Error) -> Self {
        RuntimeError::NotifyError(e)
    }
}

/// Host environment context provided to WASM modules.
#[derive(Default)]
pub struct HostState;

/// Single loaded WASM plugin module instance.
pub struct LoadedPlugin {
    pub name: String,
    pub path: PathBuf,
    store: Store<HostState>,
    on_load_fn: Option<TypedFunc<(), ()>>,
    on_frame_fn: Option<TypedFunc<(), ()>>,
}

impl LoadedPlugin {
    pub fn call_on_load(&mut self) -> Result<(), RuntimeError> {
        if let Some(f) = &self.on_load_fn {
            f.call(&mut self.store, ()).map_err(|e| RuntimeError::ExecutionError(e.to_string()))?;
        }
        Ok(())
    }

    pub fn call_on_frame(&mut self) -> Result<(), RuntimeError> {
        if let Some(f) = &self.on_frame_fn {
            f.call(&mut self.store, ()).map_err(|e| RuntimeError::ExecutionError(e.to_string()))?;
        }
        Ok(())
    }
}

/// WASM Plugin Manager — handles runtime execution, host imports, and hot-reload.
pub struct PluginManager {
    engine: Engine,
    plugins: Vec<LoadedPlugin>,
    watcher_rx: Option<Receiver<notify::Result<notify::Event>>>,
    _watcher: Option<RecommendedWatcher>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        let engine = Engine::default();
        Self {
            engine,
            plugins: Vec::new(),
            watcher_rx: None,
            _watcher: None,
        }
    }

    /// Setup hot-reload file watching on a directory.
    pub fn enable_hot_reload<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), RuntimeError> {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )?;

        if dir.as_ref().exists() {
            watcher.watch(dir.as_ref(), RecursiveMode::NonRecursive)?;
        }

        self.watcher_rx = Some(rx);
        self._watcher = Some(watcher);
        Ok(())
    }

    /// Load a WASM plugin module from a file.
    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<(), RuntimeError> {
        let path_buf = path.as_ref().to_path_buf();
        let bytes = fs::read(&path_buf)?;

        let module = Module::new(&self.engine, &bytes[..])
            .map_err(|e| RuntimeError::LoadError(e.to_string()))?;

        let mut store = Store::new(&self.engine, HostState);
        let mut linker = Linker::<HostState>::new(&self.engine);

        // Register GoldSrc Host Functions for WASM modules
        linker
            .define(
                "env",
                "server_print",
                wasmi::Func::wrap(&mut store, |_caller: wasmi::Caller<'_, HostState>, msg_ptr: i32, msg_len: i32| {
                    log::info!("[WASM Host Function] server_print called (ptr={}, len={})", msg_ptr, msg_len);
                }),
            )
            .map_err(|e| RuntimeError::LoadError(e.to_string()))?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| RuntimeError::LoadError(e.to_string()))?
            .start(&mut store)
            .map_err(|e| RuntimeError::ExecutionError(e.to_string()))?;

        let on_load_fn = instance.get_typed_func::<(), ()>(&store, "on_load").ok();
        let on_frame_fn = instance.get_typed_func::<(), ()>(&store, "on_frame").ok();

        let plugin_name = path_buf
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        let mut loaded = LoadedPlugin {
            name: plugin_name,
            path: path_buf,
            store,
            on_load_fn,
            on_frame_fn,
        };

        loaded.call_on_load()?;
        self.plugins.push(loaded);
        Ok(())
    }

    /// Poll for file changes and reload updated plugins.
    pub fn process_hot_reload(&mut self) {
        let mut reload_paths = Vec::new();
        if let Some(rx) = &self.watcher_rx {
            while let Ok(Ok(event)) = rx.try_recv() {
                for path in event.paths {
                    if path.extension().map_or(false, |ext| ext == "wasm") {
                        reload_paths.push(path);
                    }
                }
            }
        }

        for path in reload_paths {
            log::info!("[GoldSrc.rs WASM Host] Hot-reload triggered for {:?}", path);
            self.plugins.retain(|p| p.path != path);
            if let Err(err) = self.load_plugin(&path) {
                log::error!("[GoldSrc.rs WASM Host] Failed to reload {:?}: {}", path, err);
            } else {
                log::info!("[GoldSrc.rs WASM Host] Reloaded successfully {:?}", path);
            }
        }
    }

    /// Trigger on_frame on all active plugins.
    pub fn on_server_frame(&mut self) {
        self.process_hot_reload();
        for plugin in &mut self.plugins {
            if let Err(err) = plugin.call_on_frame() {
                log::error!("[GoldSrc.rs WASM Host] Error in plugin {}: {}", plugin.name, err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.plugins.len(), 0);
    }
}

