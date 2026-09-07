//! In-memory runtime command registry and invocation dispatcher.

use crate::auth::CapExpr;
use crate::command::Command;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

/// Type alias for dynamic command execution handlers.
pub type CommandHandler = Arc<dyn Fn(i32, &str) -> bool + Send + Sync + 'static>;

/// Registered command entry containing the descriptor and invocation handler.
#[derive(Clone)]
pub struct RegisteredCommand {
    pub descriptor: Command,
    pub handler: CommandHandler,
    pub parsed_cap: Option<CapExpr>,
}

/// Thread-safe in-memory command registry for dynamic command routing.
#[derive(Default)]
pub struct CommandRegistry {
    commands: Vec<RegisteredCommand>,
    lookup: HashMap<String, usize>,
}

impl CommandRegistry {
    /// Creates a new empty command registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a command descriptor and its execution handler.
    pub fn register(&mut self, descriptor: Command, handler: CommandHandler) {
        let idx = self.commands.len();
        self.lookup
            .insert(descriptor.name.to_ascii_lowercase(), idx);
        for alias in &descriptor.aliases {
            self.lookup.insert(alias.to_ascii_lowercase(), idx);
        }

        let parsed_cap = if let Some(cap) = &descriptor.capability {
            match CapExpr::parse(cap) {
                Ok(expr) => Some(expr),
                Err(err) => {
                    log::error!(
                        target: crate::consts::log_targets::AUTH,
                        "[Auth] Malformed capability expression '{}' for command '{}': {}",
                        cap, descriptor.name, err
                    );
                    None
                }
            }
        } else {
            None
        };

        self.commands.push(RegisteredCommand {
            descriptor,
            handler,
            parsed_cap,
        });
    }

    /// Dispatches a command by name with caller and raw arguments.
    ///
    /// Performs capability access check if configured on the command descriptor.
    /// Returns `true` if the command was found and consumed by the handler.
    pub fn dispatch(&self, name: &str, caller: i32, args: &str) -> bool {
        let idx_opt = self.lookup.get(name).copied().or_else(|| {
            let lower = name.to_ascii_lowercase();
            self.lookup.get(&lower).copied()
        });

        if let Some(idx) = idx_opt {
            let cmd = &self.commands[idx];

            // Pre-parsed capability access validation (Zero-alloc)
            if let Some(expr) = &cmd.parsed_cap
                && caller > 0
            {
                let has_cap = |c: &str| crate::auth::Auth::has_capability(caller, c);
                if !expr.evaluate(&has_cap) {
                    log::warn!(
                        target: crate::consts::log_targets::AUTH,
                        "[Auth] Caller {} denied command '{}': requires capability.",
                        caller, name
                    );
                    return false;
                }
            }

            (cmd.handler)(caller, args)
        } else {
            false
        }
    }

    /// Returns a slice of all registered commands.
    pub fn commands(&self) -> &[RegisteredCommand] {
        &self.commands
    }

    /// Clears all registered commands from the registry.
    pub fn clear(&mut self) {
        self.commands.clear();
        self.lookup.clear();
    }
}

static GLOBAL_REGISTRY: LazyLock<RwLock<CommandRegistry>> =
    LazyLock::new(|| RwLock::new(CommandRegistry::default()));

/// Registers a command with an execution handler in the global command registry.
pub fn register_command(
    command: Command,
    handler: impl Fn(i32, &str) -> bool + Send + Sync + 'static,
) {
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .register(command, Arc::new(handler));
}

/// Dispatches a command by name through the global command registry.
pub fn dispatch_command(name: &str, caller: i32, args: &str) -> bool {
    GLOBAL_REGISTRY
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .dispatch(name, caller, args)
}

/// Clears all commands from the global command registry.
pub fn clear_commands() {
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
}
