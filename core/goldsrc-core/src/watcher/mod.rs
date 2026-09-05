//! Unified filesystem watcher service and hot-reload debouncer.
//!
//! Provides declarative specification of filesystem watch targets ([`WatchTarget`]),
//! granular zero-allocation filtering ([`WatcherFilter`]), per-watcher debouncing,
//! dynamic pause/resume, and telemetry status inspection for admin tooling and CLI.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use notify::Watcher;

/// Standard default debounce delay between reloads of the same file (500ms).
pub const DEFAULT_RELOAD_DEBOUNCE: Duration = Duration::from_millis(500);

/// Granular filter applied to filesystem modification events in watched targets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatcherFilter {
    /// Matches all files (`*`).
    Any,
    /// Fast zero-allocation matching by file extension (e.g. `"wasm"`, `"toml"`).
    Extension(&'static str),
    /// Fast zero-allocation matching by file stem without extension (e.g. `"plugins"` for `plugins.toml`).
    Stem(&'static str),
    /// Fast zero-allocation matching by exact filename (e.g. `"plugins.toml"`).
    ExactName(&'static str),
    /// Full pattern / substring glob matching.
    Pattern(String),
}

impl WatcherFilter {
    /// Evaluates whether `path` satisfies the filter.
    pub fn matches(&self, path: &Path) -> bool {
        match self {
            Self::Any => true,
            Self::Extension(ext) => path
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case(ext)),
            Self::Stem(stem) => path
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case(stem)),
            Self::ExactName(name) => path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case(name)),
            Self::Pattern(pat) => {
                let s = path.to_string_lossy();
                s.contains(pat)
            }
        }
    }

    /// Human-readable label for inspection and CLI.
    pub fn display_label(&self) -> String {
        match self {
            Self::Any => "*".to_string(),
            Self::Extension(ext) => format!("*.{ext}"),
            Self::Stem(stem) => format!("{stem}.*"),
            Self::ExactName(name) => (*name).to_string(),
            Self::Pattern(pat) => pat.clone(),
        }
    }
}

/// Target of a filesystem watcher.
/// Ensures invalid states (such as recursive watch on a single file with an extension filter)
/// cannot be represented at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchTarget {
    /// Watch a specific standalone file directly.
    File(PathBuf),
    /// Watch a directory tree with recursive flag and file filter.
    Directory {
        path: PathBuf,
        recursive: bool,
        filter: WatcherFilter,
    },
}

impl WatchTarget {
    /// Returns the root path monitored by this target.
    pub fn root_path(&self) -> &Path {
        match self {
            Self::File(p) => p.as_path(),
            Self::Directory { path, .. } => path.as_path(),
        }
    }

    /// Evaluates if the event path matches this target's criteria.
    pub fn matches_event(&self, event_path: &Path) -> bool {
        match self {
            Self::File(expected_file) => {
                event_path == expected_file
                    || event_path
                        .canonicalize()
                        .ok()
                        .zip(expected_file.canonicalize().ok())
                        .is_some_and(|(a, b)| a == b)
            }
            Self::Directory {
                path: base_dir,
                filter,
                ..
            } => {
                if !event_path.starts_with(base_dir) {
                    if let (Ok(can_event), Ok(can_base)) =
                        (event_path.canonicalize(), base_dir.canonicalize())
                    {
                        if !can_event.starts_with(&can_base) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                filter.matches(event_path)
            }
        }
    }
}

/// Declarative specification for registering a filesystem watcher.
#[derive(Debug, Clone)]
pub struct WatcherSpec {
    /// Namespaced identifier in `<source>:<name>` format (e.g. `"core:plugins"`, `"i18n:dicts"`).
    pub id: String,
    /// Watch target (File or Directory).
    pub target: WatchTarget,
    /// Minimum time between reloads of the same file.
    pub debounce: Duration,
}

impl WatcherSpec {
    /// Creates a directory watcher specification with standard 500ms debounce.
    pub fn directory<P: Into<PathBuf>>(
        id: impl Into<String>,
        path: P,
        filter: WatcherFilter,
        recursive: bool,
    ) -> Self {
        Self {
            id: id.into(),
            target: WatchTarget::Directory {
                path: path.into(),
                recursive,
                filter,
            },
            debounce: DEFAULT_RELOAD_DEBOUNCE,
        }
    }

    /// Creates a file watcher specification with standard 500ms debounce.
    pub fn file<P: Into<PathBuf>>(id: impl Into<String>, path: P) -> Self {
        Self {
            id: id.into(),
            target: WatchTarget::File(path.into()),
            debounce: DEFAULT_RELOAD_DEBOUNCE,
        }
    }

    /// Sets custom debounce duration.
    pub fn with_debounce(mut self, debounce: Duration) -> Self {
        self.debounce = debounce;
        self
    }
}

/// An incoming filesystem event emitted by the watcher service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatcherEvent {
    /// Registered ID of the watcher that triggered this event.
    pub watcher_id: String,
    /// Path of the modified / created file.
    pub path: PathBuf,
}

/// Summary statistics of active/paused watchers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct WatcherOverview {
    pub total: usize,
    pub active: usize,
    pub paused: usize,
}

