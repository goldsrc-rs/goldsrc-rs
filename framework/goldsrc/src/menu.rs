//! Runtime Menu Session Manager, pagination router, and network renderers.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use goldsrc_api::engine::Engine;
use goldsrc_api::menu::{
    ExitBehavior, Menu, MenuContext, MenuRendererKind, RenderedMenuPage, SlotAction,
};

/// Active menu session for a single connected player.
pub struct PlayerMenuSession {
    pub menu: Menu,
    pub current_page: usize,
    pub history_stack: Vec<(Menu, usize)>,
    pub rendered_page: Option<RenderedMenuPage>,
    pub expiry_time: Option<f32>,
}

/// Global session manager handling interactive player menus.
pub struct MenuSessionManager {
    sessions: HashMap<i32, PlayerMenuSession>,
    round_number: u32,
    round_start_time: f32,
}

impl MenuSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            round_number: 1,
            round_start_time: 0.0,
        }
    }

    /// Opens a menu for a player, pushing any currently active menu to the history stack.
    pub fn open_menu(
        &mut self,
        player_idx: i32,
        new_menu: Menu,
        engine: &dyn Engine,
        current_time: f32,
    ) {
        // Check required capability
        if let Some(ref cap) = new_menu.required_capability
            && !goldsrc_api::auth::Auth::has_capability(player_idx, cap)
        {
            engine.client_print(
                player_idx,
                goldsrc_api::MessageDest::One as i32,
                &format!("[Menu] Access denied: requires capability '{cap}'.\n"),
            );
            return;
        }

        // Build history stack if an existing session exists
        let history = if let Some(existing) = self.sessions.remove(&player_idx) {
            let mut stack = existing.history_stack;
            stack.push((existing.menu, existing.current_page));
            stack
        } else {
            Vec::new()
        };

        let expiry = if new_menu.timeout_seconds > 0 {
            Some(current_time + new_menu.timeout_seconds as f32)
        } else {
            None
        };

        let mut session = PlayerMenuSession {
            menu: new_menu,
            current_page: 0,
            history_stack: history,
            rendered_page: None,
            expiry_time: expiry,
        };

        self.render_and_send(player_idx, &mut session, engine);
        self.sessions.insert(player_idx, session);
    }

    /// Dispatches a `menuselect <slot>` command (slot 1..=10).
    /// Returns `true` if handled (consumed).
    pub fn handle_menuselect(
        &mut self,
        player_idx: i32,
        slot: u8,
        engine: &dyn Engine,
        current_time: f32,
    ) -> bool {
        let Some(mut session) = self.sessions.remove(&player_idx) else {
            return false;
        };

        let Some(ref rendered) = session.rendered_page else {
            return false;
        };

        let action = match rendered.slots.get(&slot) {
            Some(act) => act.clone(),
            None => {
                // Invalid slot, re-insert session and keep menu
                self.sessions.insert(player_idx, session);
                return false;
            }
        };

        match action {
            SlotAction::Execute { id, action_name } => {
                // Close menu
                Self::clear_client_menu(player_idx, engine);

                // Dispatch to WASM hook / event
                let payload = id.to_le_bytes();
                crate::hooks::emit_event("menu_select", &payload);

                // Also trigger client command if action name is non-empty
                if !action_name.is_empty() {
                    crate::hooks::dispatch_client_command(player_idx, &action_name, "");
                }
                true
            }
            SlotAction::PrevPage => {
                if session.current_page > 0 {
                    session.current_page -= 1;
                }
                self.render_and_send(player_idx, &mut session, engine);
                self.sessions.insert(player_idx, session);
                true
            }
            SlotAction::NextPage => {
                session.current_page += 1;
                self.render_and_send(player_idx, &mut session, engine);
                self.sessions.insert(player_idx, session);
                true
            }
            SlotAction::Exit => {
                if session.menu.exit_behavior == ExitBehavior::PopParent
                    && let Some((parent_menu, parent_page)) = session.history_stack.pop()
                {
                    // Restore parent menu from stack
                    session.menu = parent_menu;
                    session.current_page = parent_page;
                    session.expiry_time = if session.menu.timeout_seconds > 0 {
                        Some(current_time + session.menu.timeout_seconds as f32)
                    } else {
                        None
                    };
                    self.render_and_send(player_idx, &mut session, engine);
                    self.sessions.insert(player_idx, session);
                } else {
                    // Close completely
                    Self::clear_client_menu(player_idx, engine);
                }
                true
            }
            SlotAction::DenyFeedback(deny_action) => {
                match deny_action {
                    goldsrc_api::menu::DenyAction::Feedback { message, sound } => {
                        if let Some(msg) = message {
                            engine.client_print(
                                player_idx,
                                goldsrc_api::MessageDest::One as i32,
                                &format!("{msg}\n"),
                            );
                        }
                        if let Some(snd) = sound {
                            engine.emit_sound(player_idx, 0, &snd, 1.0, 0.8, 0, 100);
                        }
                    }
                    goldsrc_api::menu::DenyAction::Custom(cb) => {
                        cb(player_idx);
                    }
                    _ => {}
                }
                // Keep session open
                self.sessions.insert(player_idx, session);
                true
            }
            SlotAction::Noop => {
                // Keep session open
                self.sessions.insert(player_idx, session);
                true
            }
        }
    }

    /// Clears the active menu for a player.
    pub fn close_menu(&mut self, player_idx: i32, engine: &dyn Engine) {
        if self.sessions.remove(&player_idx).is_some() {
            Self::clear_client_menu(player_idx, engine);
        }
    }

    /// Cleans up session when player disconnects.
    pub fn on_disconnect(&mut self, player_idx: i32) {
        self.sessions.remove(&player_idx);
    }

    /// Clears all sessions on map change or server shutdown.
    pub fn on_map_change(&mut self) {
        self.sessions.clear();
        self.round_number = 1;
        self.round_start_time = 0.0;
    }

    /// Updates round state for condition checking.
    pub fn on_round_start(&mut self, round: u32, current_time: f32) {
        self.round_number = round;
        self.round_start_time = current_time;
    }

    /// Frame tick checking timeouts.
    pub fn tick_frame(&mut self, current_time: f32, engine: &dyn Engine) {
        let mut expired = Vec::new();
        for (&player_idx, session) in &self.sessions {
            if let Some(expiry) = session.expiry_time
                && current_time >= expiry
            {
                expired.push(player_idx);
            }
        }

        for player_idx in expired {
            self.close_menu(player_idx, engine);
        }
    }

    fn render_and_send(
        &self,
        player_idx: i32,
        session: &mut PlayerMenuSession,
        engine: &dyn Engine,
    ) {
        let is_alive = engine.entity_health(player_idx) > 0.0;
        let elapsed = self.round_start_time;

        let ctx = MenuContext {
            player_index: player_idx,
            round_number: self.round_number,
            round_time_elapsed: elapsed,
            is_alive,
            players_count: 1, // Fallback
        };

        if let Some(rendered) = session.menu.render_page(&ctx, session.current_page) {
            match &rendered.renderer {
                MenuRendererKind::Text => {
                    // Send ShowMenu user message with multipart chunking (192 byte limit)
                    Self::send_show_menu_chunked(
                        engine,
                        player_idx,
                        rendered.keys_mask as i32,
                        rendered.timeout,
                        &rendered.text,
                    );
                }
                MenuRendererKind::Dhud {
                    position,
                    color,
                    effect,
                } => {
                    // 1. Send DHUD message with full text
                    let hud_msg = goldsrc_api::hud::HudMessage {
                        text: rendered.text.clone(),
                        kind: goldsrc_api::hud::HudKind::Dhud,
                        color: *color,
                        color2: *color,
                        position: *position,
                        effect: *effect,
                    };
                    crate::hud::send_hud_message(engine, Some(player_idx), &hud_msg);

                    // 2. Send invisible ShowMenu with active keys mask to enable slot keypresses (1..0)
                    Self::send_show_menu_chunked(
                        engine,
                        player_idx,
                        rendered.keys_mask as i32,
                        rendered.timeout,
                        "",
                    );
                }
            }
            session.rendered_page = Some(rendered);
        }
    }

    /// Sends a `ShowMenu` message chunked across multiple packets using the GoldSrc `multipart` flag
    /// to avoid exceeding the engine's 192-byte UserMessage buffer limit.
    pub fn send_show_menu_chunked(
        engine: &dyn Engine,
        player_idx: i32,
        keys_mask: i32,
        timeout: i32,
        text: &str,
    ) {
        let show_menu_id = engine.reg_user_msg("ShowMenu", -1);
        let msg_id = if show_menu_id <= 0 { 9 } else { show_menu_id };

        if text.is_empty() {
            engine.message_begin(
                goldsrc_api::MessageDest::One as i32,
                msg_id,
                None,
                Some(player_idx),
            );
            engine.write_short(keys_mask);
            engine.write_char(timeout);
            engine.write_byte(0); // multipart = 0
            engine.write_string("");
            engine.message_end();
            return;
        }

        // GoldSrc user message buffer limit is MAX_USER_MSG_DATA_LEN (192 bytes).
        // Overhead: 2 (short keys) + 1 (char time) + 1 (byte multipart) + 1 (null terminator) = 5 bytes.
        // Safe payload margin: MAX_SHOW_MENU_CHUNK_SIZE (150 bytes) per chunk.
        let max_chunk = goldsrc_api::consts::MAX_SHOW_MENU_CHUNK_SIZE;
        let mut remaining = text;

        while !remaining.is_empty() {
            let chunk_len = if remaining.len() <= max_chunk {
                remaining.len()
            } else {
                let mut end = max_chunk;
                while end > 0 && !remaining.is_char_boundary(end) {
                    end -= 1;
                }
                if end == 0 {
                    remaining.chars().next().map(|c| c.len_utf8()).unwrap_or(1)
                } else {
                    end
                }
            };

            let chunk = &remaining[..chunk_len];
            remaining = &remaining[chunk_len..];
            let has_more = !remaining.is_empty();

            engine.message_begin(
                goldsrc_api::MessageDest::One as i32,
                msg_id,
                None,
                Some(player_idx),
            );
            engine.write_short(keys_mask);
            engine.write_char(timeout);
            engine.write_byte(if has_more { 1 } else { 0 }); // 1 = append, 0 = finish
            engine.write_string(chunk);
            engine.message_end();
        }
    }

    fn clear_client_menu(player_idx: i32, engine: &dyn Engine) {
        let show_menu_id = engine.reg_user_msg("ShowMenu", -1);
        let msg_id = if show_menu_id <= 0 { 9 } else { show_menu_id };

        engine.message_begin(
            goldsrc_api::MessageDest::One as i32,
            msg_id,
            None,
            Some(player_idx),
        );
        engine.write_short(0); // keys = 0 closes the menu
        engine.write_char(0);
        engine.write_byte(0);
        engine.write_string("");
        engine.message_end();
    }
}

impl Default for MenuSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

static G_MENU_MANAGER: OnceLock<Mutex<MenuSessionManager>> = OnceLock::new();

pub fn menu_manager() -> &'static Mutex<MenuSessionManager> {
    G_MENU_MANAGER.get_or_init(|| Mutex::new(MenuSessionManager::new()))
}
