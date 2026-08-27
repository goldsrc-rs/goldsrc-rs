//! Engine network message operations.

/// Network message destination flags matching GoldSrc const.h.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDest {
    /// Broadcast to all players on the server (unreliable).
    Broadcast = 0,
    /// Send to a single player reliably.
    One = 1,
    /// Send to all players reliably.
    All = 2,
    /// Signon initialization messages.
    Init = 3,
    /// Send to players near an origin in PVS (unreliable).
    Pvs = 4,
    /// Send to players in PAS (unreliable).
    Pas = 5,
    /// Send to players in PVS (reliable).
    PvsReliable = 6,
    /// Send to players in PAS (reliable).
    PasReliable = 7,
    /// Send to a single player unreliably.
    OneUnreliable = 8,
    /// Send to spectator proxy.
    Spec = 9,
}

/// Network message writer interface.
pub trait EngineMessages: Send + Sync {
    /// Register or look up a user message by name.
    fn reg_user_msg(&self, name: &str, size: i32) -> i32;

    /// Begin constructing a network message.
    fn message_begin(
        &self,
        msg_dest: i32,
        msg_type: i32,
        origin: Option<[f32; 3]>,
        edict_index: Option<i32>,
    );

    /// Finalize and dispatch the active network message.
    fn message_end(&self);

    /// Write an 8-bit unsigned byte.
    fn write_byte(&self, val: i32);

    /// Write an 8-bit signed char.
    fn write_char(&self, val: i32);

    /// Write a 16-bit signed short.
    fn write_short(&self, val: i32);

    /// Write a 32-bit signed long.
    fn write_long(&self, val: i32);

    /// Write an angle float (compressed to 1 byte).
    fn write_angle(&self, val: f32);

    /// Write a world coordinate float.
    fn write_coord(&self, val: f32);

    /// Write a null-terminated UTF-8 string.
    fn write_string(&self, val: &str);

    /// Write an entity index.
    fn write_entity(&self, val: i32);
}

/// Type-safe builder for GoldSrc network messages.
///
/// Wraps the raw `message_begin` / `write_*` / `message_end` sequence and
/// guarantees `message_end` is always emitted: either explicitly via
/// [`MessageBuilder::send`] or automatically on drop (RAII safety net),
/// so a forgotten `send()` can never leave the engine's network buffer
/// in a dangling open state (`svc_bad` on clients).
///
/// # Examples
///
/// ```no_run
/// # use goldsrc_api::{EngineMessages, MessageDest};
/// # fn demo(engine: &dyn EngineMessages, say_text_id: i32, player: i32) {
/// use goldsrc_api::engine::MessageBuilder;
///
/// MessageBuilder::to_player(engine, MessageDest::One, say_text_id, player)
///     .byte(0)               // sender: server console
///     .string("Hello!")
///     .send();
/// # }
/// ```
pub struct MessageBuilder<'a> {
    engine: &'a dyn EngineMessages,
    /// `true` while the engine-side message is still open.
    open: bool,
}

impl<'a> MessageBuilder<'a> {
    /// Begins a broadcast message to all players (unreliable).
    pub fn broadcast(engine: &'a dyn EngineMessages, msg_type: i32) -> Self {
        Self::begin(engine, MessageDest::Broadcast, msg_type, None, None)
    }

    /// Begins a reliable message sent to a single player by entity index.
    pub fn to_player(
        engine: &'a dyn EngineMessages,
        dest: MessageDest,
        msg_type: i32,
        player_index: i32,
    ) -> Self {
        Self::begin(engine, dest, msg_type, None, Some(player_index))
    }

    /// Begins a message with full control over destination, origin, and target.
    pub fn begin(
        engine: &'a dyn EngineMessages,
        dest: MessageDest,
        msg_type: i32,
        origin: Option<[f32; 3]>,
        edict_index: Option<i32>,
    ) -> Self {
        engine.message_begin(dest as i32, msg_type, origin, edict_index);
        Self { engine, open: true }
    }

    /// Writes an 8-bit unsigned byte.
    pub fn byte(self, val: i32) -> Self {
        self.engine.write_byte(val);
        self
    }

    /// Writes an 8-bit signed char.
    pub fn char(self, val: i32) -> Self {
        self.engine.write_char(val);
        self
    }

    /// Writes a 16-bit signed short.
    pub fn short(self, val: i32) -> Self {
        self.engine.write_short(val);
        self
    }

    /// Writes a 32-bit signed long.
    pub fn long(self, val: i32) -> Self {
        self.engine.write_long(val);
        self
    }

