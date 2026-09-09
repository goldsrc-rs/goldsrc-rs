//! Centralized event and command dispatcher for backends and WASM plugins.

use crate::host::{HostEvent, HostRuntime};
use goldsrc_api::consts::log_targets;

/// Dispatches an event with an optional payload to all loaded WASM plugins.
/// Returns `true` if the host runtime is active and processed the event.
pub fn emit_event(name: &str, payload: &[u8]) -> bool {
    HostRuntime::with_manager(|m| match m {
        Some(manager) => {
            manager.call_on_event(name, payload);
            true
        }
        None => {
            log::trace!(target: log_targets::CORE, "emit_event('{name}') skipped: WASM host not initialized");
            false
        }
    })
}

/// Dispatches a player-indexed event (payload is player index as 4-byte LE).
/// Returns `true` if the host runtime is active and processed the event.
pub fn emit_player_event(name: &str, index: i32) -> bool {
    if name == "client_disconnect" {
        HostRuntime::on(HostEvent::ClientDisconnect { slot: index });
    } else if name == "client_connect" {
        HostRuntime::on(HostEvent::ClientConnect { slot: index });
    }
    emit_event(name, &index.to_le_bytes())
}

/// Dispatches client userinfo change event and updates active player menu if open.
pub fn on_client_user_info_changed(player_idx: i32) {
    HostRuntime::on(HostEvent::ClientUserInfoChanged { slot: player_idx });
    emit_player_event("client_user_info_changed", player_idx);
}

/// Dispatches a console / client command to the WASM host.
/// Returns `true` if the host runtime is active and processed the command.
pub fn dispatch_command(cmd: &str, args: &str) -> bool {
    HostRuntime::with_manager(|m| match m {
        Some(manager) => manager.dispatch_command(cmd, 0, args),
        None => {
            log::trace!(target: log_targets::CORE, "dispatch_command('{cmd}') skipped: WASM host not initialized");
            false
        }
    })
}

/// Dispatches a client command (including chat commands e.g. `say /vip` or console `vipmenu`).
/// Returns `true` if a plugin intercepted and handled the command (requesting suppression from GameDLL).
pub fn dispatch_client_command(player_idx: i32, cmd: &str, raw_args: &str) -> bool {
    // 1. Check for `menuselect <slot>` client command (slot 1..=10)
    if cmd.eq_ignore_ascii_case("menuselect") {
        let slot = raw_args.trim().parse::<u8>().unwrap_or(0);
        let slot = if slot == 0 { 10 } else { slot };

        if let Some(engine) = HostRuntime::engine() {
            let current_time = HostRuntime::current_time();
            if let Ok(mut mgr) = crate::menu::menu_manager().lock()
                && mgr.handle_menuselect(player_idx, slot, engine.as_ref(), current_time)
            {
                return true;
            }
        }

        // Fallback: dispatch raw slot to WASM plugins event "menu_select" (8 bytes payload: [player_idx: i32, slot: u32])
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&player_idx.to_le_bytes());
        payload.extend_from_slice(&(slot as u32).to_le_bytes());

        let targeted = if let Some(owner) = goldsrc_host_wasm::get_active_menu_owner(player_idx) {
            HostRuntime::with_manager(|m| {
                if let Some(manager) = m {
                    manager.call_plugin_event(&owner, "menu_select", &payload)
                } else {
                    false
                }
            })
        } else {
            false
        };

        if !targeted {
            emit_event("menu_select", &payload);
        }

        return true;
    }

    HostRuntime::with_manager(|m| {
        let Some(manager) = m else {
            return false;
        };

        if cmd.eq_ignore_ascii_case("say") || cmd.eq_ignore_ascii_case("say_team") {
            let mut text = raw_args.trim();
            if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
                text = &text[1..text.len() - 1];
            }
            if text.trim().is_empty() {
                // Suppress empty chat messages
                return true;
            }
            let trimmed_text = text.trim();
            if let Some(first_space_idx) = trimmed_text.find(|c: char| c.is_whitespace()) {
                let trigger = &trimmed_text[..first_space_idx];
                let rest_args = trimmed_text[first_space_idx..].trim_start();
                let clean_trigger = trigger.trim_start_matches(['/', '!']);

                // 1. Try exact clean trigger (e.g. "vip" or "vipmenu")
                if manager.dispatch_command(clean_trigger, player_idx, rest_args) {
                    return true;
                }
                // 2. Try raw trigger (e.g. "/vip")
                if manager.dispatch_command(trigger, player_idx, rest_args) {
                    return true;
                }
            } else {
                let trigger = trimmed_text;
                let clean_trigger = trigger.trim_start_matches(['/', '!']);

                // 1. Try exact clean trigger (e.g. "vip" or "vipmenu")
                if manager.dispatch_command(clean_trigger, player_idx, "") {
                    return true;
                }
                // 2. Try raw trigger (e.g. "/vip")
                if manager.dispatch_command(trigger, player_idx, "") {
                    return true;
                }
            }

            // Route standard player chat through the chat interceptor / placeholder pipeline
            let sender = goldsrc_api::client::Player::new(player_idx);
            let scope = if cmd.eq_ignore_ascii_case("say_team") {
                goldsrc_api::chat::ChatScope::same_team()
            } else {
                goldsrc_api::chat::ChatScope::all()
            };
            return crate::chat::process_chat_message_with_manager(
                Some(manager),
                sender,
                text,
                scope,
            );
        }

        // Direct client console command
        manager.dispatch_command(cmd, player_idx, raw_args)
    })
}

/// Invoked when a new server map is activated (ServerActivate).
pub fn on_server_activate() {
    HostRuntime::on(HostEvent::ServerActivate);
    emit_event("server_activate", &[]);
}

/// Invoked when the current server map is ending or server shutting down (ServerDeactivate).
/// Advances map generation to invalidate cached EDicts and clears player capabilities and menu sessions.
pub fn on_server_deactivate() {
    HostRuntime::on(HostEvent::ServerDeactivate);
    emit_event("server_deactivate", &[]);
}

/// Ticks the frame event in the WASM host.
pub fn on_server_frame() {
    HostRuntime::on(HostEvent::ServerFrame);
}
