//! In-memory runtime command registry and 3-tier invocation dispatcher.
//!
//! Implements a 3-tier routing architecture:
//! 1. **Pre-hooks & Interceptors**: Capability evaluation, argument validation, ACLs.
//! 2. **Executor (Monopoly with Fallback)**: Calls `override_handler` if a plugin has registered
//!    an exclusive override; otherwise falls back to the native `default_handler`.
//! 3. **Post-hooks**: Audit logging, event dispatching, metrics tracking with execution outcome.

use crate::auth::CapExpr;
use crate::command::Command;
use crate::command::error::CommandContext;
use crate::pipeline::{Interceptor, Pipeline, PipelineFlow};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

/// Type alias for dynamic command execution handlers.
pub type CommandHandler = Arc<dyn Fn(i32, &str) -> bool + Send + Sync + 'static>;

/// Pre-execution hook for single command routing validation or transformation.
pub type PreHook = Arc<dyn Fn(&mut CommandContext) -> PipelineFlow + Send + Sync + 'static>;

/// Post-execution hook invoked after executor completes, receiving outcome status.
pub type PostHook = Arc<dyn Fn(&CommandContext, bool) + Send + Sync + 'static>;

/// Registered command entry containing the descriptor, 3-tier hooks, and execution handlers.
#[derive(Clone)]
pub struct RegisteredCommand {
    pub descriptor: Command,
    /// Native default fallback handler (e.g., core moderation or engine fallback).
    pub default_handler: CommandHandler,
    /// Optional monopoly override handler registered by an active plugin.
    pub override_handler: Option<CommandHandler>,
    /// Command-specific pre-execution hooks.
    pub pre_hooks: Vec<PreHook>,
    /// Command-specific post-execution hooks.
    pub post_hooks: Vec<PostHook>,
    pub parsed_cap: Option<CapExpr>,
}

impl RegisteredCommand {
    /// Returns the currently active execution handler (override if present, else default).
    #[inline]
    pub fn active_handler(&self) -> &CommandHandler {
        self.override_handler
            .as_ref()
            .unwrap_or(&self.default_handler)
    }

    /// Legacy getter for direct handler invocation.
    #[inline]
    pub fn handler(&self) -> &CommandHandler {
        self.active_handler()
    }

    /// Returns `true` if an external plugin has registered an active monopoly override.
    #[inline]
    pub fn is_overridden(&self) -> bool {
        self.override_handler.is_some()
    }
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

