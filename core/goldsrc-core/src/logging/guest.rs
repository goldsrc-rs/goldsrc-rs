//! Transparent WASM guest logger — zero-cost facade over `log`.
//!
//! Plugins call standard `log::info!` / `log::warn!` etc. In `wasm32` the
//! global logger forwards to `host_log` (WIT import); on native it is a
//! no-op (host's `GoldSrcLogger` handles output).

#[cfg(target_arch = "wasm32")]
use std::sync::Once;

/// WASM guest logger that bridges `log` to the host.
pub struct WasmLogger;

impl log::Log for WasmLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let target = record.target();
            let msg = if target == "plugin" || target.is_empty() {
                format!("[{}] {}", record.level(), record.args())
            } else {
                format!("[{}][{}] {}", record.level(), target, record.args())
            };
            crate::goldsrc_api::bindings::goldsrc::engine::api::host_log(&msg);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Native fallback — respect target, avoid recursion into `log` again.
            eprintln!(
                "[{}][{}] {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

#[cfg(target_arch = "wasm32")]
static WASM_LOGGER: WasmLogger = WasmLogger;
#[cfg(target_arch = "wasm32")]
static INIT_ONCE: Once = Once::new();

/// Initialise the guest global logger once. Safe to call multiple times
/// (e.g. from `#[plugin]` generated `on_load`). No-op on native (host has
/// its own `GoldSrcLogger`).
pub fn init_guest_logger() {
    #[cfg(target_arch = "wasm32")]
    INIT_ONCE.call_once(|| {
        let _ = log::set_logger(&WASM_LOGGER);
        log::set_max_level(log::LevelFilter::Info);
    });
}
