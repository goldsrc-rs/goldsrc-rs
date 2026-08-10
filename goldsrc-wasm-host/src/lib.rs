//! WASM plugin host for GoldSrc.rs.
//!
//! Uses `wasmi` as the pure-Rust WASM runtime for maximum compatibility and safety
//! with 32-bit HLDS without C/C++ build system dependencies.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
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

/// Metadata structure exported by WASM plugins generated via #[plugin] macro.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PluginMetadata {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub systems: Vec<String>,
    #[serde(default)]
    pub dependencies: std::collections::HashMap<String, String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Single loaded WASM plugin module instance.
pub struct LoadedPlugin {
    pub name: String,
    pub path: PathBuf,
    pub is_paused: bool,
    pub metadata: Option<PluginMetadata>,
    store: Store<HostState>,
    on_load_fn: Option<TypedFunc<(), ()>>,
    on_unload_fn: Option<TypedFunc<(), ()>>,
    on_frame_fn: Option<TypedFunc<(), ()>>,
}

impl LoadedPlugin {
    pub fn call_on_load(&mut self) -> Result<(), RuntimeError> {
        if let Some(f) = &self.on_load_fn {
            f.call(&mut self.store, ())
                .map_err(|e| RuntimeError::ExecutionError(e.to_string()))?;
        }
        Ok(())
    }

    pub fn call_on_unload(&mut self) -> Result<(), RuntimeError> {
        if let Some(f) = &self.on_unload_fn {
            f.call(&mut self.store, ())
                .map_err(|e| RuntimeError::ExecutionError(e.to_string()))?;
        }
        Ok(())
    }
    pub fn call_on_frame(&mut self) -> Result<(), RuntimeError> {
        if self.is_paused {
            return Ok(());
        }
        if let Some(f) = &self.on_frame_fn {
            f.call(&mut self.store, ())
                .map_err(|e| RuntimeError::ExecutionError(e.to_string()))?;
        }
        Ok(())
    }
}

/// WASM Plugin Manager — handles runtime execution, host imports, and hot-reload.
pub struct PluginManager {
    engine: Engine,
    plugins: Vec<LoadedPlugin>,
    watcher_rx: Receiver<notify::Result<notify::Event>>,
    watcher_tx: std::sync::mpsc::Sender<notify::Result<notify::Event>>,
    _watchers: Vec<RecommendedWatcher>,
    watched_dirs: Vec<PathBuf>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

static PRINT_CALLBACK: std::sync::RwLock<Option<fn(&str)>> = std::sync::RwLock::new(None);

/// Set global callback for WASM server_print calls.
pub fn set_print_callback(f: fn(&str)) {
    if let Ok(mut lock) = PRINT_CALLBACK.write() {
        *lock = Some(f);
    }
}

/// Print log message via host callback (engine server_print).
pub fn host_log(msg: &str) {
    #[allow(clippy::collapsible_if)]
    if let Ok(lock) = PRINT_CALLBACK.read() {
        if let Some(print_fn) = *lock {
            print_fn(msg);
            return;
        }
    }
    println!("{}", msg);
}

impl PluginManager {
    pub fn new() -> Self {
        let engine = Engine::default();
        let (tx, rx) = channel();
        Self {
            engine,
            plugins: Vec::new(),
            watcher_rx: rx,
            watcher_tx: tx,
            _watchers: Vec::new(),
            watched_dirs: Vec::new(),
        }
    }

    /// Setup hot-reload file watching on a directory and load existing plugins.
    pub fn enable_hot_reload<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), RuntimeError> {
        let dir_ref = dir.as_ref();
        if !dir_ref.exists() {
            return Ok(());
        }

        self.watched_dirs.push(dir_ref.to_path_buf());

        host_log(&format!(
            "[GoldSrc.rs WASM Host] Watching directory {:?}\n",
            dir_ref
        ));

        let tx = self.watcher_tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )?;

        watcher.watch(dir_ref, RecursiveMode::NonRecursive)?;
        self._watchers.push(watcher);

