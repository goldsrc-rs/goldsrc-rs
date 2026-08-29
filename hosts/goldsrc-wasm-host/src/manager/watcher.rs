//! Hot-reload filesystem watcher and debounce tracking.

use crate::error::CommandError;
use notify::Watcher;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::Duration;

/// Minimum delay between reloads of the same file (debounces editor save spam).
pub const RELOAD_DEBOUNCE: Duration = Duration::from_millis(500);

/// Spawns a `notify` watcher on `dir` that forwards changed files with
/// extension `ext` to the event channel.
pub fn spawn_watcher<P: AsRef<Path>>(
    dir: P,
    ext: &'static str,
    event_tx: Sender<PathBuf>,
) -> Result<notify::RecommendedWatcher, CommandError> {
    let dir = dir.as_ref().to_path_buf();
    let _ = std::fs::create_dir_all(&dir);
    let tx = event_tx;
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            for path in event.paths {
                if path
                    .extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|e| e == ext)
                {
                    let _ = tx.send(path);
                }
            }
        }
    })
    .map_err(|e| CommandError::Watcher(e.to_string()))?;

    watcher
        .watch(&dir, notify::RecursiveMode::Recursive)
        .map_err(|e| CommandError::Watcher(e.to_string()))?;

    Ok(watcher)
}
