//! Declarative Command Specifications for GoldSrc.rs management CLI.

/// Specification metadata for a built-in CLI command.
pub struct CommandSpec {
    /// Canonical command name.
    pub name: &'static str,
    /// Command aliases.
    pub aliases: &'static [&'static str],
    /// Grouping category for help output.
    pub category: &'static str,
    /// Brief one-line summary.
    pub summary: &'static str,
    /// Syntax usage string.
    pub usage: &'static str,
    /// Option flags and descriptions: `(flag, description)`.
    pub options: &'static [(&'static str, &'static str)],
    /// Usage examples.
    pub examples: &'static [&'static str],
}

impl CommandSpec {
    /// Returns `true` if this specification matches `query` by name or alias.
    pub fn matches(&self, query: &str) -> bool {
        if self.name.eq_ignore_ascii_case(query) {
            return true;
        }
        self.aliases.iter().any(|a| a.eq_ignore_ascii_case(query))
    }
}

/// Canonical table of all built-in management commands.
pub const BUILTIN_COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: "plugins",
        aliases: &["pl", "p"],
        category: "plugin:lifecycle",
        summary: "Manage WASM plugins (list, info, load, unload, reload, pause, unpause, cmds)",
        usage: "grs plugins <subcommand> [OPTIONS] [TARGET]",
        options: &[
            (
                "list [OPTIONS]",
                "List loaded plugins (options: --flat, -p, -s, -a, --paused)",
            ),
            ("info <name|index>", "Show detailed metadata for a plugin"),
            ("load <file...>", "Load WASM plugin component(s)"),
            (
                "unload <name|index...> [-a]",
                "Unload one or all loaded plugins",
            ),
            (
                "reload <name|index...> [-a]",
                "Reload one or all loaded plugins from disk",
            ),
            ("pause <name|index...> [-a]", "Pause plugin execution"),
            (
                "unpause <name|index...> [-a]",
                "Resume execution of paused plugin(s)",
            ),
            (
                "cmds [command_name]",
                "List registered plugin commands or inspect a command",
            ),
        ],
        examples: &[
            "grs plugins list",
            "grs pl ps",
            "grs plugins info vip_core",
            "grs plugins load admin_system.wasm",
            "grs plugins reload --all",
            "grs plugins pause vip_menu",
            "grs plugins unpause vip_menu",
        ],
    },
    CommandSpec {
        name: "ps",
        aliases: &["list", "ls"],
        category: "plugin:lifecycle",
        summary: "List loaded plugins (fast shortcut for 'grs plugins list')",
        usage: "grs ps [OPTIONS]",
        options: &[
            ("--flat", "Print flat, unformatted list"),
            ("-p, --paused", "Show only paused plugins"),
            ("-a, --active", "Show only running/active plugins"),
        ],
        examples: &["grs ps", "grs ps --flat", "grs ps -p"],
    },
    CommandSpec {
        name: "reload",
        aliases: &["rld"],
        category: "plugin:lifecycle",
        summary: "Reload WASM plugins from disk (fast shortcut for 'grs plugins reload')",
        usage: "grs reload [TARGET...] [-a|--all]",
        options: &[("-a, --all", "Reload all plugins")],
        examples: &["grs reload --all", "grs reload vip_core"],
    },
    CommandSpec {
        name: "watchers",
        aliases: &["watch", "w"],
        category: "watcher:fs",
        summary: "Inspect and control filesystem watchers",
        usage: "grs watchers <list|pause|resume> [OPTIONS]",
        options: &[
            ("list [--json]", "List all registered filesystem watchers"),
            (
                "pause <id>",
                "Pause filesystem watcher by ID (e.g. core:plugins)",
            ),
            ("resume <id>", "Resume paused filesystem watcher by ID"),
        ],
        examples: &[
            "grs watchers list",
            "grs w list",
            "grs watchers pause core:plugins",
            "grs watchers resume core:plugins",
        ],
    },
    CommandSpec {
        name: "cmd",
        aliases: &["exec", "c"],
        category: "exec:dispatch",
        summary: "Execute a plugin command directly through the host dispatcher",
        usage: "grs cmd <command_name> [args...]",
        options: &[],
        examples: &["grs cmd vip_add 1", "grs cmd test_cvar sv_gravity 600"],
    },
    CommandSpec {
        name: "status",
        aliases: &["stat", "st", "s"],
        category: "sys:runtime",
        summary: "Show host runtime stats (active plugins, hot-reload watchers, engine)",
        usage: "grs status",
        options: &[],
        examples: &["grs status", "grs st"],
    },
    CommandSpec {
        name: "version",
        aliases: &["ver", "v"],
        category: "sys:runtime",
        summary: "Show host runtime and GoldSrc.rs engine version info",
        usage: "grs version",
        options: &[],
        examples: &["grs version", "grs ver"],
    },
    CommandSpec {
        name: "help",
        aliases: &["?"],
        category: "sys:help",
        summary: "Display general help or specialized help for a command",
        usage: "grs help [COMMAND|NAMESPACE]",
        options: &[],
        examples: &[
            "grs help",
            "grs help plugins",
            "grs help plugin",
            "grs help watcher",
        ],
    },
];