        if let Ok(entries) = fs::read_dir(dir_ref) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "wasm") {
                    let res = self.load_plugin(&path);
                    if let Err(err) = res {
                        host_log(&format!(
                            "[GoldSrc.rs WASM Host] Failed to load {:?}: {}\n",
                            path, err
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate SemVer dependencies across all loaded plugins and sort them topologically.
    pub fn validate_and_sort_dependencies(&mut self) -> Vec<String> {
        use semver::{Version, VersionReq};
        use std::collections::HashMap;

        let mut errors = Vec::new();

        // Map plugin identifiers (metadata name or filename) to index and parsed version
        let mut plugin_map: HashMap<String, (usize, Version)> = HashMap::new();

        for (idx, plugin) in self.plugins.iter().enumerate() {
            let (name, ver_str) = if let Some(meta) = &plugin.metadata {
                (meta.name.clone(), meta.version.clone())
            } else {
                let stem = plugin
                    .path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                (stem, "1.0.0".to_string())
            };

            let version = Version::parse(&ver_str).unwrap_or_else(|_| Version::new(1, 0, 0));
            plugin_map.insert(name, (idx, version));
        }

        // Validate dependency constraints
        for plugin in &self.plugins {
            if let Some(meta) = &plugin.metadata {
                for (dep_name, req_str) in &meta.dependencies {
                    if let Some((_, dep_ver)) = plugin_map.get(dep_name) {
                        if let Ok(req) = VersionReq::parse(req_str) {
                            if !req.matches(dep_ver) {
                                errors.push(format!(
                                    "Plugin '{}' requires '{}' {}, but version '{}' is loaded",
                                    meta.name, dep_name, req_str, dep_ver
                                ));
                            }
                        } else {
                            errors.push(format!(
                                "Plugin '{}' has invalid version constraint requirement '{}' for '{}'",
                                meta.name, req_str, dep_name
                            ));
                        }
                    } else {
                        errors.push(format!(
                            "Plugin '{}' missing required dependency '{}'",
                            meta.name, dep_name
                        ));
                    }
                }
            }
        }

        // Topological Sort (Kahn's Algorithm)
        let n = self.plugins.len();
        let mut in_degree = vec![0usize; n];
        let mut adj = vec![Vec::new(); n];

        for (idx, plugin) in self.plugins.iter().enumerate() {
            if let Some(meta) = &plugin.metadata {
                for dep_name in meta.dependencies.keys() {
                    if let Some(&(dep_idx, _)) = plugin_map.get(dep_name) {
                        adj[dep_idx].push(idx);
                        in_degree[idx] += 1;
                    }
                }
            }
        }

        let mut queue = Vec::new();
        for (i, degree) in in_degree.iter().enumerate() {
            if *degree == 0 {
                queue.push(i);
            }
        }

        let mut sorted_indices = Vec::new();
        while let Some(u) = queue.pop() {
            sorted_indices.push(u);
            for &v in &adj[u] {
                in_degree[v] -= 1;
                if in_degree[v] == 0 {
                    queue.push(v);
                }
            }
        }

        if sorted_indices.len() == n {
            let mut old_plugins: Vec<Option<LoadedPlugin>> =
                self.plugins.drain(..).map(Some).collect();
            for idx in sorted_indices {
                if let Some(p) = old_plugins[idx].take() {
                    self.plugins.push(p);
                }
            }
        } else if errors.is_empty() {
            errors.push("Circular dependency detected among WASM plugins".to_string());
        }

        errors
    }

    /// Unload plugin by path, invoking on_unload callback if present.
    pub fn unload_plugin<P: AsRef<Path>>(&mut self, path: P) {
        let p_ref = path.as_ref();
        if let Some(idx) = self.plugins.iter().position(|p| p.path == p_ref) {
            let mut plugin = self.plugins.remove(idx);
            let _ = plugin.call_on_unload();
            host_log(&format!(
                "[GoldSrc.rs WASM Host] Unloaded plugin {:?}\n",
                plugin.name
            ));
        }
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
                wasmi::Func::wrap(
                    &mut store,
                    |caller: wasmi::Caller<'_, HostState>, msg_ptr: i32, msg_len: i32| {
                        let Some(wasmi::Extern::Memory(mem)) = caller.get_export("memory") else {
                            return;
                        };
                        let mut buf = vec![0u8; msg_len as usize];
                        if mem.read(&caller, msg_ptr as usize, &mut buf).is_err() {
                            return;
                        };
                        let Ok(s) = std::str::from_utf8(&buf) else {
                            return;
                        };
                        host_log(s);
                    },
                ),
            )
            .map_err(|e| RuntimeError::LoadError(e.to_string()))?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| RuntimeError::LoadError(e.to_string()))?
            .start(&mut store)
            .map_err(|e| RuntimeError::ExecutionError(e.to_string()))?;

        let on_load_fn = instance.get_typed_func::<(), ()>(&store, "on_load").ok();
        let on_unload_fn = instance.get_typed_func::<(), ()>(&store, "on_unload").ok();
        let on_frame_fn = instance.get_typed_func::<(), ()>(&store, "on_frame").ok();

        let metadata = if let Ok(meta_fn) =
            instance.get_typed_func::<(), i32>(&store, "__goldsrc_plugin_metadata")
        {
            if let Ok(ptr) = meta_fn.call(&mut store, ()) {
                if let Some(wasmi::Extern::Memory(mem)) = instance.get_export(&store, "memory") {
                    let mut bytes = Vec::new();
                    let mut offset = ptr as usize;
                    let mut byte = [0u8; 1];
                    while mem.read(&store, offset, &mut byte).is_ok() && byte[0] != 0 {
                        bytes.push(byte[0]);
                        offset += 1;
                    }
                    if let Ok(json_str) = std::str::from_utf8(&bytes) {
                        serde_json::from_str::<PluginMetadata>(json_str).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let plugin_name = path_buf
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        let mut loaded = LoadedPlugin {
            name: plugin_name.clone(),
            path: path_buf,
            is_paused: false,
            metadata,
            store,
            on_load_fn,
            on_unload_fn,
            on_frame_fn,
        };

        loaded.call_on_load()?;
        host_log(&format!(
            "[GoldSrc.rs WASM Host] Loaded plugin {}\n",
            plugin_name
        ));
        self.plugins.push(loaded);
        Ok(())
    }

    /// Poll for file changes and handle all plugin lifecycle events (Create, Modify, Delete).
    pub fn process_hot_reload(&mut self) {
        let mut reload_paths = Vec::new();
        while let Ok(Ok(event)) = self.watcher_rx.try_recv() {
            for path in event.paths {
                if path.extension().is_some_and(|ext| ext == "wasm") {
                    reload_paths.push(path);
                }
            }
        }

        reload_paths.sort();
        reload_paths.dedup();

        for path in reload_paths {
            if path.exists() {
                // Scenario 1 & 2: Plugin Created or Overwritten/Modified
                let is_reload = self.plugins.iter().any(|p| p.path == path);
                self.unload_plugin(&path);

                if let Err(err) = self.load_plugin(&path) {
                    host_log(&format!(
                        "[GoldSrc.rs WASM Host] Failed to {} {:?}: {}\n",
                        if is_reload { "reload" } else { "load" },
                        path,
                        err
                    ));
                } else if is_reload {
                    host_log(&format!(
                        "[GoldSrc.rs WASM Host] Reloaded plugin {:?}\n",
                        path
                    ));
                }
            } else {
                // Scenario 3: Plugin Deleted / Removed / Renamed away
                self.unload_plugin(&path);
            }
        }
    }

    /// Trigger on_frame on all active plugins.
    pub fn on_server_frame(&mut self) {
        self.process_hot_reload();
        for plugin in &mut self.plugins {
            if let Err(err) = plugin.call_on_frame() {
                host_log(&format!(
                    "[GoldSrc.rs WASM Host] Error in plugin {}: {}\n",
                    plugin.name, err
                ));
            }
        }
    }

    /// Get list of plugin info for CLI commands.
    pub fn get_plugins_info(&self) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .enumerate()
            .map(|(idx, p)| PluginInfo {
                index: idx + 1,
                name: p.name.clone(),
                path: p.path.clone(),
                is_paused: p.is_paused,
                has_on_load: p.on_load_fn.is_some(),
                has_on_unload: p.on_unload_fn.is_some(),
                has_on_frame: p.on_frame_fn.is_some(),
            })
            .collect()
    }

    /// Find plugin index by 1-based index string or filename/name.
    pub fn find_plugin_index(&self, query: &str) -> Option<usize> {
        if let Some(idx) = query
            .parse::<usize>()
            .ok()
            .filter(|&i| (1..=self.plugins.len()).contains(&i))
        {
            return Some(idx - 1);
        }
        let q = query.trim_end_matches(".wasm");
        self.plugins
            .iter()
            .position(|p| p.name.trim_end_matches(".wasm").eq_ignore_ascii_case(q))
    }

    /// Pause or unpause a loaded plugin.
    pub fn pause_plugin(&mut self, query: &str, pause: bool) -> Result<String, String> {
        if let Some(idx) = self.find_plugin_index(query) {
            self.plugins[idx].is_paused = pause;
            let name = self.plugins[idx].name.clone();
            let action = if pause { "Paused" } else { "Unpaused" };
            Ok(format!(
                "[GoldSrc.rs WASM Host] {} plugin '{}'\n",
                action, name
            ))
        } else {
            Err(format!(
                "[GoldSrc.rs WASM Host] Plugin '{}' not found.\n",
                query
            ))
        }
    }

    /// Unload a plugin by 1-based index or name.
    pub fn unload_plugin_by_query(&mut self, query: &str) -> Result<String, String> {
        if let Some(idx) = self.find_plugin_index(query) {
            let mut plugin = self.plugins.remove(idx);
            let _ = plugin.call_on_unload();
            Ok(format!(
                "[GoldSrc.rs WASM Host] Unloaded plugin '{}'\n",
                plugin.name
            ))
        } else {
            Err(format!(
                "[GoldSrc.rs WASM Host] Plugin '{}' not found.\n",
                query
            ))
        }
    }

    /// Reload a plugin by 1-based index or name.
    pub fn reload_plugin_by_query(&mut self, query: &str) -> Result<String, String> {
        if let Some(idx) = self.find_plugin_index(query) {
            let path = self.plugins[idx].path.clone();
            let _ = self.unload_plugin_by_query(query);
            match self.load_plugin(&path) {
                Ok(_) => Ok(format!(
                    "[GoldSrc.rs WASM Host] Reloaded plugin {:?}\n",
                    path.file_name().unwrap_or_default()
                )),
                Err(err) => Err(format!(
                    "[GoldSrc.rs WASM Host] Failed to reload {:?}: {}\n",
                    path, err
                )),
            }
        } else {
            Err(format!(
                "[GoldSrc.rs WASM Host] Plugin '{}' not found.\n",
                query
            ))
        }
    }

    /// Unload all active plugins.
    pub fn unload_all_plugins(&mut self) -> String {
        let count = self.plugins.len();
        for mut plugin in self.plugins.drain(..) {
            let _ = plugin.call_on_unload();
        }
        format!("[GoldSrc.rs WASM Host] Unloaded all plugins ({})\n", count)
    }

    /// Reload all active plugins.
    pub fn reload_all_plugins(&mut self) -> String {
        let paths: Vec<PathBuf> = self.plugins.iter().map(|p| p.path.clone()).collect();
        let count = paths.len();
        for mut plugin in self.plugins.drain(..) {
            let _ = plugin.call_on_unload();
        }
        let mut reloaded = 0;
        for path in &paths {
            if self.load_plugin(path).is_ok() {
                reloaded += 1;
            }
        }
        format!(
            "[GoldSrc.rs WASM Host] Reloaded {}/{} plugins\n",
            reloaded, count
        )
    }

    /// Pause or unpause all active plugins.
    pub fn pause_all_plugins(&mut self, pause: bool) -> String {
        let count = self.plugins.len();
        for plugin in &mut self.plugins {
            plugin.is_paused = pause;
        }
        let action = if pause { "Paused" } else { "Unpaused" };
        format!(
            "[GoldSrc.rs WASM Host] {} all plugins ({})\n",
            action, count
        )
    }

    /// Load a plugin by name or filename from watched directories.
    pub fn load_plugin_by_name(&mut self, name: &str) -> Result<String, String> {
        let file_name = if name.ends_with(".wasm") {
            name.to_string()
        } else {
            format!("{}.wasm", name)
        };

        for dir in &self.watched_dirs {
            let path = dir.join(&file_name);
            if path.exists() {
                return match self.load_plugin(&path) {
                    Ok(_) => Ok(format!(
                        "[GoldSrc.rs WASM Host] Loaded plugin {:?}\n",
                        file_name
                    )),
                    Err(err) => Err(format!(
                        "[GoldSrc.rs WASM Host] Failed to load {:?}: {}\n",
                        file_name, err
                    )),
                };
            }
        }
        Err(format!(
            "[GoldSrc.rs WASM Host] Plugin file '{}' not found in watched directories.\n",
            file_name
        ))
    }

    /// Get status information (plugins count, active watchers count).
    pub fn get_status_info(&self) -> (usize, usize) {
        (self.plugins.len(), self._watchers.len())
    }
}

/// Information about a loaded WASM plugin for management CLI.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub index: usize,
    pub name: String,
    pub path: PathBuf,
    pub is_paused: bool,
    pub has_on_load: bool,
    pub has_on_unload: bool,
    pub has_on_frame: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.plugins.len(), 0);
        assert_eq!(manager.get_status_info(), (0, 0));
    }

    #[test]
    fn test_find_plugin_index() {
        let mut manager = PluginManager::new();
        manager.plugins.push(LoadedPlugin {
            name: "test_plugin.wasm".to_string(),
            path: PathBuf::from("plugins/test_plugin.wasm"),
            is_paused: false,
            metadata: None,
            store: Store::new(&manager.engine, HostState),
            on_load_fn: None,
            on_unload_fn: None,
            on_frame_fn: None,
        });

        assert_eq!(manager.find_plugin_index("1"), Some(0));
        assert_eq!(manager.find_plugin_index("test_plugin"), Some(0));
        assert_eq!(manager.find_plugin_index("test_plugin.wasm"), Some(0));
        assert_eq!(manager.find_plugin_index("TEST_PLUGIN"), Some(0));
        assert_eq!(manager.find_plugin_index("2"), None);
        assert_eq!(manager.find_plugin_index("unknown"), None);

        assert!(manager.pause_plugin("1", true).is_ok());
        assert!(manager.plugins[0].is_paused);
        assert!(manager.pause_plugin("1", false).is_ok());
        assert!(!manager.plugins[0].is_paused);

        let info = manager.get_plugins_info();
        assert_eq!(info.len(), 1);
        assert_eq!(info[0].name, "test_plugin.wasm");
        assert_eq!(info[0].index, 1);
    }

    #[test]
    fn test_dependency_resolution() {
        let mut manager = PluginManager::new();

        let mut deps_child = std::collections::HashMap::new();
        deps_child.insert("parent_plugin".to_string(), "^1.0.0".to_string());

        // Child loaded FIRST, parent loaded SECOND
        manager.plugins.push(LoadedPlugin {
            name: "child_plugin.wasm".to_string(),
            path: PathBuf::from("plugins/child_plugin.wasm"),
            is_paused: false,
            metadata: Some(PluginMetadata {
                name: "child_plugin".to_string(),
                version: "1.0.0".to_string(),
                systems: vec![],
                dependencies: deps_child,
            }),
            store: Store::new(&manager.engine, HostState),
            on_load_fn: None,
            on_unload_fn: None,
            on_frame_fn: None,
        });

        manager.plugins.push(LoadedPlugin {
            name: "parent_plugin.wasm".to_string(),
            path: PathBuf::from("plugins/parent_plugin.wasm"),
            is_paused: false,
            metadata: Some(PluginMetadata {
                name: "parent_plugin".to_string(),
                version: "1.2.0".to_string(),
                systems: vec![],
                dependencies: std::collections::HashMap::new(),
            }),
            store: Store::new(&manager.engine, HostState),
            on_load_fn: None,
            on_unload_fn: None,
            on_frame_fn: None,
        });

        let errors = manager.validate_and_sort_dependencies();
        assert!(
            errors.is_empty(),
            "Expected no dependency errors: {:?}",
            errors
        );

        // Verify topological sorting: parent MUST come before child
        assert_eq!(manager.plugins[0].name, "parent_plugin.wasm");
        assert_eq!(manager.plugins[1].name, "child_plugin.wasm");
    }
}
