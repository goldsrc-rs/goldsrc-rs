//! Engine network message operations.

/// Network message destination flags.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDest {
    /// Broadcast to all players on the server.
    Broadcast = 0,
    /// Send to all players reliably.
    All = 1,
    /// Send to a single player reliably.
    One = 2,
    /// Send to players near an origin (unreliable).
    Pvs = 3,
    /// Send to players in the PVS reliably.
    Pas = 4,
    /// Send to players in PVS (reliable).
    PvsReliable = 5,
    /// Send to players in PAS (reliable).
    PasReliable = 6,
    /// Send to a single player unreliably.
    OneUnreliable = 7,
}

/// Network message writer interface.
pub trait EngineMessages: Send + Sync {
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

    /// Print a message to the server console.
    fn server_print(&self, message: &str);

    /// Execute a server command string.
    fn server_command(&self, command: &str);
}