/// Public telemetry status of a registered watcher.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WatcherStatus {
    pub id: String,
    pub target_type: &'static str,
    pub path: PathBuf,
    pub filter_desc: String,
    pub recursive: bool,
    pub is_paused: bool,
    pub debounce_ms: u64,
    pub events_fired: u64,
    #[serde(skip_serializing)]
    pub last_event: Option<Instant>,
}

/// Errors that can occur during watcher registration and operation.
#[derive(Debug, thiserror::Error)]
pub enum WatcherError {
    #[error("Watcher IO error on '{0}': {1}")]
    Io(PathBuf, #[source] std::io::Error),

    #[error("Notify system error on '{0}': {1}")]
    Notify(PathBuf, #[source] notify::Error),

    #[error("Watcher with ID '{0}' is already registered")]
    AlreadyExists(String),
}

struct InternalWatcherEntry {
    spec: WatcherSpec,
    _handle: notify::RecommendedWatcher,
    is_paused: bool,
    events_fired: u64,
    last_event: Option<Instant>,
    last_reload_by_path: HashMap<PathBuf, Instant>,
}

/// Centralized filesystem watcher service managing system and plugin hot-reload hooks.
pub struct WatcherService {
    watchers: HashMap<String, InternalWatcherEntry>,
    event_tx: crossbeam_channel::Sender<WatcherEvent>,
    event_rx: crossbeam_channel::Receiver<WatcherEvent>,
}

impl Default for WatcherService {
    fn default() -> Self {
        Self::new()
    }
}

impl WatcherService {
    /// Creates a new uninitialized `WatcherService`.
    pub fn new() -> Self {
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        Self {
            watchers: HashMap::new(),
            event_tx,
            event_rx,
        }
    }

    /// Registers and activates a new watcher according to [`WatcherSpec`].
    pub fn register(&mut self, spec: WatcherSpec) -> Result<(), WatcherError> {
        if self.watchers.contains_key(&spec.id) {
            return Err(WatcherError::AlreadyExists(spec.id));
        }

        let watch_path = match &spec.target {
            WatchTarget::File(p) => {
                if let Some(parent) = p.parent() {
                    parent.to_path_buf()
                } else {
                    p.clone()
                }
            }
            WatchTarget::Directory { path, .. } => path.clone(),
        };

        let _ = std::fs::create_dir_all(&watch_path);

        let tx = self.event_tx.clone();
        let target_clone = spec.target.clone();
        let id_clone = spec.id.clone();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                for p in event.paths {
                    if target_clone.matches_event(&p) {
                        let _ = tx.send(WatcherEvent {
                            watcher_id: id_clone.clone(),
                            path: p,
                        });
                    }
                }
            }
        })
        .map_err(|e| WatcherError::Notify(watch_path.clone(), e))?;

        let mode = match &spec.target {
            WatchTarget::File(_) => notify::RecursiveMode::NonRecursive,
            WatchTarget::Directory { recursive, .. } => {
                if *recursive {
                    notify::RecursiveMode::Recursive
                } else {
                    notify::RecursiveMode::NonRecursive
                }
            }
        };

        watcher
            .watch(&watch_path, mode)
            .map_err(|e| WatcherError::Notify(watch_path, e))?;

        self.watchers.insert(
            spec.id.clone(),
            InternalWatcherEntry {
                spec,
                _handle: watcher,
                is_paused: false,
                events_fired: 0,
                last_event: None,
                last_reload_by_path: HashMap::new(),
            },
        );

