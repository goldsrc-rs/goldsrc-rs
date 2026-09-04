//! Centralized event and command dispatcher for backends and WASM plugins.

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
    if name == "client_disconnect" {
        goldsrc_api::auth::Auth::remove_player(index);
        goldsrc_api::menu::close_player_menu(index);
        goldsrc_host_wasm::clear_active_menu_owner(index);
        if let Ok(mut mgr) = crate::menu::menu_manager().lock() {
            mgr.on_disconnect(index);
        }
    }
    let res = emit_event(name, &index.to_le_bytes());

    if name == "client_connect" || name == "client_disconnect" {
        let player_count = goldsrc_api::auth::Auth::total_players();
        let current_map = HostRuntime::current_map();
        HostRuntime::evaluate_rules(&current_map, player_count);
    }

    res
}

/// Dispatches client userinfo change event and updates active player menu if open.
pub fn on_client_user_info_changed(player_idx: i32) {
    emit_player_event("client_user_info_changed", player_idx);
    goldsrc_api::menu::refresh_player_menu(player_idx);
}

/// Dispatches a console / client command to the WASM host.
/// Returns `true` if the host runtime is active and processed the command.
pub fn dispatch_command(cmd: &str, args: &str) -> bool {
    HostRuntime::with_manager(|m| match m {
        Some(manager) => manager.dispatch_command(cmd, 0, args),
        None => {
            log::trace!(target: "core", "dispatch_command('{cmd}') skipped: WASM host not initialized");
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

        // Dispatch raw slot to WASM plugins event "menu_select" (8 bytes payload: [player_idx: i32, slot: u32])
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
            let mut parts = text.split_whitespace();
            if let Some(trigger) = parts.next() {
                let clean_trigger = trigger.trim_start_matches(['/', '!']);
                let rest_args = parts.collect::<Vec<_>>().join(" ");

                // 1. Try exact clean trigger (e.g. "vip" or "vipmenu")
                if manager.dispatch_command(clean_trigger, player_idx, &rest_args) {
                    return true;
                }
                // 2. Try raw trigger (e.g. "/vip")
                if manager.dispatch_command(trigger, player_idx, &rest_args) {
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
    emit_event("server_activate", &[]);
    if let Some(engine) = HostRuntime::engine() {
        let map_name = engine.cvar_get_string("mapname").unwrap_or_default();
        let player_count = goldsrc_api::auth::Auth::total_players();
        HostRuntime::evaluate_rules(&map_name, player_count);
    }
}

/// Invoked when the current server map is ending or server shutting down (ServerDeactivate).
/// Advances map generation to invalidate cached EDicts and clears player capabilities and menu sessions.
pub fn on_server_deactivate() {
    goldsrc_api::edict::bump_map_generation();
    goldsrc_api::auth::Auth::clear_all_players();
    goldsrc_api::menu::clear_all_menus();
    goldsrc_host_wasm::clear_all_active_menu_owners();
    if let Ok(mut mgr) = crate::menu::menu_manager().lock() {
        mgr.on_map_change();
    }
    HostRuntime::on_map_change();
    emit_event("server_deactivate", &[]);
}

/// Ticks the frame event in the WASM host.
pub fn on_server_frame() {
    HostRuntime::on_server_frame();
}
