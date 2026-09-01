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
        name: "list",
        aliases: &["ls", "ps"],
        category: "Inspection & Debugging",
        summary: "List loaded WASM plugins in hierarchical tree view or flat list",
        usage: "grs list [OPTIONS]",
        options: &[
            ("-p, --page <N>", "Show specific page (default: 1)"),
            ("-s, --size <N>", "Set page size (default: 5)"),
            ("-a, --all", "Show all plugins (ignore pagination)"),
            ("--flat", "Disable bundle grouping and display as flat list"),
            ("--paused", "Show only paused plugins"),
        ],
        examples: &[
            "grs list",
            "grs list -p 2",
            "grs list --flat",
            "grs list -a",
            "grs list --paused",
        ],
    },
    CommandSpec {
        name: "info",
        aliases: &["show"],
        category: "Inspection & Debugging",
        summary: "Show detailed metadata, systems, exports, and path of a plugin",
        usage: "grs info <name|index...> [OPTIONS]",
        options: &[(
            "-f, --field <FIELD>",
            "Print only a specific field value (e.g. name, version, author, desc, license, url, path, status, systems, deps)",
        )],
        examples: &[
            "grs info vip_core",
            "grs info 0",
            "grs info test_suite -f version",
            "grs info vip_menu --field license",
        ],
    },
    CommandSpec {
        name: "cmds",
        aliases: &["commands"],
        category: "Inspection & Debugging",
        summary: "List registered plugin commands or inspect a specific command",
        usage: "grs cmds [COMMAND_NAME]",
        options: &[],
        examples: &["grs cmds", "grs cmds vip", "grs cmds admin_slay"],
    },
    CommandSpec {
        name: "cmd",
        aliases: &["exec"],
        category: "Execution Control",
        summary: "Execute a plugin command directly through the host dispatcher",
        usage: "grs cmd <command_name> [args...]",
        options: &[],
        examples: &["grs cmd vip_add 1", "grs cmd test_cvar sv_gravity 600"],
    },
    CommandSpec {
        name: "load",
        aliases: &[],
        category: "Plugin Lifecycle",
        summary: "Load WASM plugin component(s) from cstrike/goldsrc/plugins/",
        usage: "grs load <file1> [file2...]",
        options: &[],
        examples: &["grs load admin_system.wasm", "grs load vip_core vip_menu"],
    },
    CommandSpec {
        name: "unload",
        aliases: &[],
        category: "Plugin Lifecycle",
        summary: "Gracefully unload one or all loaded plugins",
        usage: "grs unload <name|index...> [-a|--all]",
        options: &[("-a, --all", "Unload all currently loaded plugins")],
        examples: &[
            "grs unload admin_system",
            "grs unload 1",
            "grs unload --all",
        ],
    },
    CommandSpec {
        name: "reload",
        aliases: &[],
        category: "Plugin Lifecycle",
        summary: "Reload plugin(s) from disk, refreshing bytecode and exports",
        usage: "grs reload <name|index...> [-a|--all]",
        options: &[("-a, --all", "Reload all currently loaded plugins")],
        examples: &["grs reload test_suite", "grs reload 0", "grs reload --all"],
    },
    CommandSpec {
        name: "pause",
        aliases: &[],
        category: "Execution Control",
        summary: "Suspend plugin execution (skips frame and event dispatches)",
        usage: "grs pause <name|index...> [-a|--all]",
        options: &[("-a, --all", "Pause all loaded plugins")],
        examples: &["grs pause vip_menu", "grs pause --all"],
    },
    CommandSpec {
        name: "unpause",
        aliases: &["resume"],
        category: "Execution Control",
        summary: "Resume execution of paused plugin(s)",
        usage: "grs unpause <name|index...> [-a|--all]",
        options: &[("-a, --all", "Unpause all plugins")],
        examples: &["grs unpause vip_menu", "grs unpause -a"],
    },
    CommandSpec {
        name: "status",
        aliases: &[],
        category: "System",
        summary: "Show host runtime stats (active plugins, hot-reload watchers, engine)",
        usage: "grs status",
        options: &[],
        examples: &["grs status"],
    },
    CommandSpec {
        name: "version",
        aliases: &["ver"],
        category: "System",
        summary: "Show host runtime and GoldSrc.rs engine version info",
        usage: "grs version",
        options: &[],
        examples: &["grs version"],
    },
    CommandSpec {
        name: "help",
        aliases: &["?"],
        category: "System",
        summary: "Display general help or specialized help for a command",
        usage: "grs help [COMMAND]",
        options: &[],
        examples: &["grs help", "grs help list", "grs help reload"],
    },
];

/// Find a command specification by query (name or alias).
pub fn find_command_spec(query: &str) -> Option<&'static CommandSpec> {
    BUILTIN_COMMANDS.iter().find(|spec| spec.matches(query))
}

/// Print specialized, formatted help for a single command.
pub fn print_command_help<F: FnMut(&str)>(spec: &CommandSpec, mut out: F) {
    out(&format!("--- GoldSrc.rs Help: grs {} ---\n", spec.name));
    out(&format!("{}\n\n", spec.summary));
    out(&format!("Usage:\n  {}\n\n", spec.usage));

    if !spec.aliases.is_empty() {
        out(&format!("Aliases:\n  {}\n\n", spec.aliases.join(", ")));
    }

    out("Options:\n");
    out("  -h, --help                Show this help message\n");
    for (flag, desc) in spec.options {
        out(&format!("  {:<24}  {}\n", flag, desc));
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

/// Print global CLI help dynamically categorized from all registered command specs.
pub fn print_host_help<F: FnMut(&str)>(mut out: F) {
    out("--- GoldSrc.rs Management CLI ---\n");
    out("Usage: grs <COMMAND> [OPTIONS] [TARGET]\n");
    out("Aliases: goldsrc-rs, mrs, meta-rs\n\n");
    out("Commands:\n");

    let categories = [
        "Plugin Lifecycle",
        "Execution Control",
        "Inspection & Debugging",
        "System",
    ];

    for cat in categories {
        out(&format!("  {}:\n", cat));
        for spec in BUILTIN_COMMANDS.iter().filter(|s| s.category == cat) {
            let aliases_hint = if !spec.aliases.is_empty() {
                format!(" ({})", spec.aliases.join(", "))
            } else {
                String::new()
            };
            out(&format!(
                "    {:<24} {}{}\n",
                spec.name, spec.summary, aliases_hint
            ));
        }
        out("\n");
    }

    out("Run 'grs help <COMMAND>' or 'grs <COMMAND> --help' for detailed option syntax.\n");
}
