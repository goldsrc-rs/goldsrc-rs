//! Command execution error pipeline, result types, and invocation context.

use crate::client::Player;
use crate::command::CommandTarget;

/// Taxonomy of errors that can occur during command parsing, guard checks, or execution.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    /// The caller lacks the required capability/permission.
    AccessDenied {
        capability: String,
        custom_message: Option<String>,
    },
    /// Invalid arguments supplied to the command.
    InvalidArguments {
        usage: &'static str,
        param_name: &'static str,
        reason: String,
    },
    /// The caller's life or team state does not satisfy typestate requirements (e.g. dead player).
    InvalidState { expected: &'static str },
    /// A targeted player was not found on the server.
    TargetNotFound { query: String },
    /// The command is currently on cooldown for the caller.
    Cooldown { remaining_seconds: f32 },
    /// Generic plugin-specific error message.
    Custom(String),
    /// Silently abort execution without sending a message to the caller.
    Silent,
}

impl CommandError {
    /// Construct a custom user error.
    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }

    /// Construct an access denied error.
    pub fn access_denied(cap: impl Into<String>) -> Self {
        Self::AccessDenied {
            capability: cap.into(),
            custom_message: None,
        }
    }

    /// Construct an access denied error with custom feedback message.
    pub fn access_denied_with_msg(cap: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::AccessDenied {
            capability: cap.into(),
            custom_message: Some(msg.into()),
        }
    }

    /// Construct an invalid argument error.
    pub fn invalid_args(
        usage: &'static str,
        param_name: &'static str,
        reason: impl Into<String>,
    ) -> Self {
        Self::InvalidArguments {
            usage,
            param_name,
            reason: reason.into(),
        }
    }

    /// Construct a target not found error.
    pub fn target_not_found(query: impl Into<String>) -> Self {
        Self::TargetNotFound {
            query: query.into(),
        }
    }

    /// Construct an invalid player state error.
    pub fn invalid_state(expected: &'static str) -> Self {
        Self::InvalidState { expected }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccessDenied {
                capability,
                custom_message,
            } => {
                if let Some(msg) = custom_message {
                    write!(f, "{msg}")
                } else {
                    write!(f, "Access denied: missing capability '{capability}'")
                }
            }
            Self::InvalidArguments {
                usage,
                param_name,
                reason,
            } => {
                write!(
                    f,
                    "Invalid parameter '{param_name}': {reason}. Usage: {usage}"
                )
            }
            Self::InvalidState { expected } => {
                write!(f, "Invalid player state (expected: {expected})")
            }
            Self::TargetNotFound { query } => {
                write!(f, "Player '{query}' not found")
            }
            Self::Cooldown { remaining_seconds } => {
                write!(
                    f,
                    "Command is on cooldown ({remaining_seconds:.1}s remaining)"
                )
            }
            Self::Custom(msg) => write!(f, "{msg}"),
            Self::Silent => Ok(()),
        }
    }
}

impl std::error::Error for CommandError {}

/// Standard result returned by command handlers.
pub type CommandResult = Result<(), CommandError>;

/// Invocation context provided to command handlers.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// The player invoking the command, or `None` if invoked from server console.
    pub player: Option<Player>,
    /// Ingress channel through which the command was invoked.
    pub target: CommandTarget,
    /// The command name used (or alias).
    pub command_name: String,
    /// Raw unparsed arguments string.
    pub raw_args: String,
    /// Tokenized argument list.
    pub args: Vec<String>,
}

impl CommandContext {
    /// Create a new invocation context.
    pub fn new(
        player: Option<Player>,
        target: CommandTarget,
        command_name: impl Into<String>,
        raw_args: impl Into<String>,
    ) -> Self {
        let raw = raw_args.into();
        let args = raw.split_whitespace().map(|s| s.to_string()).collect();
        Self {
            player,
            target,
            command_name: command_name.into(),
            raw_args: raw,
            args,
        }
    }

    /// Returns `true` if invoked from the server console.
    pub fn is_server(&self) -> bool {
        self.player.is_none()
    }

    fn print_server(message: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_log(message);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("{message}");
        }
    }

    /// Send a formatted reply to the caller (chat if invoked from chat, console otherwise).
    pub fn reply(&self, message: &str) {
        if let Some(player) = &self.player {
            match self.target {
                CommandTarget::Chat { .. } => player.print_chat(message),
                _ => {
                    player.print_chat(message);
                }
            }
        } else {
            Self::print_server(message);
        }
    }

    /// Explicitly send a reply to the caller's in-game chat.
    pub fn reply_chat(&self, message: &str) {
        if let Some(player) = &self.player {
            player.print_chat(message);
        } else {
            Self::print_server(message);
        }
    }

    /// Explicitly send a reply to the server console.
    pub fn reply_console(&self, message: &str) {
        Self::print_server(message);
    }
}
