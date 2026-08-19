//! Centralized hook dispatcher and safe event emission for backends.

use crate::host::HostRuntime;

/// Dispatches an event with an optional payload to all loaded WASM plugins.
/// Returns `true` if the host runtime is active and processed the event.
pub fn emit_event(name: &str, payload: &[u8]) -> bool {
    HostRuntime::with_manager(|m| match m {
        Some(manager) => {
            manager.call_on_event(name, payload);
            true
        }
        None => {
            log::trace!(target: "core", "emit_event('{name}') skipped: WASM host not initialized");
            false
        }
    })
}

/// Dispatches a player-indexed event (payload is player index as 4-byte LE).
/// Returns `true` if the host runtime is active and processed the event.
pub fn emit_player_event(name: &str, index: i32) -> bool {
    emit_event(name, &index.to_le_bytes())
}

/// Dispatches a console / client command to the WASM host.
/// Returns `true` if the host runtime is active and processed the command.
pub fn dispatch_command(cmd: &str, args: &str) -> bool {
    HostRuntime::with_manager(|m| match m {
        Some(manager) => {
            manager.dispatch_command(cmd, args);
            true
        }
        None => {
            log::trace!(target: "core", "dispatch_command('{cmd}') skipped: WASM host not initialized");
            false
        }
    })
}

/// Ticks the frame event in the WASM host.
pub fn on_server_frame() {
    HostRuntime::on_server_frame();
}