/// Canonical table of built-in DSL category namespaces and descriptions.
pub const BUILTIN_CATEGORIES: &[(&str, &str)] = &[
    (
        "plugin:lifecycle",
        "Plugin management & execution lifecycle",
    ),
    ("watcher:fs", "Filesystem watchers & hot-reload controls"),
    ("exec:dispatch", "Direct command execution & dispatch"),
    ("sys:runtime", "Engine runtime status & system telemetry"),
    ("sys:help", "Interactive help & introspection system"),
];

/// Find a command specification by query (name or alias).
pub fn find_command_spec(query: &str) -> Option<&'static CommandSpec> {
    BUILTIN_COMMANDS.iter().find(|spec| spec.matches(query))
}

/// Print specialized, formatted help for a single command.
pub fn print_command_help<F: FnMut(&str)>(spec: &CommandSpec, mut out: F) {
    out(&format!("--- GoldSrc.rs Help: grs {} ---\n", spec.name));
    out(&format!("{}\n", spec.summary));
    out(&format!("Namespace:   [{}]\n\n", spec.category));
    out(&format!("Usage:\n  {}\n\n", spec.usage));

    if !spec.aliases.is_empty() {
        out(&format!("Aliases:\n  {}\n\n", spec.aliases.join(", ")));
    }

    out("Options / Subcommands:\n");
    out("  -h, --help                Show this help message\n");
    for (flag, desc) in spec.options {
        out(&format!("  {:<26} {}\n", flag, desc));
    }
    out("\n");

    if !spec.examples.is_empty() {
        out("Examples:\n");
        for ex in spec.examples {
            out(&format!("  {}\n", ex));
        }
        out("\n");
    }
}

/// Print specialized help for all commands matching a namespace or category.
pub fn print_category_help<F: FnMut(&str)>(category_query: &str, mut out: F) -> bool {
    let query_lower = category_query.to_lowercase();
    let matching_specs: Vec<&CommandSpec> = BUILTIN_COMMANDS
        .iter()
        .filter(|s| {
            s.category.eq_ignore_ascii_case(&query_lower)
                || s.category.starts_with(&query_lower)
                || s.category
                    .split_once(':')
                    .map(|(ns, _)| ns.eq_ignore_ascii_case(&query_lower))
                    .unwrap_or(false)
        })
        .collect();

    if matching_specs.is_empty() {
        return false;
    }

    out(&format!(
        "--- GoldSrc.rs Commands in namespace '{}' ---\n\n",
        category_query
    ));
    for spec in matching_specs {
        let aliases_hint = if !spec.aliases.is_empty() {
            format!(" ({})", spec.aliases.join(", "))
        } else {
            String::new()
        };
        out(&format!(
            "  {:<14} [{:<16}] {}{}\n",
            spec.name, spec.category, spec.summary, aliases_hint
        ));
    }
    out("\nRun 'grs help <COMMAND>' for detailed command options.\n");
    true
}

/// Print global CLI help dynamically categorized by DSL namespaces.
pub fn print_host_help<F: FnMut(&str)>(mut out: F) {
    out("--- GoldSrc.rs Management CLI ---\n");
    out("Usage: grs <COMMAND> [SUBCOMMAND] [OPTIONS] [TARGET]\n");
    out("Aliases: goldsrc-rs, mrs, meta-rs\n\n");
    out("Commands by namespace:\n");

    for &(cat, desc) in BUILTIN_CATEGORIES {
        out(&format!("  [{}] - {}\n", cat, desc));
        for spec in BUILTIN_COMMANDS.iter().filter(|s| s.category == cat) {
            let aliases_hint = if !spec.aliases.is_empty() {
                format!(" ({})", spec.aliases.join(", "))
            } else {
                String::new()
            };
            out(&format!(
                "    {:<16} {}{}\n",
                spec.name, spec.summary, aliases_hint
            ));
        }
        out("\n");
    }

    out("Run 'grs help <COMMAND>' or 'grs help <namespace>' for detailed option syntax.\n");
}
