//! In-memory runtime command registry and invocation dispatcher.

use crate::auth::CapExpr;
use crate::command::Command;
use crate::command::error::CommandContext;
use crate::pipeline::{Interceptor, Pipeline, PipelineFlow};
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
    pipeline: Pipeline<CommandContext>,
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

    /// Returns a shared reference to the pre-execution interceptor pipeline.
    pub fn pipeline(&self) -> &Pipeline<CommandContext> {
        &self.pipeline
    }

    /// Returns a mutable reference to the pre-execution interceptor pipeline.
    pub fn pipeline_mut(&mut self) -> &mut Pipeline<CommandContext> {
        &mut self.pipeline
    }

    /// Dispatches a command by name with caller and raw arguments.
    ///
    /// Performs pipeline middleware execution and capability access check if configured.
    /// Returns `true` if the command was found and consumed by the handler.
    pub fn dispatch(&self, name: &str, caller: i32, args: &str) -> bool {
        let idx_opt = self.lookup.get(name).copied().or_else(|| {
            let lower = name.to_ascii_lowercase();
            self.lookup.get(&lower).copied()
        });

        if let Some(idx) = idx_opt {
            let cmd = &self.commands[idx];

            let player = if caller > 0 {
                Some(crate::client::Player::new(caller))
            } else {
                None
            };
            let mut ctx = CommandContext::new(player, cmd.descriptor.target.clone(), name, args);

            // Execute pre-command interceptor pipeline
            match self.pipeline.execute(&mut ctx) {
                PipelineFlow::Block => return false,
                PipelineFlow::Handled => return true,
                PipelineFlow::Continue => {}
            }

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

            (cmd.handler)(caller, &ctx.raw_args)
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

/// Adds an interceptor middleware to the global command pipeline.
pub fn use_command_interceptor<I>(interceptor: I)
where
    I: Interceptor<CommandContext> + Send + Sync + 'static,
{
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .pipeline_mut()
        .use_interceptor(interceptor);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::error::CommandError;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_command_registry_pipeline_interceptor() {
        let mut reg = CommandRegistry::new();
        let intercepted = Arc::new(AtomicU32::new(0));

        let int_clone = intercepted.clone();
        reg.pipeline_mut().use_fn(move |ctx| {
            if ctx.command_name == "blocked_cmd" {
                int_clone.fetch_add(1, Ordering::SeqCst);
                PipelineFlow::Block
            } else {
                PipelineFlow::Continue
            }
        });

        let cmd1 = Command::builder("test_cmd").build();
        let executed1 = Arc::new(AtomicU32::new(0));
        let ex1 = executed1.clone();
        reg.register(
            cmd1,
            Arc::new(move |_caller, _args| {
                ex1.fetch_add(1, Ordering::SeqCst);
                true
            }),
        );

        let cmd2 = Command::builder("blocked_cmd").build();
        let executed2 = Arc::new(AtomicU32::new(0));
        let ex2 = executed2.clone();
        reg.register(
            cmd2,
            Arc::new(move |_caller, _args| {
                ex2.fetch_add(1, Ordering::SeqCst);
                true
            }),
        );

        assert!(reg.dispatch("test_cmd", 1, "hello"));
        assert_eq!(executed1.load(Ordering::SeqCst), 1);

        assert!(!reg.dispatch("blocked_cmd", 1, ""));
        assert_eq!(executed2.load(Ordering::SeqCst), 0);
        assert_eq!(intercepted.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_command_register_handler_with_context() {
        clear_commands();

        let cmd = Command::builder("heal").build();
        let called = Arc::new(AtomicU32::new(0));
        let called_clone = called.clone();

        cmd.register_handler(move |ctx| {
            if ctx.args.is_empty() {
                return Err(CommandError::invalid_args(
                    "heal <amount>",
                    "amount",
                    "missing",
                ));
            }
            called_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        // Test invalid args
        assert!(!dispatch_command("heal", 1, ""));
        assert_eq!(called.load(Ordering::SeqCst), 0);

        // Test valid args
        assert!(dispatch_command("heal", 1, "100"));
        assert_eq!(called.load(Ordering::SeqCst), 1);

        clear_commands();
    }
}
