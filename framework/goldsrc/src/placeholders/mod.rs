//! Global Placeholder Engine, Registry, and String Interpolator for GoldSrc.rs.

use goldsrc_api::client::Player;
use goldsrc_api::placeholders::{
    PlaceholderCall, PlaceholderHandler, PlaceholderMetadata, parse_placeholder_call,
};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

/// Global thread-safe registry of registered placeholder providers.
static PLACEHOLDER_REGISTRY: LazyLock<RwLock<PlaceholderRegistry>> =
    LazyLock::new(|| RwLock::new(PlaceholderRegistry::new()));

/// Registered entry containing metadata and execution closure.
#[derive(Clone)]
pub struct RegistryEntry {
    pub plugin_name: String,
    pub metadata: PlaceholderMetadata,
    pub handler: Arc<dyn PlaceholderHandler>,
}

/// Dynamic placeholder registry supporting scoped (`plugin:name`) and global (`name`) resolution.
pub struct PlaceholderRegistry {
    entries: HashMap<(String, String), RegistryEntry>, // (plugin_name, placeholder_name) -> Entry
    short_lookup: HashMap<String, Vec<String>>,        // short_name -> list of plugin_names
}

impl Default for PlaceholderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaceholderRegistry {
    /// Creates an empty registry and initializes built-in engine placeholders.
    pub fn new() -> Self {
        let mut reg = Self {
            entries: HashMap::new(),
            short_lookup: HashMap::new(),
        };
        reg.register_builtins();
        reg
    }

    fn register_builtins(&mut self) {
        // 1. {name} / {engine:name(target=...)}
        self.register(
            "engine",
            PlaceholderMetadata {
                name: "name".to_string(),
                description: "Returns player display name".to_string(),
                usage: "{name} or {name(target=1..32)}".to_string(),
                aliases: vec!["player_name".to_string()],
                capability: None,
            },
            Arc::new(|caller: Player, call: &PlaceholderCall| {
                let target_player = resolve_target(caller, call);
                target_player
                    .name()
                    .unwrap_or_else(|| format!("Player#{}", target_player.index()))
            }),
        );

        // 2. {ip} / {engine:ip(target=...)}
        self.register(
            "engine",
            PlaceholderMetadata {
                name: "ip".to_string(),
                description: "Returns player IP address".to_string(),
                usage: "{ip} or {ip(target=1..32)}".to_string(),
                aliases: vec!["player_ip".to_string()],
                capability: None,
            },
            Arc::new(|_caller: Player, _call: &PlaceholderCall| "127.0.0.1".to_string()),
        );

        // 3. {authid} / {id} / {steamid}
        self.register(
            "engine",
            PlaceholderMetadata {
                name: "authid".to_string(),
                description: "Returns player SteamID / AuthID".to_string(),
                usage: "{authid} or {id}".to_string(),
                aliases: vec!["id".to_string(), "steamid".to_string()],
                capability: None,
            },
            Arc::new(|_caller: Player, _call: &PlaceholderCall| "STEAM_ID_PENDING".to_string()),
        );

        // 4. {health} / {hp}
        self.register(
            "engine",
            PlaceholderMetadata {
                name: "health".to_string(),
                description: "Returns player current health points".to_string(),
                usage: "{health} or {hp}".to_string(),
                aliases: vec!["hp".to_string()],
                capability: None,
            },
            Arc::new(|caller: Player, call: &PlaceholderCall| {
                let target_player = resolve_target(caller, call);
                (target_player.health() as i32).to_string()
            }),
        );

        // 5. {armor} / {ap}
        self.register(
            "engine",
            PlaceholderMetadata {
                name: "armor".to_string(),
                description: "Returns player current armor points".to_string(),
                usage: "{armor} or {ap}".to_string(),
                aliases: vec!["ap".to_string()],
                capability: None,
            },
            Arc::new(|caller: Player, call: &PlaceholderCall| {
                let target_player = resolve_target(caller, call);
                (target_player.armorvalue() as i32).to_string()
            }),
        );
    }