    /// Registers a command descriptor and its default execution handler.
    pub fn register(&mut self, descriptor: Command, default_handler: CommandHandler) {
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
            default_handler,
            override_handler: None,
            pre_hooks: Vec::new(),
            post_hooks: Vec::new(),
            parsed_cap,
        });
    }

    /// Finds a registered command index by name or alias.
    fn find_index(&self, name: &str) -> Option<usize> {
        self.lookup.get(name).copied().or_else(|| {
            let lower = name.to_ascii_lowercase();
            self.lookup.get(&lower).copied()
        })
    }

    /// Overrides the execution handler of an existing command with a monopoly handler.
    ///
    /// Returns `true` if the command was found and overridden, or `false` otherwise.
    pub fn override_executor(&mut self, name: &str, handler: CommandHandler) -> bool {
        if let Some(idx) = self.find_index(name) {
            self.commands[idx].override_handler = Some(handler);
            true
        } else {
            false
        }
    }

    /// Restores the native default execution handler, clearing any active plugin override.
    ///
    /// Returns `true` if the command was found, or `false` otherwise.
    pub fn restore_executor(&mut self, name: &str) -> bool {
        if let Some(idx) = self.find_index(name) {
            self.commands[idx].override_handler = None;
            true
        } else {
            false
        }
    }

    /// Attaches a pre-execution hook to a specific command.
    pub fn add_pre_hook(&mut self, name: &str, hook: PreHook) -> bool {
        if let Some(idx) = self.find_index(name) {
            self.commands[idx].pre_hooks.push(hook);
            true
        } else {
            false
        }
    }

    /// Attaches a post-execution hook to a specific command.
    pub fn add_post_hook(&mut self, name: &str, hook: PostHook) -> bool {
        if let Some(idx) = self.find_index(name) {
            self.commands[idx].post_hooks.push(hook);
            true
        } else {
            false
        }
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
    /// Executes through the 3-tier routing architecture:
    /// - Global interceptors and command-specific pre-hooks.
    /// - Capability verification.
    /// - Active executor (override monopoly handler or default fallback).
    /// - Command-specific post-hooks.
    pub fn dispatch(&self, name: &str, caller: i32, args: &str) -> bool {
        let Some(idx) = self.find_index(name) else {
            return false;
        };
        let cmd = &self.commands[idx];

        let player = if caller > 0 {
            Some(crate::client::Player::new(caller))
        } else {
            None
        };
        let mut ctx = CommandContext::new(player, cmd.descriptor.target.clone(), name, args);

        // 1. Global pre-command interceptor pipeline
        match self.pipeline.execute(&mut ctx) {
            PipelineFlow::Block => return false,
            PipelineFlow::Handled => return true,
            PipelineFlow::Continue => {}
        }

        // 2. Command-specific pre-hooks
        for hook in &cmd.pre_hooks {
            match hook(&mut ctx) {
                PipelineFlow::Block => return false,
                PipelineFlow::Handled => return true,
                PipelineFlow::Continue => {}
            }
        }

        // 3. Pre-parsed capability access validation (Zero-alloc)
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

        // 4. Monopoly Executor with fallback
        let handler = cmd.active_handler();
        let success = handler(caller, &ctx.raw_args);

        // 5. Command-specific post-hooks (audit, telemetry, events)
        for post in &cmd.post_hooks {
            post(&ctx, success);
        }

        success
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

/// Overrides the command executor in the global registry with a monopoly handler.
pub fn override_command_executor(
    name: &str,
    handler: impl Fn(i32, &str) -> bool + Send + Sync + 'static,
) -> bool {
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .override_executor(name, Arc::new(handler))
}

/// Restores the default command executor in the global registry.
pub fn restore_command_executor(name: &str) -> bool {
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .restore_executor(name)
}

/// Registers a command-specific pre-hook in the global registry.
pub fn add_command_pre_hook(
    name: &str,
    hook: impl Fn(&mut CommandContext) -> PipelineFlow + Send + Sync + 'static,
) -> bool {
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .add_pre_hook(name, Arc::new(hook))
}

/// Registers a command-specific post-hook in the global registry.
pub fn add_command_post_hook(
    name: &str,
    hook: impl Fn(&CommandContext, bool) + Send + Sync + 'static,
) -> bool {
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .add_post_hook(name, Arc::new(hook))
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
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

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
    fn test_3_tier_routing_override_and_fallback() {
        let mut reg = CommandRegistry::new();
        let default_called = Arc::new(AtomicU32::new(0));
        let override_called = Arc::new(AtomicU32::new(0));

        let def_c = default_called.clone();
        reg.register(
            Command::builder("kick").build(),
            Arc::new(move |_caller, _args| {
                def_c.fetch_add(1, Ordering::SeqCst);
                true
            }),
        );

        // 1. Initially dispatches to default
        assert!(reg.dispatch("kick", 0, "player1"));
        assert_eq!(default_called.load(Ordering::SeqCst), 1);
        assert_eq!(override_called.load(Ordering::SeqCst), 0);

        // 2. Plugin overrides executor
        let ovr_c = override_called.clone();
        assert!(reg.override_executor(
            "kick",
            Arc::new(move |_caller, _args| {
                ovr_c.fetch_add(1, Ordering::SeqCst);
                true
            })
        ));

        assert!(reg.dispatch("kick", 0, "player1"));
        assert_eq!(default_called.load(Ordering::SeqCst), 1);
        assert_eq!(override_called.load(Ordering::SeqCst), 1);

        // 3. Plugin unloads and restores default executor
        assert!(reg.restore_executor("kick"));
        assert!(reg.dispatch("kick", 0, "player1"));
        assert_eq!(default_called.load(Ordering::SeqCst), 2);
        assert_eq!(override_called.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_command_pre_post_hooks() {
        let mut reg = CommandRegistry::new();
        let pre_ran = Arc::new(AtomicBool::new(false));
        let post_outcome = Arc::new(AtomicBool::new(false));

        reg.register(
            Command::builder("mute").build(),
            Arc::new(|_caller, _args| true),
        );

        let pre_c = pre_ran.clone();
        reg.add_pre_hook(
            "mute",
            Arc::new(move |_ctx| {
                pre_c.store(true, Ordering::SeqCst);
                PipelineFlow::Continue
            }),
        );

        let post_c = post_outcome.clone();
        reg.add_post_hook(
            "mute",
            Arc::new(move |_ctx, success| {
                post_c.store(success, Ordering::SeqCst);
            }),
        );

        assert!(reg.dispatch("mute", 1, "test"));
        assert!(pre_ran.load(Ordering::SeqCst));
        assert!(post_outcome.load(Ordering::SeqCst));
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
