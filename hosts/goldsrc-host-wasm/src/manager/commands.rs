//! Command registry subsystem for WASM host plugins.
//!
//! Encapsulates routing from console/chat command names to target plugin indices,
//! handling multi-owner routing, re-indexing on plugin unload, and diagnostics.

use std::collections::HashMap;

/// Registry mapping command names to the plugin indices that handle them.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct CommandRegistry {
    routes: HashMap<String, Vec<usize>>,
}

impl CommandRegistry {
    /// Creates a new, empty command registry.
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// Registers a list of commands for the specified plugin index.
    pub fn register_commands(&mut self, plugin_idx: usize, commands: &[String]) {
        for cmd in commands {
            let owners = self.routes.entry(cmd.clone()).or_default();
            if !owners.contains(&plugin_idx) {
                owners.push(plugin_idx);
            }
        }
    }

    /// Unregisters commands associated with the specified plugin index.
    pub fn unregister_commands(&mut self, plugin_idx: usize, commands: &[String]) {
        for cmd in commands {
            if let Some(owners) = self.routes.get_mut(cmd) {
                owners.retain(|&i| i != plugin_idx);
                if owners.is_empty() {
                    self.routes.remove(cmd);
                }
            }
        }
    }

    /// Updates stored indices after a plugin has been removed at `removed_idx`.
    pub fn reindex_after_removal(&mut self, removed_idx: usize) {
        for owners in self.routes.values_mut() {
            for i in owners.iter_mut() {
                if *i > removed_idx {
                    *i -= 1;
                }
            }
        }
    }

    /// Returns a slice of plugin indices registered to handle `cmd`.
    pub fn get_handlers(&self, cmd: &str) -> Option<&[usize]> {
        self.routes.get(cmd).map(|v| v.as_slice())
    }

    /// Returns all registered command names.
    pub fn command_names(&self) -> Vec<String> {
        self.routes.keys().cloned().collect()
    }

    /// Returns `true` if the registry contains no commands.
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }

    /// Returns the number of unique registered command names.
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    /// Clears all registered commands.
    pub fn clear(&mut self) {
        self.routes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_registry_lifecycle() {
        let mut reg = CommandRegistry::new();
        assert!(reg.is_empty());

        let cmds0 = vec!["vip".to_string(), "vipmenu".to_string()];
        let cmds1 = vec!["vip".to_string(), "admin".to_string()];
        reg.register_commands(0, &cmds0);
        reg.register_commands(1, &cmds1);

        assert_eq!(reg.len(), 3);
        assert_eq!(reg.get_handlers("vip"), Some([0, 1].as_slice()));
        assert_eq!(reg.get_handlers("vipmenu"), Some([0].as_slice()));
        assert_eq!(reg.get_handlers("admin"), Some([1].as_slice()));
        assert_eq!(reg.get_handlers("nonexistent"), None);

        // Unregister plugin 0
        reg.unregister_commands(0, &cmds0);
        reg.reindex_after_removal(0);

        // Now plugin 1 shifted to index 0
        assert_eq!(reg.get_handlers("vip"), Some([0].as_slice()));
        assert_eq!(reg.get_handlers("vipmenu"), None);
        assert_eq!(reg.get_handlers("admin"), Some([0].as_slice()));
        assert_eq!(reg.len(), 2);

        reg.clear();
        assert!(reg.is_empty());
    }
}
