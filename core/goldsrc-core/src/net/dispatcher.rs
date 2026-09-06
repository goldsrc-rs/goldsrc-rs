//! Network message dispatcher for packing and transmitting GoldSrc engine user messages.
//!
//! Encapsulates low-level byte-packing protocols for `TextMsg`, `SayText`, HUD messages,
//! and ensures strict adherence to GoldSrc buffer boundaries (185 bytes) and UTF-8 safety.

use goldsrc_api::consts::SAFE_SAYTEXT_LIMIT;
use goldsrc_api::{HUD_PRINTCENTER, HUD_PRINTCONSOLE, HUD_PRINTNOTIFY, MessageDest, PrintTarget};

/// Dispatcher responsible for formatting, chunking, and sending GoldSrc network user messages.
pub struct NetworkMessageDispatcher;

impl NetworkMessageDispatcher {
    /// Dispatches a formatted message to a player according to the target print channel.
    pub fn dispatch_player_print(
        engine: &dyn goldsrc_api::Engine,
        player_index: i32,
        target: PrintTarget,
        message: &str,
    ) {
        if !(1..=32).contains(&player_index) || !engine.entity_is_valid(player_index) {
            return;
        }

        match target {
            PrintTarget::Console | PrintTarget::Notify | PrintTarget::Center => {
                let (msg_dest, formatted) = match target {
                    PrintTarget::Console => (
                        HUD_PRINTCONSOLE,
                        if message.ends_with('\n') {
                            message.to_string()
                        } else {
                            format!("{message}\n")
                        },
                    ),
                    PrintTarget::Notify => {
                        (HUD_PRINTNOTIFY, goldsrc_api::format_notify_text(message))
                    }
                    PrintTarget::Center => {
                        (HUD_PRINTCENTER, goldsrc_api::format_center_text(message))
                    }
                    _ => unreachable!(),
                };

                Self::send_text_msg(engine, player_index, msg_dest, &formatted);
            }
            PrintTarget::Chat | PrintTarget::ColoredChat => {
                Self::send_say_text(engine, player_index, player_index, message);
            }
        }
    }

    /// Sends a `TextMsg` user message (console, notify, center text) to a single client.
    pub fn send_text_msg(
        engine: &dyn goldsrc_api::Engine,
        player_index: i32,
        msg_dest: i32,
        formatted: &str,
    ) {
        let text_msg_id = engine.reg_user_msg("TextMsg", -1);
        if text_msg_id > 0 && text_msg_id < 255 {
            let mut payload = formatted.to_string();
            // AMX Mod X protocol: if format string is used, double newline is needed for notify/console in cstrike
            if (msg_dest == HUD_PRINTNOTIFY || msg_dest == HUD_PRINTCONSOLE)
                && !payload.ends_with("\n\n")
            {
                payload.push('\n');
            }

            let safe_msg = if payload.len() > 185 {
                let mut end = 185;
                while end > 0 && !payload.is_char_boundary(end) {
                    end -= 1;
                }
                &payload[..end]
            } else {
                &payload
            };

            engine.message_begin(
                MessageDest::One as i32,
                text_msg_id,
                None,
                Some(player_index),
            );
            engine.write_byte(msg_dest);
            engine.write_string("%s");
            engine.write_string(safe_msg);
            engine.message_end();
        } else {
            // Fallback to direct client_print
            engine.client_print(player_index, msg_dest, formatted);
        }
    }

    /// Sends a `SayText` user message to a single client.
    pub fn send_say_text(
        engine: &dyn goldsrc_api::Engine,
        receiver_index: i32,
        sender_index: i32,
        message: &str,
    ) {
        let formatted = goldsrc_api::format_say_text(message);
        let say_text_id = engine.reg_user_msg("SayText", -1);
        if say_text_id > 0 && say_text_id < 255 {
            engine.message_begin(
                MessageDest::One as i32,
                say_text_id,
                None,
                Some(receiver_index),
            );
            // 1. Sender entity index for team color ^3 resolution
            engine.write_byte(sender_index);
            // 2. Chat message payload (starts with \x02 / \x01 in CS 1.6 client)
            let payload = if !formatted.starts_with(['\x01', '\x02', '\x03', '\x04']) {
                format!("\x01{formatted}")
            } else {
                formatted
            };
            let safe_msg = if payload.len() > SAFE_SAYTEXT_LIMIT {
                let mut end = SAFE_SAYTEXT_LIMIT;
                while end > 0 && !payload.is_char_boundary(end) {
                    end -= 1;
                }
                &payload[..end]
            } else {
                &payload
            };
            engine.write_string(safe_msg);
            engine.message_end();
        } else {
            // Fallback to HUD_PRINTCHAT via ClientPrintf if SayText user message isn't registered yet
            let safe_text = format!("{formatted}\n");
            engine.client_print(receiver_index, goldsrc_api::HUD_PRINTCHAT, &safe_text);
        }
    }

