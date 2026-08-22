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
//! goldsrc::logging::init(config.logging.clone(), move |msg| {
//!     engine.server_print(msg);
//! });
//!
//! // Anywhere in the codebase:
//! use goldsrc::logging::{LogLevel, LogTarget};
//! goldsrc_sys::gslog!(LogLevel::Info, LogTarget::Core, "Plugin count: {}", n);
//! ```

use crate::paths::{BackendType, PathResolver};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Mutex,
};

// ============================================================================
// Configuration (matches [logging] section in goldsrc.toml)
// ============================================================================

/// Minimum log level. Messages below this level are silently dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Most verbose; messages for fine-grained tracing.
    Trace,
    /// Debug diagnostics.
    Debug,
    /// Normal operational messages.
    Info,
    /// Warnings that do not stop execution.
    Warn,
    /// Errors that may indicate failure.
    Error,
}

impl LogLevel {
    /// Returns the fixed-width tag used in log lines (e.g. `"INFO "`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }

    /// Convert from `log::Level`
    pub fn from_log_level(level: log::Level) -> Self {
        match level {
            log::Level::Trace => Self::Trace,
            log::Level::Debug => Self::Debug,
            log::Level::Info => Self::Info,
            log::Level::Warn => Self::Warn,
            log::Level::Error => Self::Error,
        }
    }

    /// Convert to `log::LevelFilter`
    pub fn to_level_filter(self) -> log::LevelFilter {
        match self {
            Self::Trace => log::LevelFilter::Trace,
            Self::Debug => log::LevelFilter::Debug,
            Self::Info => log::LevelFilter::Info,
            Self::Warn => log::LevelFilter::Warn,
            Self::Error => log::LevelFilter::Error,
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
    /// Returns the lowercase name used in log lines (e.g. `"core"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Proxy => "proxy",
            Self::Wasm => "wasm",
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

struct GoldSrcLogger {
    config: LogConfig,
    log_path: PathBuf,
    file_handle: Option<std::fs::File>,
    /// Optional callback forwarding messages to the server console.
    console_cb: Option<ConsoleCb>,
}

impl GoldSrcLogger {
    fn new(config: LogConfig, backend_type: BackendType, console_cb: Option<ConsoleCb>) -> Self {
        let fw_dir = PathResolver::framework_dir(backend_type);
        let log_dir = fw_dir.join(crate::paths::LOGS_DIR_NAME);
        let _ = fs::create_dir_all(&log_dir);
        let log_path = log_dir.join(goldsrc_api::consts::DEFAULT_LOG_FILE_NAME);
        let file_handle = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();
        Self {
            config,
            log_path,
            file_handle,
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

    fn emit(&mut self, level: LogLevel, target: LogTarget, message: &str) {
        if !self.should_emit(level, target) {
            return;
        }

        // Clean plain text format for file: [INFO][core] message
        let plain_line = format!("[{}][{}] {}\n", level.as_str(), target.as_str(), message);

        // File output (re-uses open file handle if available, falls back to open on demand).
        if self.config.file_output {
            if let Some(ref mut file) = self.file_handle {
                let _ = file.write_all(plain_line.as_bytes());
            } else if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.log_path)
            {
                let _ = file.write_all(plain_line.as_bytes());
                self.file_handle = Some(file);
            }
        }

        // Console output with ANSI colors
        if self.config.console_output
            && let Some(ref cb) = self.console_cb
        {
            // ANSI color mapping:
            // Trace: Cyan (\x1b[36m), Debug: Gray/Dim (\x1b[90m), Info: Green (\x1b[32m),
            // Warn: Yellow (\x1b[33m), Error: Red (\x1b[31m), Reset: \x1b[0m
            let level_color = match level {
                LogLevel::Trace => "\x1b[36m",
                LogLevel::Debug => "\x1b[90m",
                LogLevel::Info => "\x1b[32m",
                LogLevel::Warn => "\x1b[33m",
                LogLevel::Error => "\x1b[31m",
            };
            let target_color = "\x1b[35m"; // Magenta for target
            let reset = "\x1b[0m";

            let console_line = format!(
                "[{level_color}{}{reset}][{target_color}{}{reset}] {}\n",
                level.as_str(),
                target.as_str(),
                message
            );
            cb(&console_line);
        }
    }
}

// ============================================================================
// Global singleton access
// ============================================================================

struct LoggerImpl {
    inner: Mutex<GoldSrcLogger>,
}

impl log::Log for LoggerImpl {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = LogLevel::from_log_level(record.level());

        let target = match record.target() {
            "proxy" => LogTarget::Proxy,
            "wasm" => LogTarget::Wasm,
            "plugin" => LogTarget::Plugin,
            _ => LogTarget::Core,
        };

        let message = format!("{}", record.args());

        let mut guard = match self.inner.lock() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        guard.emit(level, target, &message);
    }

    fn flush(&self) {}
}

static LOGGER_INSTANCE: std::sync::OnceLock<LoggerImpl> = std::sync::OnceLock::new();

/// Initialise the global logger. Must be called once at server startup,
/// after the engine has provided function pointers.
///
/// `console_cb` is an optional closure that forwards formatted log lines
/// to the server console via `pfnServerPrint`.
pub fn init<F>(config: LogConfig, backend_type: BackendType, console_cb: Option<F>)
where
    F: Fn(&str) + Send + Sync + 'static,
{
    let cb: Option<ConsoleCb> = console_cb.map(|f| Box::new(f) as ConsoleCb);
    let level_filter = config.level.to_level_filter();

    if let Ok(_logger) = LOGGER_INSTANCE.set(LoggerImpl {
        inner: Mutex::new(GoldSrcLogger::new(config, backend_type, cb)),
    }) {
        let _ = log::set_logger(LOGGER_INSTANCE.get().unwrap());
        log::set_max_level(level_filter);
    }
}

/// Returns the path where the log file is written (for display / config).
pub fn log_file_path(backend: BackendType) -> PathBuf {
    PathResolver::framework_dir(backend)
        .join(crate::paths::LOGS_DIR_NAME)
        .join(goldsrc_api::consts::DEFAULT_LOG_FILE_NAME)
}
