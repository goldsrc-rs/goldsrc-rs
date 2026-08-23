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

#[cfg(test)]
mod tests {
    use super::*;

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
}
