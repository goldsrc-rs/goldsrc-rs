//! Wasmtime HostState and api::Host implementation.
//!
//! Provides the host-function environment exposed to WASM plugins:
//! entity manipulation, cvars, networking messages, HUD, capabilities, and storage sandbox.

use crate::bindings::goldsrc::engine::api;
use goldsrc_api::Engine as GoldsrcEngine;
use std::sync::Arc;

/// Wasmtime store state exposed to WASM plugins via host functions.
pub struct HostState {
    /// Engine bridge for real game-state access.
    pub engine: Arc<dyn GoldsrcEngine>,
    /// Per-store memory and table limit enforcement.
    pub limits: wasmtime::StoreLimits,
    /// Identifier of the calling plugin.
    pub plugin_name: String,
    /// Explicitly allowed shared storage buckets from metadata.
    pub shared_buckets: Vec<String>,
}

impl HostState {
    /// Resolves bucket name enforcing plugin isolation and allowlist sharing.
    pub fn resolve_bucket(&self, bucket: &str) -> Option<String> {
        if bucket.contains('/') {
            // Check if bucket is explicitly allowlisted
            if self.shared_buckets.iter().any(|b| b == bucket) {
                Some(bucket.to_string())
            } else {
                crate::host_log(&format!(
                    "[ERROR] Plugin '{}' attempted unauthorized access to shared bucket '{}'",
                    self.plugin_name, bucket
                ));
                None
            }
        } else {
            // Auto-prefix with plugin name
            Some(format!("{}/{}", self.plugin_name, bucket))
        }
    }
}

impl api::Host for HostState {
    fn host_log(&mut self, msg: String) {
        crate::host_log(&msg);
    }

    fn host_entity_is_valid(&mut self, index: i32) -> bool {
        self.engine.entity_is_valid(index)
    }
    fn host_entity_classname(&mut self, index: i32) -> Option<String> {
        self.engine.entity_classname(index)
    }
    fn host_entity_health(&mut self, index: i32) -> f32 {
        self.engine.entity_health(index)
    }
    fn host_entity_set_health(&mut self, index: i32, health: f32) {
        self.engine.entity_set_health(index, health);
    }
    fn host_entity_origin(&mut self, index: i32) -> api::Vector3 {
        let [x, y, z] = self.engine.entity_origin(index);
        api::Vector3 { x, y, z }
    }
    fn host_entity_set_origin(&mut self, index: i32, pos: api::Vector3) {
        self.engine.entity_set_origin(index, [pos.x, pos.y, pos.z]);
    }
    fn host_entity_velocity(&mut self, index: i32) -> api::Vector3 {
        let [x, y, z] = self.engine.entity_velocity(index);
        api::Vector3 { x, y, z }
    }
    fn host_entity_set_velocity(&mut self, index: i32, vel: api::Vector3) {
        self.engine
            .entity_set_velocity(index, [vel.x, vel.y, vel.z]);
    }
    fn host_entity_angles(&mut self, index: i32) -> api::Vector3 {
        let [x, y, z] = self.engine.entity_angles(index);
        api::Vector3 { x, y, z }
    }
    fn host_entity_set_angles(&mut self, index: i32, angles: api::Vector3) {
        self.engine
            .entity_set_angles(index, [angles.x, angles.y, angles.z]);
    }
    fn host_create_named_entity(&mut self, classname: String) -> Option<i32> {
        self.engine.create_named_entity(&classname)
    }
    fn host_remove_entity(&mut self, index: i32) {
        self.engine.remove_entity(index);
    }
    fn host_drop_to_floor(&mut self, index: i32) -> i32 {
        self.engine.drop_to_floor(index)
    }

    fn host_player_name(&mut self, index: i32) -> Option<String> {
        self.engine.player_name(index)
    }
    fn host_player_armorvalue(&mut self, index: i32) -> f32 {
        self.engine.player_armorvalue(index)
    }
    fn host_player_set_armorvalue(&mut self, index: i32, armor: f32) {
        self.engine.player_set_armorvalue(index, armor);
    }