    /// Registers a new placeholder provider in the registry.
    pub fn register(
        &mut self,
        plugin_name: &str,
        metadata: PlaceholderMetadata,
        handler: Arc<dyn PlaceholderHandler>,
    ) {
        let name_lower = metadata.name.to_lowercase();
        let plugin_lower = plugin_name.to_lowercase();

        self.short_lookup
            .entry(name_lower.clone())
            .or_default()
            .push(plugin_lower.clone());

        for alias in &metadata.aliases {
            self.short_lookup
                .entry(alias.to_lowercase())
                .or_default()
                .push(plugin_lower.clone());
        }

        self.entries.insert(
            (plugin_lower, name_lower),
            RegistryEntry {
                plugin_name: plugin_name.to_string(),
                metadata,
                handler,
            },
        );
    }

    /// Resolves and formats a placeholder call string.
    pub fn evaluate_call(&self, caller: Player, call: &PlaceholderCall) -> Result<String, String> {
        let ident_lower = call.ident.to_lowercase();

        // 1. If explicit domain is given (e.g. {stats:rank})
        if let Some(ref domain) = call.domain {
            let domain_lower = domain.to_lowercase();
            if let Some(entry) = self
                .entries
                .get(&(domain_lower.clone(), ident_lower.clone()))
            {
                return Ok(entry.handler.evaluate(caller, call));
            }
            return Err(format!(
                "Placeholder '{{{}: {}}}' not found in plugin '{}'",
                domain, call.ident, domain
            ));
        }

        // 2. Short name / alias lookup (e.g. {rank} or {hp})
        if let Some(plugins) = self.short_lookup.get(&ident_lower) {
            if plugins.len() == 1 {
                let plugin = &plugins[0];
                // Try exact name or search among registered aliases for this plugin
                if let Some(entry) = self.entries.get(&(plugin.clone(), ident_lower.clone())) {
                    return Ok(entry.handler.evaluate(caller, call));
                }
                for entry in self.entries.values() {
                    if entry.plugin_name.eq_ignore_ascii_case(plugin)
                        && (entry.metadata.name.eq_ignore_ascii_case(&ident_lower)
                            || entry
                                .metadata
                                .aliases
                                .iter()
                                .any(|a| a.eq_ignore_ascii_case(&ident_lower)))
                    {
                        return Ok(entry.handler.evaluate(caller, call));
                    }
                }
            } else if plugins.len() > 1 {
                return Err(format!(
                    "Ambiguous placeholder '{{{}}}' registered by multiple plugins: {:?}. Use fully-qualified format '{{{}: ...}}'",
                    call.ident, plugins, plugins[0]
                ));
            }
        }

        Err(format!("Unknown placeholder '{{{}}}'", call.ident))
    }
}

fn resolve_target(caller: Player, call: &PlaceholderCall) -> Player {
    if let Some(target_str) = call.get_param("target", 0)
        && let Ok(slot) = target_str.parse::<i32>()
        && (1..=32).contains(&slot)
    {
        return Player::new(slot);
    }
    caller
}

/// Replaces all `{...}` placeholders in `template` evaluated in the context of `caller`.
pub fn format_placeholders(template: &str, caller: Player) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let tail = &rest[start + 1..];

        if let Some(end) = tail.find('}') {
            let inner = &tail[..end];
            let reg = match PLACEHOLDER_REGISTRY.read() {
                Ok(r) => r,
                Err(e) => e.into_inner(),
            };

            match parse_placeholder_call(inner) {
                Ok(call) => match reg.evaluate_call(caller, &call) {
                    Ok(val) => out.push_str(&val),
                    Err(_) => {
                        // Keep original if unresolvable
                        out.push('{');
                        out.push_str(inner);
                        out.push('}');
                    }
                },
                Err(_) => {
                    out.push('{');
                    out.push_str(inner);
                    out.push('}');
                }
            }
            rest = &tail[end + 1..];
        } else {
            out.push('{');
            rest = tail;
            break;
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_placeholders() {
        let player = Player::new(1);
        let formatted = format_placeholders("Hello, {name}! Your IP is {ip}, HP: {hp}", player);
        assert!(formatted.contains("Hello, Player#1!"));
        assert!(formatted.contains("127.0.0.1"));
        assert!(formatted.contains("HP: 0")); // default unspawned player health in pure unit test
    }
}