    /// Writes an angle float (compressed to 1 byte).
    pub fn angle(self, val: f32) -> Self {
        self.engine.write_angle(val);
        self
    }

    /// Writes a world coordinate float.
    pub fn coord(self, val: f32) -> Self {
        self.engine.write_coord(val);
        self
    }

    /// Writes a null-terminated UTF-8 string.
    pub fn string(self, val: &str) -> Self {
        self.engine.write_string(val);
        self
    }

    /// Writes an entity index.
    pub fn entity(self, val: i32) -> Self {
        self.engine.write_entity(val);
        self
    }

    /// Finalizes and dispatches the message.
    pub fn send(mut self) {
        self.engine.message_end();
        self.open = false;
    }
}

impl Drop for MessageBuilder<'_> {
    fn drop(&mut self) {
        if self.open {
            self.engine.message_end();
            self.open = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Records the sequence of message operations for assertions.
    #[derive(Default)]
    struct MockMessages {
        ops: std::sync::Mutex<Vec<String>>,
    }

    impl EngineMessages for MockMessages {
        fn reg_user_msg(&self, _name: &str, _size: i32) -> i32 {
            42
        }
        fn message_begin(
            &self,
            msg_dest: i32,
            msg_type: i32,
            _origin: Option<[f32; 3]>,
            edict_index: Option<i32>,
        ) {
            self.ops
                .lock()
                .unwrap()
                .push(format!("begin({msg_dest},{msg_type},{edict_index:?})"));
        }
        fn message_end(&self) {
            self.ops.lock().unwrap().push("end".into());
        }
        fn write_byte(&self, val: i32) {
            self.ops.lock().unwrap().push(format!("byte({val})"));
        }
        fn write_char(&self, val: i32) {
            self.ops.lock().unwrap().push(format!("char({val})"));
        }
        fn write_short(&self, val: i32) {
            self.ops.lock().unwrap().push(format!("short({val})"));
        }
        fn write_long(&self, val: i32) {
            self.ops.lock().unwrap().push(format!("long({val})"));
        }
        fn write_angle(&self, val: f32) {
            self.ops.lock().unwrap().push(format!("angle({val})"));
        }
        fn write_coord(&self, val: f32) {
            self.ops.lock().unwrap().push(format!("coord({val})"));
        }
        fn write_string(&self, val: &str) {
            self.ops.lock().unwrap().push(format!("string({val})"));
        }
        fn write_entity(&self, val: i32) {
            self.ops.lock().unwrap().push(format!("entity({val})"));
        }
    }

    #[test]
    fn test_message_dest_discriminants() {
        assert_eq!(MessageDest::Broadcast as i32, 0);
        assert_eq!(MessageDest::One as i32, 1);
        assert_eq!(MessageDest::All as i32, 2);
        assert_eq!(MessageDest::Init as i32, 3);
        assert_eq!(MessageDest::Pvs as i32, 4);
        assert_eq!(MessageDest::Pas as i32, 5);
        assert_eq!(MessageDest::PvsReliable as i32, 6);
        assert_eq!(MessageDest::PasReliable as i32, 7);
        assert_eq!(MessageDest::OneUnreliable as i32, 8);
        assert_eq!(MessageDest::Spec as i32, 9);
    }

    #[test]
    fn test_send_writes_in_order_and_closes() {
        let mock = MockMessages::default();
        MessageBuilder::to_player(&mock, MessageDest::One, 76, 3)
            .byte(0)
            .string("hi")
            .send();

        assert_eq!(
            *mock.ops.lock().unwrap(),
            vec![
                "begin(1,76,Some(3))".to_string(),
                "byte(0)".to_string(),
                "string(hi)".to_string(),
                "end".to_string(),
            ]
        );
    }

    #[test]
    fn test_drop_without_send_emits_message_end() {
        let mock = MockMessages::default();
        {
            MessageBuilder::broadcast(&mock, 23).byte(1).short(2);
            // No send() — dropped here.
        }

        assert_eq!(
            *mock.ops.lock().unwrap(),
            vec![
                "begin(0,23,None)".to_string(),
                "byte(1)".to_string(),
                "short(2)".to_string(),
                "end".to_string(),
            ]
        );
    }

    #[test]
    fn test_drop_after_send_does_not_double_end() {
        let mock = MockMessages::default();
        {
            MessageBuilder::to_player(&mock, MessageDest::All, 5, 1).send();
        }

        assert_eq!(
            mock.ops
                .lock()
                .unwrap()
                .iter()
                .filter(|op| *op == "end")
                .count(),
            1
        );
    }
}
