//! Engine server console I/O and command execution operations.

/// Operations for printing to the server console and executing engine commands.
pub trait EngineConsole: Send + Sync {
    /// Prints a message to the server console.
    fn server_print(&self, message: &str);

    /// Executes a server command string in the engine command buffer.
    fn server_command(&self, command: &str);
}
