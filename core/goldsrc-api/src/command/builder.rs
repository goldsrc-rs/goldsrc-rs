//! Programmatic builder API for runtime command registration.

use crate::command::CommandTarget;

/// Runtime representation of a registered command.
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    /// Primary command name (e.g. `vipmenu`).
    pub name: String,
    /// Command aliases (e.g. `vip`, `/vip`, `!vip`).
    pub aliases: Vec<String>,
    /// Routing ingress channel and filters.
    pub target: CommandTarget,
    /// Required capability expression, if any (e.g. `vip.menu`, `admin.*`).
    pub capability: Option<String>,
    /// Brief description of the command.
    pub description: String,
    /// Usage help syntax (e.g. `<player> <amount>`).
    pub usage: String,
}

impl Command {
    /// Create a new command builder.
    pub fn builder(name: impl Into<String>) -> CommandBuilder {
        CommandBuilder::new(name)
    }
}

/// Fluent builder for constructing [`Command`] definitions.
#[derive(Debug, Clone)]
pub struct CommandBuilder {
    name: String,
    aliases: Vec<String>,
    target: CommandTarget,
    capability: Option<String>,
    description: String,
    usage: String,
}

impl CommandBuilder {
    /// Starts building a new command with the specified primary name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            aliases: Vec::new(),
            target: CommandTarget::Any,
            capability: None,
            description: String::new(),
            usage: String::new(),
        }
    }

    /// Adds a single alias for this command.
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Adds multiple aliases for this command.
    pub fn aliases(mut self, aliases: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.aliases.extend(aliases.into_iter().map(|a| a.into()));
        self
    }

    /// Sets the command routing target and channel filters.
    pub fn target(mut self, target: CommandTarget) -> Self {
        self.target = target;
        self
    }

    /// Sets the required capability expression for permission checks.
    pub fn capability(mut self, cap: impl Into<String>) -> Self {
        self.capability = Some(cap.into());
        self
    }

    /// Sets the human-readable description for documentation and help listings.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Sets the command syntax usage string.
    pub fn usage(mut self, usage: impl Into<String>) -> Self {
        self.usage = usage.into();
        self
    }

    /// Finalizes the builder into a [`Command`] instance.
    pub fn build(self) -> Command {
        Command {
            name: self.name,
            aliases: self.aliases,
            target: self.target,
            capability: self.capability,
            description: self.description,
            usage: self.usage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{ChatScope, PlayerStateFilter};

    #[test]
    fn test_command_builder() {
        let cmd = Command::builder("vipmenu")
            .alias("/vip")
            .alias("!vip")
            .target(CommandTarget::Chat {
                scope: ChatScope::Both,
                filter: PlayerStateFilter::AliveOnly,
                silent: true,
            })
            .capability("vip.access")
            .description("Opens VIP equipment menu")
            .usage("vipmenu [kit_number]")
            .build();

        assert_eq!(cmd.name, "vipmenu");
        assert_eq!(cmd.aliases, vec!["/vip", "!vip"]);
        assert_eq!(cmd.capability.as_deref(), Some("vip.access"));
        assert_eq!(cmd.description, "Opens VIP equipment menu");
        assert_eq!(cmd.usage, "vipmenu [kit_number]");
        assert!(matches!(
            cmd.target,
            CommandTarget::Chat { silent: true, .. }
        ));
    }
}