    /// Broadcasts a `TextMsg` to all connected clients (`MessageDest::All`).
    pub fn broadcast_text_msg(engine: &dyn goldsrc_api::Engine, msg_dest: i32, message: &str) {
        for idx in 1..=32 {
            if engine.entity_is_valid(idx) {
                Self::send_text_msg(engine, idx, msg_dest, message);
            }
        }
    }

    /// Broadcasts a `SayText` message to all connected clients (`MessageDest::All`).
    pub fn broadcast_say_text(engine: &dyn goldsrc_api::Engine, sender_index: i32, message: &str) {
        for idx in 1..=32 {
            if engine.entity_is_valid(idx) {
                Self::send_say_text(engine, idx, sender_index, message);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use goldsrc_api::{
        EngineConsole, EngineCvars, EngineEntities, EngineMessages, EnginePhysics, EnginePrecache,
        EngineSound, TraceResult,
    };
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockNetEngine {
        messages: Mutex<Vec<(i32, i32, Option<i32>)>>,
        bytes: Mutex<Vec<i32>>,
        strings: Mutex<Vec<String>>,
        ended: Mutex<usize>,
    }

    impl EnginePrecache for MockNetEngine {
        fn precache_model(&self, _s: &str) -> i32 {
            0
        }
        fn precache_sound(&self, _s: &str) -> i32 {
            0
        }
        fn precache_generic(&self, _s: &str) -> i32 {
            0
        }
    }

    impl EngineMessages for MockNetEngine {
        fn message_begin(&self, d: i32, t: i32, _o: Option<[f32; 3]>, e: Option<i32>) {
            self.messages.lock().unwrap().push((d, t, e));
        }
        fn message_end(&self) {
            *self.ended.lock().unwrap() += 1;
        }
        fn write_byte(&self, b: i32) {
            self.bytes.lock().unwrap().push(b);
        }
        fn write_char(&self, _c: i32) {}
        fn write_short(&self, _s: i32) {}
        fn write_long(&self, _l: i32) {}
        fn write_angle(&self, _a: f32) {}
        fn write_coord(&self, _c: f32) {}
        fn write_string(&self, s: &str) {
            self.strings.lock().unwrap().push(s.to_string());
        }
        fn write_entity(&self, _e: i32) {}
        fn reg_user_msg(&self, name: &str, _size: i32) -> i32 {
            match name {
                "TextMsg" => 64,
                "SayText" => 65,
                _ => -1,
            }
        }
    }

    impl EngineEntities for MockNetEngine {
        fn entity_is_valid(&self, index: i32) -> bool {
            (1..=32).contains(&index)
        }
        fn entity_classname(&self, _index: i32) -> Option<String> {
            None
        }
        fn entity_health(&self, _index: i32) -> f32 {
            100.0
        }
        fn entity_set_health(&self, _index: i32, _health: f32) {}
        fn entity_origin(&self, _index: i32) -> [f32; 3] {
            [0.0; 3]
        }
        fn entity_set_origin(&self, _index: i32, _pos: [f32; 3]) {}
        fn entity_velocity(&self, _index: i32) -> [f32; 3] {
            [0.0; 3]
        }
        fn entity_set_velocity(&self, _index: i32, _vel: [f32; 3]) {}
        fn entity_angles(&self, _index: i32) -> [f32; 3] {
            [0.0; 3]
        }
        fn entity_set_angles(&self, _index: i32, _angles: [f32; 3]) {}
        fn player_name(&self, _index: i32) -> Option<String> {
            Some("Player".into())
        }
        fn player_team(&self, _index: i32) -> i32 {
            1
        }
        fn player_lang(&self, _index: i32) -> Option<String> {
            Some("en".into())
        }
        fn player_armorvalue(&self, _index: i32) -> f32 {
            0.0
        }
        fn player_set_armorvalue(&self, _index: i32, _armor: f32) {}
        fn create_named_entity(&self, _classname: &str) -> Option<i32> {
            None
        }
        fn remove_entity(&self, _index: i32) {}
        fn drop_to_floor(&self, _index: i32) -> i32 {
            0
        }
        fn dispatch_spawn(&self, _index: i32) -> i32 {
            0
        }
        fn dispatch_touch(&self, _touched: i32, _other: i32) {}
    }

    impl EngineCvars for MockNetEngine {
        fn cvar_get_string(&self, _n: &str) -> Option<String> {
            None
        }
        fn cvar_set_string(&self, _n: &str, _v: &str) {}
        fn cvar_get_float(&self, _n: &str) -> f32 {
            0.0
        }
        fn cvar_set_float(&self, _n: &str, _v: f32) {}
    }

    impl EngineConsole for MockNetEngine {
        fn server_command(&self, _cmd: &str) {}
        fn server_print(&self, _msg: &str) {}
        fn client_print(&self, _client_index: i32, _dest: i32, _message: &str) {}
    }

    impl EngineSound for MockNetEngine {
        fn emit_sound(
            &self,
            _entity: i32,
            _channel: i32,
            _sample: &str,
            _volume: f32,
            _attenuation: f32,
            _flags: i32,
            _pitch: i32,
        ) {
        }
        fn emit_ambient_sound(
            &self,
            _entity: i32,
            _pos: [f32; 3],
            _sample: &str,
            _volume: f32,
            _attenuation: f32,
            _flags: i32,
            _pitch: i32,
        ) {
        }
    }

    impl EnginePhysics for MockNetEngine {
        fn trace_line(
            &self,
            _start: [f32; 3],
            _end: [f32; 3],
            _flags: i32,
            _skip_entity: i32,
        ) -> TraceResult {
            TraceResult {
                all_solid: false,
                start_solid: false,
                in_open: true,
                in_water: false,
                fraction: 1.0,
                end_pos: [0.0; 3],
                plane_normal: [0.0; 3],
                hit_entity: -1,
            }
        }
        fn trace_hull(
            &self,
            _start: [f32; 3],
            _end: [f32; 3],
            _flags: i32,
            _hull_number: i32,
            _skip_entity: i32,
        ) -> TraceResult {
            TraceResult {
                all_solid: false,
                start_solid: false,
                in_open: true,
                in_water: false,
                fraction: 1.0,
                end_pos: [0.0; 3],
                plane_normal: [0.0; 3],
                hit_entity: -1,
            }
        }
        fn point_contents(&self, _point: [f32; 3]) -> i32 {
            0
        }
    }

    #[test]
    fn test_dispatcher_send_text_msg() {
        let engine = MockNetEngine::default();
        NetworkMessageDispatcher::send_text_msg(&engine, 1, HUD_PRINTCENTER, "Round Started!");

        let msgs = engine.messages.lock().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0], (MessageDest::One as i32, 64, Some(1)));

        let bytes = engine.bytes.lock().unwrap();
        assert_eq!(bytes[0], HUD_PRINTCENTER);

        let strings = engine.strings.lock().unwrap();
        assert_eq!(strings[0], "%s");
        assert_eq!(strings[1], "Round Started!");
        assert_eq!(*engine.ended.lock().unwrap(), 1);
    }

    #[test]
    fn test_dispatcher_send_say_text() {
        let engine = MockNetEngine::default();
        NetworkMessageDispatcher::send_say_text(&engine, 2, 1, "Hello from team!");

        let msgs = engine.messages.lock().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0], (MessageDest::One as i32, 65, Some(2)));

        let bytes = engine.bytes.lock().unwrap();
        assert_eq!(bytes[0], 1); // sender index

        let strings = engine.strings.lock().unwrap();
        assert_eq!(strings[0], "\x01Hello from team!");
        assert_eq!(*engine.ended.lock().unwrap(), 1);
    }

    #[test]
    fn test_dispatcher_player_print() {
        let engine = MockNetEngine::default();
        NetworkMessageDispatcher::dispatch_player_print(
            &engine,
            3,
            PrintTarget::Notify,
            "Notice message",
        );
        let msgs = engine.messages.lock().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0], (MessageDest::One as i32, 64, Some(3)));
        assert_eq!(*engine.ended.lock().unwrap(), 1);
    }
}