    fn host_cvar_get_float(&mut self, name: String) -> f32 {
        self.engine.cvar_get_float(&name)
    }
    fn host_cvar_set_float(&mut self, name: String, val: f32) {
        self.engine.cvar_set_float(&name, val);
    }
    fn host_cvar_get_string(&mut self, name: String) -> Option<String> {
        self.engine.cvar_get_string(&name)
    }
    fn host_cvar_set_string(&mut self, name: String, val: String) {
        self.engine.cvar_set_string(&name, &val);
    }

    fn host_precache_model(&mut self, path: String) -> i32 {
        self.engine.precache_model(&path)
    }
    fn host_precache_sound(&mut self, path: String) -> i32 {
        self.engine.precache_sound(&path)
    }
    fn host_precache_generic(&mut self, path: String) -> i32 {
        self.engine.precache_generic(&path)
    }

    fn host_emit_sound(
        &mut self,
        entity: i32,
        channel: i32,
        sample: String,
        volume: f32,
        attenuation: f32,
        sound_flags: i32,
        pitch: i32,
    ) {
        self.engine.emit_sound(
            entity,
            channel,
            &sample,
            volume,
            attenuation,
            sound_flags,
            pitch,
        );
    }

    fn host_print_chat(&mut self, player_index: i32, message: String) {
        if !(1..=32).contains(&player_index) || !self.engine.entity_is_valid(player_index) {
            self.engine
                .server_print(&format!("[Chat to #{player_index}] {message}\n"));
            return;
        }
        let formatted = goldsrc_api::format_say_text(&message);
        let say_text_id = self.engine.reg_user_msg("SayText", -1);
        let msg_id = if say_text_id <= 0 { 76 } else { say_text_id };
        self.engine.message_begin(
            goldsrc_api::MessageDest::One as i32,
            msg_id,
            None,
            Some(player_index),
        );
        // In GoldSrc CS 1.6 SayText, first byte is the sender entity index (1..32 for player colors, or 0)
        self.engine.write_byte(player_index);
        // Truncate message if oversized to prevent buffer overflow (SayText payload max 192 bytes)
        let safe_msg = if formatted.len() > 175 {
            let mut end = 175;
            while end > 0 && !formatted.is_char_boundary(end) {
                end -= 1;
            }
            &formatted[..end]
        } else {
            &formatted
        };
        // SayText string must be sent without extra trailing newline
        self.engine.write_string(safe_msg);
        self.engine.message_end();
    }

    fn host_print_center(&mut self, player_index: i32, message: String) {
        if player_index < 0 {
            self.engine.server_print(&format!("[Center] {message}\n"));
            return;
        }

        let formatted = goldsrc_api::format_center_text(&message);
        let text_msg_id = self.engine.reg_user_msg("TextMsg", -1);
        let msg_id = if text_msg_id <= 0 { 75 } else { text_msg_id };

        let dest = if player_index == 0 {
            goldsrc_api::MessageDest::All as i32
        } else {
            if !(1..=32).contains(&player_index) || !self.engine.entity_is_valid(player_index) {
                return;
            }
            goldsrc_api::MessageDest::One as i32
        };

        let target_edict = if player_index == 0 {
            None
        } else {
            Some(player_index)
        };

        self.engine.message_begin(dest, msg_id, None, target_edict);

        // AMX Mod X / HLSDK UTIL_ClientPrint protocol for center messages:
        // 1. Write destination byte: HUD_PRINTCENTER (4)
        // 2. Write format string: "%s"
        // 3. Write formatted message (newlines replaced with '\r', safe truncated to <= 185 bytes)
        self.engine.write_byte(goldsrc_api::HUD_PRINTCENTER);
        self.engine.write_string("%s");

        let safe_msg = if formatted.len() > 185 {
            let mut end = 185;
            while end > 0 && !formatted.is_char_boundary(end) {
                end -= 1;
            }
            &formatted[..end]
        } else {
            &formatted
        };

        self.engine.write_string(safe_msg);
        self.engine.message_end();
    }