        Ok(())
    }

    /// Suspends event forwarding for watcher `id`.
    pub fn pause(&mut self, id: &str) -> bool {
        if let Some(entry) = self.watchers.get_mut(id) {
            entry.is_paused = true;
            true
        } else {
            false
        }
    }

    /// Resumes event forwarding for watcher `id`.
    pub fn resume(&mut self, id: &str) -> bool {
        if let Some(entry) = self.watchers.get_mut(id) {
            entry.is_paused = false;
            true
        } else {
            false
        }
    }

    /// Returns the number of currently registered watchers.
    pub fn watcher_count(&self) -> usize {
        self.watchers.len()
    }

    /// Returns high-level statistics of registered watchers.
    pub fn status(&self) -> WatcherOverview {
        let total = self.watchers.len();
        let paused = self.watchers.values().filter(|w| w.is_paused).count();
        WatcherOverview {
            total,
            active: total.saturating_sub(paused),
            paused,
        }
    }

    /// Drains all debounced filesystem events from the channel that occurred since last call.
    /// Drops duplicate events within the configured debounce window per watcher/path.
    pub fn drain_events(&mut self) -> Vec<WatcherEvent> {
        let mut ready = Vec::new();
        let now = Instant::now();

        while let Ok(event) = self.event_rx.try_recv() {
            let Some(entry) = self.watchers.get_mut(&event.watcher_id) else {
                continue;
            };

            if entry.is_paused {
                continue;
            }

            if let Some(last) = entry.last_reload_by_path.get(&event.path)
                && now.duration_since(*last) < entry.spec.debounce
            {
                continue;
            }

            entry.last_reload_by_path.insert(event.path.clone(), now);
            entry.events_fired += 1;
            entry.last_event = Some(now);

            ready.push(event);
        }

        ready
    }

    /// Returns telemetry status for all registered watchers, sorted by ID.
    pub fn list_watchers(&self) -> Vec<WatcherStatus> {
        let mut list: Vec<_> = self
            .watchers
            .values()
            .map(|w| {
                let (target_type, filter_desc, recursive) = match &w.spec.target {
                    WatchTarget::File(_) => ("File", "-".to_string(), false),
                    WatchTarget::Directory {
                        filter, recursive, ..
                    } => ("Directory", filter.display_label(), *recursive),
                };

                WatcherStatus {
                    id: w.spec.id.clone(),
                    target_type,
                    path: w.spec.target.root_path().to_path_buf(),
                    filter_desc,
                    recursive,
                    is_paused: w.is_paused,
                    debounce_ms: w.spec.debounce.as_millis() as u64,
                    events_fired: w.events_fired,
                    last_event: w.last_event,
                }
            })
            .collect();

        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_filter_matches_correctly() {
        let wasm_filter = WatcherFilter::Extension("wasm");
        assert!(wasm_filter.matches(Path::new("cstrike/plugins/vip.wasm")));
        assert!(!wasm_filter.matches(Path::new("cstrike/plugins/vip.toml")));

        let stem_filter = WatcherFilter::Stem("plugins");
        assert!(stem_filter.matches(Path::new("cstrike/configs/plugins.toml")));
        assert!(stem_filter.matches(Path::new("cstrike/configs/plugins.json")));
        assert!(!stem_filter.matches(Path::new("cstrike/configs/other.toml")));

        let exact_filter = WatcherFilter::ExactName("plugins.toml");
        assert!(exact_filter.matches(Path::new("configs/plugins.toml")));
        assert!(!exact_filter.matches(Path::new("configs/plugins.json")));

        let pattern_filter = WatcherFilter::Pattern("lang".to_string());
        assert!(pattern_filter.matches(Path::new("data/lang/vip.toml")));
        assert!(!pattern_filter.matches(Path::new("data/sounds/beep.wav")));

        let any_filter = WatcherFilter::Any;
        assert!(any_filter.matches(Path::new("any/file.txt")));
    }

    #[test]
    fn test_watch_target_file_and_directory_evaluation() {
        let file_target = WatchTarget::File(PathBuf::from("configs/plugins.toml"));
        assert!(file_target.matches_event(Path::new("configs/plugins.toml")));
        assert!(!file_target.matches_event(Path::new("configs/other.toml")));

        let dir_target = WatchTarget::Directory {
            path: PathBuf::from("plugins"),
            recursive: true,
            filter: WatcherFilter::Extension("wasm"),
        };
        assert!(dir_target.matches_event(Path::new("plugins/vip.wasm")));
        assert!(dir_target.matches_event(Path::new("plugins/nested/sub.wasm")));
        assert!(!dir_target.matches_event(Path::new("plugins/vip.toml")));
    }
}
