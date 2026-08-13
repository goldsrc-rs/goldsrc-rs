//! Unified structured logger for GoldSrc.rs backends.
//!
//! # Architecture
//! The logger is a global singleton (`OnceLock<Mutex<GsLogger>>`) that is
//! initialised once at server startup by the active backend.  Both backends
//! share the same implementation via this crate.
//!
//! # Usage
//! ```ignore
//! // In the backend entry point, after the engine is ready:
//! goldsrc_sys::log::init(config.logging.clone(), move |msg| {
//!     engine.server_print(msg);
//! });
//!
//! // Anywhere in the codebase:
//! use goldsrc_sys::log::{LogLevel, LogTarget};
//! goldsrc_sys::gslog!(LogLevel::Info, LogTarget::Core, "Plugin count: {}", n);
//! ```

use crate::paths::{ADDONS_DIR_NAME, DEFAULT_MOD_DIR, FRAMEWORK_NAME, LOGS_DIR_NAME};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

// ============================================================================
// Configuration (matches [logging] section in goldsrc.toml)
// ============================================================================

/// Minimum log level. Messages below this level are silently dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO ",
            Self::Warn => "WARN ",
            Self::Error => "ERROR",
        }
    }
}

/// Logical sub-system producing the log message. Enables per-target filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogTarget {
    /// Framework core (init, lifecycle, config).
    Core,
    /// GameDLL proxy layer (standalone backend).
    Proxy,
    /// WASM host and plugin management.
    Wasm,
    /// Individual plugin code.
    Plugin,
}

impl LogTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core ",
            Self::Proxy => "proxy",
            Self::Wasm => "wasm ",
            Self::Plugin => "plugin",
        }
    }
}

/// The `[logging]` section of `goldsrc.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Minimum level to emit. Default: `Info`.
    #[serde(default = "default_level")]
    pub level: LogLevel,

    /// Write log lines to `<logs_dir>/goldsrc.log`.
    #[serde(default = "default_true")]
    pub file_output: bool,

    /// Forward log lines to the server console via the registered callback.
    #[serde(default = "default_true")]
    pub console_output: bool,

    /// Restrict output to these targets. Empty vec = all targets allowed.
    #[serde(default)]
    pub targets: Vec<LogTarget>,
}

fn default_level() -> LogLevel {
    LogLevel::Info
}
fn default_true() -> bool {
    true
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_level(),
            file_output: true,
            console_output: true,
            targets: Vec::new(), // all targets
        }
    }
}

// ============================================================================
// Logger singleton
// ============================================================================

type ConsoleCb = Box<dyn Fn(&str) + Send + Sync + 'static>;

struct GsLogger {
    config: LogConfig,
    log_path: PathBuf,
    /// Optional callback forwarding messages to the server console.
    console_cb: Option<ConsoleCb>,
}

impl GsLogger {
    fn new(config: LogConfig, console_cb: Option<ConsoleCb>) -> Self {
        let log_dir = PathBuf::from(DEFAULT_MOD_DIR)
            .join(ADDONS_DIR_NAME)
            .join(FRAMEWORK_NAME)
            .join(LOGS_DIR_NAME);
        let _ = fs::create_dir_all(&log_dir);
        let log_path = log_dir.join("goldsrc.log");
        Self {
            config,
            log_path,
            console_cb,
        }
    }

    fn should_emit(&self, level: LogLevel, target: LogTarget) -> bool {
        if level < self.config.level {
            return false;
        }
        if self.config.targets.is_empty() {
            return true;
        }
        self.config.targets.contains(&target)
    }

    fn emit(&self, level: LogLevel, target: LogTarget, message: &str) {
        if !self.should_emit(level, target) {
            return;
        }

        // Format: [INFO ][core ] message
        let line = format!("[{}][{}] {}\n", level.as_str(), target.as_str(), message);

        // File output.
        if self.config.file_output {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.log_path)
            {
                let _ = file.write_all(line.as_bytes());
            }
        }

        // Console output (via engine callback if registered).
        if self.config.console_output {
            if let Some(ref cb) = self.console_cb {
                cb(&line);
            }
        }
    }
}

// ============================================================================
// Global singleton access
// ============================================================================

static LOGGER: OnceLock<Mutex<GsLogger>> = OnceLock::new();

/// Initialise the global logger.  Must be called once at server startup,
/// after the engine has provided function pointers.
///
/// `console_cb` is an optional closure that forwards formatted log lines
/// to the server console via `pfnServerPrint`.
///
/// Calling `init` a second time is a no-op (the `OnceLock` guarantees
/// single initialisation).
pub fn init<F>(config: LogConfig, console_cb: Option<F>)
where
    F: Fn(&str) + Send + Sync + 'static,
{
    let cb: Option<ConsoleCb> = console_cb.map(|f| Box::new(f) as ConsoleCb);
    // OnceLock::set returns Err if already set — we simply ignore it.
    let _ = LOGGER.set(Mutex::new(GsLogger::new(config, cb)));
}

/// Emit a log line.  If the logger has not been initialised yet,
/// falls back to `eprintln!` (always available, even before engine init).
pub fn log(level: LogLevel, target: LogTarget, message: &str) {
    match LOGGER.get() {
        Some(mtx) => {
            // Poisoned mutex → recover and continue.
            let guard = match mtx.lock() {
                Ok(g) => g,
                Err(e) => e.into_inner(),
            };
            guard.emit(level, target, message);
        }
        None => {
            // Logger not yet initialised — write to stderr as fallback.
            eprintln!(
                "[GoldSrc.rs][{}][{}] {}",
                level.as_str().trim(),
                target.as_str().trim(),
                message
            );
        }
    }
}

/// Returns the path where the log file is written (for display / config).
pub fn log_file_path() -> PathBuf {
    PathBuf::from(DEFAULT_MOD_DIR)
        .join(ADDONS_DIR_NAME)
        .join(FRAMEWORK_NAME)
        .join(LOGS_DIR_NAME)
        .join("goldsrc.log")
}

// ============================================================================
// Convenience macros
// ============================================================================

/// Log with explicit level and target.
///
/// ```ignore
/// gslog!(LogLevel::Warn, LogTarget::Wasm, "Plugin {} failed: {}", name, e);
/// ```
#[macro_export]
macro_rules! gslog {
    ($level:expr, $target:expr, $($arg:tt)*) => {
        $crate::log::log($level, $target, &format!($($arg)*))
    };
}

/// Shorthand macros for each log level (target still required).
#[macro_export]
macro_rules! log_trace {
    ($target:expr, $($arg:tt)*) => {
        $crate::gslog!($crate::log::LogLevel::Trace, $target, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($target:expr, $($arg:tt)*) => {
        $crate::gslog!($crate::log::LogLevel::Debug, $target, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_info {
    ($target:expr, $($arg:tt)*) => {
        $crate::gslog!($crate::log::LogLevel::Info, $target, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($target:expr, $($arg:tt)*) => {
        $crate::gslog!($crate::log::LogLevel::Warn, $target, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_error {
    ($target:expr, $($arg:tt)*) => {
        $crate::gslog!($crate::log::LogLevel::Error, $target, $($arg)*)
    };
}