    fn host_print_console(&mut self, player_index: i32, message: String) {
        if player_index <= 0 || !self.engine.entity_is_valid(player_index) {
            self.engine
                .server_print(&format!("[Console#{player_index}] {message}\n"));
            return;
        }
        // 0 = PRINT_CONSOLE in GoldSrc client_printf
        self.engine
            .client_print(player_index, goldsrc_api::PRINT_CONSOLE, &message);
    }

    fn host_dispatch_spawn(&mut self, index: i32) -> i32 {
        self.engine.dispatch_spawn(index)
    }

    fn host_dispatch_touch(&mut self, touched: i32, other: i32) {
        self.engine.dispatch_touch(touched, other);
    }

    fn host_show_menu(&mut self, player_index: i32, keys_mask: i32, timeout: i32, text: String) {
        if player_index <= 0 {
            return;
        }
        let show_menu_id = self.engine.reg_user_msg("ShowMenu", -1);
        if show_menu_id <= 0 || show_menu_id == 255 {
            return;
        }

        crate::notify_show_menu(player_index, keys_mask, timeout, &text);

        if text.is_empty() {
            self.engine.message_begin(
                goldsrc_api::MessageDest::One as i32,
                show_menu_id,
                None,
                Some(player_index),
            );
            self.engine.write_short(keys_mask);
            self.engine.write_char(timeout);
            self.engine.write_byte(0);
            self.engine.write_string("");
            self.engine.message_end();
            return;
        }

        let max_chunk = goldsrc_api::consts::MAX_SHOW_MENU_CHUNK_SIZE;
        let mut remaining = &text[..];

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

            self.engine.message_begin(
                goldsrc_api::MessageDest::One as i32,
                show_menu_id,
                None,
                Some(player_index),
            );
            self.engine.write_short(keys_mask);
            self.engine.write_char(timeout);
            self.engine.write_byte(if has_more { 1 } else { 0 });
            self.engine.write_string(chunk);
            self.engine.message_end();
        }
    }

    fn host_send_hud_message(
        &mut self,
        _player_index: i32,
        channel: i32,
        x: f32,
        y: f32,
        r: i32,
        g: i32,
        b: i32,
        a: i32,
        effect: i32,
        fade_in: f32,
        fade_out: f32,
        hold_time: f32,
        text: String,
    ) {
        let x_val = (if x < 0.0 { -1.0 } else { x } * 8192.0) as i32;
        let y_val = (if y < 0.0 { -1.0 } else { y } * 8192.0) as i32;

        self.engine.message_begin(
            goldsrc_api::MessageDest::Broadcast as i32,
            goldsrc_api::consts::SVC_TEMPENTITY,
            None,
            None,
        );
        self.engine
            .write_byte(goldsrc_api::consts::TE_TEXTMESSAGE as i32);
        self.engine.write_byte(channel.clamp(1, 4));
        self.engine.write_short(x_val);
        self.engine.write_short(y_val);
        self.engine.write_byte(effect.clamp(0, 2));
        self.engine.write_byte(r.clamp(0, 255));
        self.engine.write_byte(g.clamp(0, 255));
        self.engine.write_byte(b.clamp(0, 255));
        self.engine.write_byte(a.clamp(0, 255));
        self.engine.write_byte(r.clamp(0, 255)); // 2nd color fallback
        self.engine.write_byte(g.clamp(0, 255));
        self.engine.write_byte(b.clamp(0, 255));
        self.engine.write_byte(a.clamp(0, 255));
        self.engine.write_short((fade_in * 256.0) as i32);
        self.engine.write_short((fade_out * 256.0) as i32);
        self.engine.write_short((hold_time * 256.0) as i32);
        if effect.clamp(0, 2) == 2 {
            self.engine.write_short(0); // fx_time placeholder
        }
        self.engine.write_string(&text);
        self.engine.message_end();
    }

    fn host_send_dhud_message(
        &mut self,
        player_index: i32,
        x: f32,
        y: f32,
        r: i32,
        g: i32,
        b: i32,
        _a: i32,
        effect: i32,
        fade_in: f32,
        fade_out: f32,
        hold_time: f32,
        text: String,
    ) {
        const SVC_DIRECTOR: i32 = 51;
        const DRC_CMD_MESSAGE: i32 = 6;

        let (dest, target_idx) = if player_index <= 0 {
            (goldsrc_api::MessageDest::Broadcast as i32, None)
        } else {
            (goldsrc_api::MessageDest::One as i32, Some(player_index))
        };

        let text_bytes = text.as_bytes();
        let len = text_bytes.len().min(128);
        let safe_text = &text[..len];

        // Pack color into 0x00RRGGBB format expected by client VGUI director parser
        let packed_color = b.clamp(0, 255) | (g.clamp(0, 255) << 8) | (r.clamp(0, 255) << 16);

        self.engine
            .message_begin(dest, SVC_DIRECTOR, None, target_idx);
        self.engine.write_byte((len as i32) + 31);
        self.engine.write_byte(DRC_CMD_MESSAGE);
        self.engine.write_byte(effect.clamp(0, 2));
        self.engine.write_long(packed_color);
        self.engine.write_long(x.to_bits() as i32);
        self.engine.write_long(y.to_bits() as i32);
        self.engine.write_long(fade_in.to_bits() as i32);
        self.engine.write_long(fade_out.to_bits() as i32);
        self.engine.write_long(hold_time.to_bits() as i32);
        self.engine.write_long(0); // fx_time
        self.engine.write_string(safe_text);
        self.engine.message_end();
    }

    fn host_register_capability(&mut self, name: String, description: String) -> bool {
        if name.is_empty() || name.len() > 256 || description.len() > 4096 {
            return false;
        }
        goldsrc_api::auth::Auth::register_capability(&name, &description)
    }

    fn host_has_capability(&mut self, player_index: i32, name: String) -> bool {
        goldsrc_api::auth::Auth::has_capability(player_index, &name)
    }

    fn host_grant_capability(&mut self, player_index: i32, name: String) -> bool {
        goldsrc_api::auth::Auth::grant_capability(player_index, &name)
    }

    fn host_revoke_capability(&mut self, player_index: i32, name: String) -> bool {
        goldsrc_api::auth::Auth::revoke_capability(player_index, &name)
    }

    fn host_storage_get(&mut self, bucket: String, key: String) -> Option<Vec<u8>> {
        let resolved = self.resolve_bucket(&bucket)?;
        if let Ok(lock) = crate::STORAGE_GET_CB.read() {
            if let Some(cb) = *lock {
                return cb(&resolved, &key);
            }
        }
        None
    }

    fn host_storage_set(&mut self, bucket: String, key: String, val: Vec<u8>) -> bool {
        let Some(resolved) = self.resolve_bucket(&bucket) else {
            return false;
        };
        if let Ok(lock) = crate::STORAGE_SET_CB.read() {
            if let Some(cb) = *lock {
                return cb(&resolved, &key, &val);
            }
        }
        false
    }

    fn host_storage_delete(&mut self, bucket: String, key: String) -> bool {
        let Some(resolved) = self.resolve_bucket(&bucket) else {
            return false;
        };
        if let Ok(lock) = crate::STORAGE_DELETE_CB.read() {
            if let Some(cb) = *lock {
                return cb(&resolved, &key);
            }
        }
        false
    }

    fn host_storage_fetch_add(&mut self, bucket: String, key: String, delta: i64) -> i64 {
        let Some(resolved) = self.resolve_bucket(&bucket) else {
            return 0;
        };
        if let Ok(lock) = crate::STORAGE_FETCH_ADD_CB.read() {
            if let Some(cb) = *lock {
                return cb(&resolved, &key, delta);
            }
        }
        0
    }

    fn host_translate(&mut self, dict: String, lang: String, key: String) -> String {
        if let Ok(lock) = crate::TRANSLATE_CB.read() {
            if let Some(cb) = *lock {
                return cb(&dict, &lang, &key);
            }
        }
        key
    }
}
