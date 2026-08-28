//! Host CLI dispatch and declarative formatting logic for GoldSrc.rs.

use goldsrc_wasm_host::PluginManager;
use lexopt::Arg;
use std::ffi::{CStr, OsString, c_char};
use std::sync::OnceLock;

/// Backend accessors needed to run the host CLI as a server command.
///
/// The backend provides C-compatible argv access, a live handle to its
/// [`PluginManager`], an output callback and version metadata. Shared by both
/// backends so the `meta-rs`/`mrs`/`grs` command set behaves identically.
pub struct HostCliBackend {
    /// Returns the current engine-provided argc.
    pub argc: fn() -> i32,
    /// Returns the engine-provided argv entry at `i`.
    pub argv: fn(i32) -> *const c_char,
    /// Prints a line to the server console.
    pub print: fn(&str),
    /// `(package_version, git_hash, build_target)`.
    pub version: (&'static str, &'static str, &'static str),
}

static HOST_CLI: OnceLock<HostCliBackend> = OnceLock::new();

/// Initialize the shared host CLI backend accessors. Call once at backend init.
pub fn init_host_cli(backend: HostCliBackend) {
    let _ = HOST_CLI.set(backend);
}

/// Shared server-command handler for `meta-rs` / `mrs` / `grs`.
///
/// # Safety
/// Registered as a C server command; the engine provides the argv accessors.
pub unsafe extern "C" fn handle_host_command() {
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let Some(backend) = HOST_CLI.get() else {
            return;
        };
        let argc = (backend.argc)();
        if argc == 0 {
            return;
        }
        let mut raw_args = Vec::new();
        for i in 0..argc {
            let arg_ptr = (backend.argv)(i);
            if !arg_ptr.is_null()
                && let Ok(cstr) = unsafe { CStr::from_ptr(arg_ptr) }.to_str()
            {
                raw_args.push(OsString::from(cstr));
            }
        }
        crate::host::HostRuntime::with_manager(|manager| {
            dispatch_host_command(raw_args, manager, backend.version, backend.print);
        });
    }));
    if let Err(err) = res {
        let err_msg = if let Some(s) = err.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = err.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        if let Some(backend) = HOST_CLI.get() {
            (backend.print)(&format!(
                "[GoldSrc.rs PANIC] Caught panic in CLI Command: {}\n",
                err_msg
            ));
        }
    }
}

/// Shared server-command handler for WASM plugin commands (e.g. `test_player`, `vip_add`).
///
/// # Safety
/// Registered as a C server command via `pfnAddServerCommand`.
pub unsafe extern "C" fn handle_plugin_server_command() {
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let Some(backend) = HOST_CLI.get() else {
            return;
        };
        let argc = (backend.argc)();
        if argc == 0 {
            return;
        }
        let name_ptr = (backend.argv)(0);
        if name_ptr.is_null() {
            return;
        }
        let Ok(cmd_name) = (unsafe { CStr::from_ptr(name_ptr) }).to_str() else {
            return;
        };

        // Reconstruct args string from argv(1..argc)
        let mut args = String::new();
        for i in 1..argc {
            let arg_ptr = (backend.argv)(i);
            if !arg_ptr.is_null()
                && let Ok(arg_s) = (unsafe { CStr::from_ptr(arg_ptr) }).to_str()
            {
                if !args.is_empty() {
                    args.push(' ');
                }
                args.push_str(arg_s);
            }
        }

        let handled = crate::hooks::dispatch_command(cmd_name, &args);
        if !handled {
            (backend.print)(&format!(
                "[GoldSrc.rs] Command '{}' was not handled by any active plugin.\n",
                cmd_name
            ));
        }
        // Force flush deferred prints immediately so command output appears in real time
        (backend.print)("");
    }));
    if let Err(err) = res {
        let err_msg = if let Some(s) = err.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = err.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        if let Some(backend) = HOST_CLI.get() {
            (backend.print)(&format!(
                "[GoldSrc.rs PANIC] Caught panic in plugin command handler: {}\n",
                err_msg
            ));
        }
    }
}

/// Register server commands pointing at the shared host CLI handler.
///
/// If `names` is empty, defaults to `&["goldsrc-rs", "grs", "meta-rs", "mrs"]`.
pub fn register_host_commands_with_names(
    names: &[&str],
    mut add: impl FnMut(&str, unsafe extern "C" fn()),
) {
    for &name in names {
        add(name, handle_host_command);
    }
}

/// Register default server commands (`goldsrc-rs`, `grs`, `meta-rs`, `mrs`).
pub fn register_host_commands(add: impl FnMut(&str, unsafe extern "C" fn())) {
    register_host_commands_with_names(&["goldsrc-rs", "grs", "meta-rs", "mrs"], add);
}

/// Register all commands currently exposed by loaded plugins as direct server console commands.
pub fn register_plugin_server_commands(mut add: impl FnMut(&str, unsafe extern "C" fn())) {
    crate::host::HostRuntime::with_manager(|manager| {
        if let Some(mgr) = manager {
            for cmd in mgr.registered_commands() {
                add(&cmd, handle_plugin_server_command);
            }
        }
    });
}

// ============================================================================
// Declarative Command Specifications
// ============================================================================

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

pub fn dispatch_host_command<F: FnMut(&str)>(
    raw_args: Vec<std::ffi::OsString>,
    manager: Option<&mut PluginManager>,
    version_info: (&str, &str, &str),
    mut out: F,
) {
    let (pkg_version, git_hash, build_target) = version_info;

    let mut parser = lexopt::Parser::from_args(raw_args);
    // Skip binary name ("mrs", "meta-rs", or "grs")
    let _ = parser.next();

    let command_arg = match parser.next() {
        Ok(Some(Arg::Value(val))) => val.to_string_lossy().to_lowercase(),
        Ok(Some(Arg::Short('h') | Arg::Long("help"))) => {
            print_host_help(out);
            return;
        }
        _ => {
            print_host_help(out);
            return;
        }
    };

    // If user ran `grs help [target]`
    if command_arg == "help" || command_arg == "?" {
        if let Ok(Some(Arg::Value(sub_name))) = parser.next() {
            let sub_query = sub_name.to_string_lossy();
            if let Some(spec) = find_command_spec(&sub_query) {
                print_command_help(spec, out);
            } else {
                out(&format!(
                    "[GoldSrc.rs] Unknown command '{}'. Run 'grs help' for command list.\n",
                    sub_query
                ));
            }
        } else {
            print_host_help(out);
        }
        return;
    }

    let Some(spec) = find_command_spec(&command_arg) else {
        out(&format!(
            "[GoldSrc.rs] Unknown command '{}'. Run 'grs help' for available commands.\n",
            command_arg
        ));
        return;
    };

    // Subcommand execution with automated --help / -h inspection
    match spec.name {
        "list" => {
            let mut page: usize = 1;
            let mut size: Option<usize> = None;
            let mut only_paused = false;
            let mut all = false;
            let mut flat_view = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('h') | Arg::Long("help") => {
                        print_command_help(spec, out);
                        return;
                    }
                    Arg::Short('p') | Arg::Long("page") => {
                        if let Ok(val) = parser.value() {
                            page = val.to_string_lossy().parse().unwrap_or(1);
                        }
                    }
                    Arg::Short('s') | Arg::Long("size") => {
                        if let Ok(val) = parser.value() {
                            size = Some(val.to_string_lossy().parse().unwrap_or(5));
                        }
                    }
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Long("flat") => flat_view = true,
                    Arg::Long("paused") => only_paused = true,
                    _ => {}
                }
            }

            let Some(manager) = manager else {
                out("[GoldSrc.rs] Error: WASM Host not initialized.\n");
                return;
            };

            let mut plugins = manager.get_plugins_info();
            if only_paused {
                plugins
                    .retain(|p| matches!(p.status, goldsrc_wasm_host::PluginStatus::Paused { .. }));
            }

            let total_plugins = plugins.len();
            if plugins.is_empty() {
                out("[GoldSrc.rs] WASM plugins (0):\n  (No plugins found)\n");
                return;
            }

            // Check if we should render hierarchical tree view (default if bundles exist and !flat_view)
            let has_bundles = plugins.iter().any(|p| {
                p.metadata
                    .as_ref()
                    .and_then(|m| m.bundle.as_ref())
                    .is_some()
                    || p.name.contains('/')
            });

            if !flat_view && has_bundles {
                out(&format!(
                    "[GoldSrc.rs] WASM plugins ({} loaded):\n",
                    total_plugins
                ));

                // Partition plugins into root vs bundles
                let mut root_plugins = Vec::new();
                let mut bundle_groups: std::collections::BTreeMap<
                    String,
                    Vec<&goldsrc_wasm_host::PluginInfo>,
                > = std::collections::BTreeMap::new();

                for p in &plugins {
                    let bundle_name = p
                        .metadata
                        .as_ref()
                        .and_then(|m| m.bundle.as_ref().cloned())
                        .or_else(|| {
                            if let Some((b, _)) = p.name.split_once('/') {
                                Some(b.to_string())
                            } else {
                                None
                            }
                        });

                    if let Some(b) = bundle_name {
                        bundle_groups.entry(b).or_default().push(p);
                    } else {
                        root_plugins.push(p);
                    }
                }

                // 1. Render root plugins
                for p in root_plugins {
                    let status = p.status.label();
                    let version_str = p
                        .metadata
                        .as_ref()
                        .map(|m| format!("v{}", m.version))
                        .unwrap_or_else(|| "v1.0.0".to_string());
                    let author_str = p
                        .metadata
                        .as_ref()
                        .map(|m| {
                            if m.author.trim().is_empty() {
                                goldsrc_api::consts::DEFAULT_PLUGIN_AUTHOR
                            } else {
                                m.author.as_str()
                            }
                        })
                        .unwrap_or(goldsrc_api::consts::DEFAULT_PLUGIN_AUTHOR);

                    out(&format!(
                        "  [#{}] {:<16} {:<8} {:<18} | {:<7}\n",
                        p.index, p.name, version_str, author_str, status
                    ));
                }

                // 2. Render bundles with tree branches
                for (bundle_name, bundle_plugins) in bundle_groups {
                    out(&format!(
                        "  [{}/] ({} plugins)\n",
                        bundle_name,
                        bundle_plugins.len()
                    ));
                    let count = bundle_plugins.len();
                    for (i, p) in bundle_plugins.iter().enumerate() {
                        let is_last = i + 1 == count;
                        let branch = if is_last { "└── " } else { "├── " };
                        let status = p.status.label();
                        let version_str = p
                            .metadata
                            .as_ref()
                            .map(|m| format!("v{}", m.version))
                            .unwrap_or_else(|| "v1.0.0".to_string());
                        let display_name = p
                            .name
                            .strip_prefix(&format!("{}/", bundle_name))
                            .unwrap_or(&p.name);
                        let author_str = p
                            .metadata
                            .as_ref()
                            .map(|m| {
                                if m.author.trim().is_empty() {
                                    goldsrc_api::consts::DEFAULT_PLUGIN_AUTHOR
                                } else {
                                    m.author.as_str()
                                }
                            })
                            .unwrap_or(goldsrc_api::consts::DEFAULT_PLUGIN_AUTHOR);

                        out(&format!(
                            "    {}[#{}] {:<14} {:<8} {:<18} | {:<7}\n",
                            branch, p.index, display_name, version_str, author_str, status
                        ));
                    }
                }
                return;
            }

            // Flat paginated view
            let page_size = if all {
                total_plugins.max(1)
            } else {
                size.unwrap_or(5)
            };
            let total_pages = (total_plugins + page_size - 1) / page_size.max(1);
            let page_idx = page.saturating_sub(1);

            out(&format!(
                "[GoldSrc.rs] WASM plugins ({}) [Page {}/{}]:\n",
                total_plugins,
                if total_pages == 0 { 1 } else { page },
                if total_pages == 0 { 1 } else { total_pages }
            ));

            let start = (page_idx * page_size).min(total_plugins);
            let end = (start + page_size).min(total_plugins);

            for p in &plugins[start..end] {
                let status = p.status.label();

                let version_str = p
                    .metadata
                    .as_ref()
                    .map(|m| format!("v{}", m.version))
                    .unwrap_or_else(|| "v1.0.0".to_string());

                let author_str = p
                    .metadata
                    .as_ref()
                    .map(|m| {
                        if m.author.trim().is_empty() {
                            goldsrc_api::consts::DEFAULT_PLUGIN_AUTHOR
                        } else {
                            m.author.as_str()
                        }
                    })
                    .unwrap_or(goldsrc_api::consts::DEFAULT_PLUGIN_AUTHOR);

                let desc_str = p
                    .metadata
                    .as_ref()
                    .and_then(|m| {
                        if m.description.is_empty() {
                            None
                        } else {
                            Some(m.description.as_str())
                        }
                    })
                    .unwrap_or("-");

                out(&format!(
                    "  [#{}] {:<16} {:<8} {:<18} | {:<7} | {}\n",
                    p.index, p.name, version_str, author_str, status, desc_str
                ));
            }
        }
        "load" => {
            let mut targets = Vec::new();
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('h') | Arg::Long("help") => {
                        print_command_help(spec, out);
                        return;
                    }
                    Arg::Value(val) => {
                        targets.push(val.to_string_lossy().into_owned());
                    }
                    _ => {}
                }
            }
            if targets.is_empty() {
                print_command_help(spec, out);
                return;
            }
            let Some(manager) = manager else {
                out("[GoldSrc.rs] Error: WASM Host not initialized.\n");
                return;
            };
            for t in targets {
                match manager.load_plugin_by_name(&t) {
                    Ok(msg) => out(&msg),
                    Err(err) => out(&err.to_string()),
                }
            }
        }
        "unload" => {
            let mut targets = Vec::new();
            let mut all = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('h') | Arg::Long("help") => {
                        print_command_help(spec, out);
                        return;
                    }
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Value(val) => targets.push(val.to_string_lossy().into_owned()),
                    _ => {}
                }
            }
            let Some(manager) = manager else {
                out("[GoldSrc.rs] Error: WASM Host not initialized.\n");
                return;
            };
            if all {
                let msg = manager.unload_all_plugins();
                out(&msg);
            } else if !targets.is_empty() {
                for t in targets {
                    match manager.unload_plugin_by_query(&t) {
                        Ok(msg) => out(&msg),
                        Err(err) => out(&err.to_string()),
                    }
                }
            } else {
                print_command_help(spec, out);
            }
        }
        "reload" => {
            let mut targets = Vec::new();
            let mut all = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('h') | Arg::Long("help") => {
                        print_command_help(spec, out);
                        return;
                    }
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Value(val) => targets.push(val.to_string_lossy().into_owned()),
                    _ => {}
                }
            }
            let Some(manager) = manager else {
                out("[GoldSrc.rs] Error: WASM Host not initialized.\n");
                return;
            };
            if all {
                let msg = manager.reload_all_plugins();
                out(&msg);
            } else if !targets.is_empty() {
                for t in targets {
                    match manager.reload_plugin_by_query(&t) {
                        Ok(msg) => out(&msg),
                        Err(err) => out(&err.to_string()),
                    }
                }
            } else {
                print_command_help(spec, out);
            }
        }
        "pause" => {
            let mut targets = Vec::new();
            let mut all = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('h') | Arg::Long("help") => {
                        print_command_help(spec, out);
                        return;
                    }
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Value(val) => targets.push(val.to_string_lossy().into_owned()),
                    _ => {}
                }
            }
            let Some(manager) = manager else {
                out("[GoldSrc.rs] Error: WASM Host not initialized.\n");
                return;
            };
            if all {
                let msg = manager.pause_all_plugins(true);
                out(&msg);
            } else if !targets.is_empty() {
                for t in targets {
                    match manager.pause_plugin(&t, true) {
                        Ok(msg) => out(&msg),
                        Err(err) => out(&err.to_string()),
                    }
                }
            } else {
                print_command_help(spec, out);
            }
        }
        "unpause" => {
            let mut targets = Vec::new();
            let mut all = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('h') | Arg::Long("help") => {
                        print_command_help(spec, out);
                        return;
                    }
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Value(val) => targets.push(val.to_string_lossy().into_owned()),
                    _ => {}
                }
            }
            let Some(manager) = manager else {
                out("[GoldSrc.rs] Error: WASM Host not initialized.\n");
                return;
            };
            if all {
                let msg = manager.pause_all_plugins(false);
                out(&msg);
            } else if !targets.is_empty() {
                for t in targets {
                    match manager.pause_plugin(&t, false) {
                        Ok(msg) => out(&msg),
                        Err(err) => out(&err.to_string()),
                    }
                }
            } else {
                print_command_help(spec, out);
            }
        }
        "info" => {
            let mut targets = Vec::new();
            let mut requested_field: Option<String> = None;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('h') | Arg::Long("help") => {
                        print_command_help(spec, out);
                        return;
                    }
                    Arg::Short('f') | Arg::Long("field") => {
                        if let Ok(val) = parser.value() {
                            requested_field = Some(val.to_string_lossy().to_lowercase());
                        }
                    }
                    Arg::Value(val) => targets.push(val.to_string_lossy().into_owned()),
                    _ => {}
                }
            }
            if targets.is_empty() {
                print_command_help(spec, out);
                return;
            }
            let Some(manager) = manager else {
                out("[GoldSrc.rs] Error: WASM Host not initialized.\n");
                return;
            };
            for t in targets {
                if let Some(idx) = manager.find_plugin(&t) {
                    let info = &manager.get_plugins_info()[idx];
                    let clean_path = crate::paths::PathResolver::normalize(&info.path);
                    let status_str = match &info.status {
                        goldsrc_wasm_host::PluginStatus::Loaded => "Loaded".to_string(),
                        goldsrc_wasm_host::PluginStatus::Running => "Running".to_string(),
                        goldsrc_wasm_host::PluginStatus::Paused { reason } => {
                            if let Some(r) = reason {
                                format!("Paused ({r})")
                            } else {
                                "Paused".to_string()
                            }
                        }
                        goldsrc_wasm_host::PluginStatus::Blocked { reason } => {
                            format!("Blocked ({reason})")
                        }
                        goldsrc_wasm_host::PluginStatus::Degraded { reason } => {
                            format!("Degraded ({reason})")
                        }
                        goldsrc_wasm_host::PluginStatus::Poisoned { error } => {
                            format!("Poisoned ({error})")
                        }
                        goldsrc_wasm_host::PluginStatus::Unloaded => "Unloaded".to_string(),
                    };

                    let meta_name = info
                        .metadata
                        .as_ref()
                        .map(|m| m.name.as_str())
                        .unwrap_or(info.name.as_str());
                    let meta_version = info
                        .metadata
                        .as_ref()
                        .map(|m| m.version.as_str())
                        .unwrap_or(goldsrc_api::consts::DEFAULT_PLUGIN_VERSION);
                    let meta_author = info
                        .metadata
                        .as_ref()
                        .map(|m| m.author.as_str())
                        .unwrap_or(goldsrc_api::consts::DEFAULT_PLUGIN_AUTHOR);
                    let meta_desc = info
                        .metadata
                        .as_ref()
                        .map(|m| m.description.as_str())
                        .unwrap_or(goldsrc_api::consts::DEFAULT_PLUGIN_DESCRIPTION);
                    let meta_license = info
                        .metadata
                        .as_ref()
                        .map(|m| m.license.as_str())
                        .unwrap_or(goldsrc_api::consts::DEFAULT_PLUGIN_LICENSE);
                    let meta_url = info
                        .metadata
                        .as_ref()
                        .map(|m| m.url.as_str())
                        .unwrap_or(goldsrc_api::consts::DEFAULT_PLUGIN_URL);
                    let meta_systems = info
                        .metadata
                        .as_ref()
                        .map(|m| {
                            if m.systems.is_empty() {
                                goldsrc_api::consts::DEFAULT_PLUGIN_SYSTEMS.to_string()
                            } else {
                                m.systems.join(", ")
                            }
                        })
                        .unwrap_or_else(|| goldsrc_api::consts::DEFAULT_PLUGIN_SYSTEMS.to_string());
                    let meta_require = info
                        .metadata
                        .as_ref()
                        .map(|m| {
                            if m.require.is_empty() {
                                goldsrc_api::consts::DEFAULT_PLUGIN_REQUIRE.to_string()
                            } else {
                                m.require.join(", ")
                            }
                        })
                        .unwrap_or_else(|| goldsrc_api::consts::DEFAULT_PLUGIN_REQUIRE.to_string());

                    if let Some(ref field) = requested_field {
                        let value = match field.as_str() {
                            "name" => meta_name,
                            "version" => meta_version,
                            "author" => meta_author,
                            "description" => meta_desc,
                            "license" => meta_license,
                            "url" => meta_url,
                            "path" => &clean_path,
                            "status" => &status_str,
                            "systems" => &meta_systems,
                            "require" => &meta_require,
                            "index" => {
                                out(&format!("{}\n", info.index));
                                continue;
                            }
                            other => {
                                out(&format!(
                                    "[GoldSrc.rs] Unknown field '{}'. Supported: name, version, author, description, license, url, path, status, systems, require, index.\n",
                                    other
                                ));
                                continue;
                            }
                        };
                        out(&format!("{}\n", value));
                        continue;
                    }

                    out(&format!("--- Plugin Info: {} ---\n", info.name));
                    out(&format!("  Index:        #{}\n", info.index));
                    out(&format!("  Path:         \"{}\"\n", clean_path));
                    out(&format!("  Status:       {}\n", status_str));
                    out(&format!("  Meta Name:    {}\n", meta_name));
                    out(&format!("  Version:      {}\n", meta_version));
                    out(&format!("  Author:       {}\n", meta_author));
                    out(&format!("  Description:  {}\n", meta_desc));
                    out(&format!("  License:      {}\n", meta_license));
                    out(&format!("  URL:          {}\n", meta_url));
                    out(&format!("  Systems:      {}\n", meta_systems));
                    out(&format!("  Require:      {}\n", meta_require));
                } else {
                    out(&format!("[GoldSrc.rs] Plugin '{}' not found.\n", t));
                }
            }
        }
        "status" => {
            while let Ok(Some(arg)) = parser.next() {
                if let Arg::Short('h') | Arg::Long("help") = arg {
                    print_command_help(spec, out);
                    return;
                }
            }
            let Some(manager) = manager else {
                out("[GoldSrc.rs] Error: WASM Host not initialized.\n");
                return;
            };
            let (plugins_count, watchers_count) = manager.get_status_info();
            out("--- GoldSrc.rs Host Engine Status ---\n");
            out(&format!(
                "  Version:    v{} (git: {})\n",
                pkg_version, git_hash
            ));
            out(&format!("  Target:     {}\n", build_target));
            out("  WASM Engine: wasmtime (Component Model)\n");
            out(&format!("  Plugins:    {} loaded\n", plugins_count));
            out(&format!(
                "  Watchers:   {} active directory watcher(s)\n",
                watchers_count
            ));
        }
        "cmds" => {
            let mut positional = Vec::new();
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('h') | Arg::Long("help") => {
                        print_command_help(spec, out);
                        return;
                    }
                    Arg::Value(v) => {
                        positional.push(v.to_string_lossy().to_string());
                    }
                    _ => {}
                }
            }
            let Some(manager) = manager else {
                out("[GoldSrc.rs] Error: WASM Host not initialized.\n");
                return;
            };
            let plugins = manager.get_plugins_info();

            if let Some(query) = positional.first() {
                let query_clean = query.trim().to_ascii_lowercase();
                let mut found_count = 0;

                for p in &plugins {
                    if let Some(meta) = &p.metadata {
                        if !meta.command_defs.is_empty() {
                            for cmd in &meta.command_defs {
                                let name_matches = cmd.name.eq_ignore_ascii_case(&query_clean)
                                    || cmd
                                        .name
                                        .trim_start_matches(['/', '!'])
                                        .eq_ignore_ascii_case(
                                            query_clean.trim_start_matches(['/', '!']),
                                        );
                                let alias_matches = cmd.aliases.iter().any(|a| {
                                    a.eq_ignore_ascii_case(&query_clean)
                                        || a.trim_start_matches(['/', '!']).eq_ignore_ascii_case(
                                            query_clean.trim_start_matches(['/', '!']),
                                        )
                                });

                                if name_matches || alias_matches {
                                    found_count += 1;
                                    out(&format!("--- Command: {} ---\n", cmd.name));
                                    out(&format!(
                                        "  Plugin:      {} v{}\n",
                                        meta.name, meta.version
                                    ));
                                    if !cmd.description.is_empty() {
                                        out(&format!("  Description: {}\n", cmd.description));
                                    }
                                    let usage = if !cmd.usage.is_empty() {
                                        cmd.usage.as_str()
                                    } else {
                                        cmd.name.as_str()
                                    };
                                    out(&format!("  Usage:       {}\n", usage));
                                    if !cmd.aliases.is_empty() {
                                        out(&format!(
                                            "  Aliases:     {}\n",
                                            cmd.aliases.join(", ")
                                        ));
                                    }
                                    let access =
                                        cmd.capability.as_deref().unwrap_or("Public (None)");
                                    out(&format!("  Access:      {}\n", access));
                                    out("\n");
                                }
                            }
                        } else if meta
                            .commands
                            .iter()
                            .any(|c| c.eq_ignore_ascii_case(&query_clean))
                        {
                            found_count += 1;
                            out(&format!("--- Command: {} ---\n", query));
                            out(&format!("  Plugin:      {} v{}\n", meta.name, meta.version));
                            if !meta.description.is_empty() {
                                out(&format!("  Description: {}\n", meta.description));
                            }
                            out(&format!("  Usage:       {}\n", query));
                            out("  Access:      Public (None)\n\n");
                        }
                    }
                }

                if found_count == 0 {
                    out(&format!(
                        "[GoldSrc.rs] Command '{}' not found in any active plugin.\nType 'grs cmds' to list all registered commands.\n",
                        query
                    ));
                }
                return;
            }

            let total_cmds: usize = plugins
                .iter()
                .filter_map(|p| {
                    p.metadata.as_ref().map(|m| {
                        if !m.command_defs.is_empty() {
                            m.command_defs.len()
                        } else {
                            m.commands.len()
                        }
                    })
                })
                .sum();
            out(&format!(
                "--- Registered Plugin Commands ({}) ---\n",
                total_cmds
            ));
            if total_cmds == 0 {
                out("  (No commands registered)\n");
            } else {
                for p in plugins {
                    if let Some(meta) = &p.metadata {
                        if !meta.command_defs.is_empty() {
                            let desc = if !meta.description.is_empty() {
                                format!(" - {}", meta.description)
                            } else {
                                String::new()
                            };
                            out(&format!(
                                "\n[{name} v{ver}]{desc}\n",
                                name = meta.name,
                                ver = meta.version
                            ));
                            for cmd in &meta.command_defs {
                                let usage_or_name = if !cmd.usage.is_empty() {
                                    &cmd.usage
                                } else {
                                    &cmd.name
                                };
                                let aliases_str = if !cmd.aliases.is_empty() {
                                    format!(" (aliases: {})", cmd.aliases.join(", "))
                                } else {
                                    String::new()
                                };
                                out(&format!("  * {}{}\n", usage_or_name, aliases_str));
                                if !cmd.description.is_empty() || cmd.capability.is_some() {
                                    let cap_str = if let Some(ref cap) = cmd.capability {
                                        format!(" [requires: {}]", cap)
                                    } else {
                                        String::new()
                                    };
                                    out(&format!("    {}{}\n", cmd.description, cap_str));
                                }
                            }
                        } else if !meta.commands.is_empty() {
                            let desc = if !meta.description.is_empty() {
                                format!(" - {}", meta.description)
                            } else {
                                String::new()
                            };
                            out(&format!(
                                "\n[{name} v{ver}]{desc}\n",
                                name = meta.name,
                                ver = meta.version
                            ));
                            for cmd in &meta.commands {
                                out(&format!("  * {}\n", cmd));
                            }
                        }
                    }
                }
            }
        }
        "cmd" => {
            let mut positional = Vec::new();
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('h') | Arg::Long("help") => {
                        print_command_help(spec, out);
                        return;
                    }
                    Arg::Value(v) => {
                        positional.push(v.to_string_lossy().to_string());
                    }
                    _ => {}
                }
            }
            if positional.is_empty() {
                print_command_help(spec, out);
            } else {
                let cmd_name = &positional[0];
                let cmd_args = positional[1..].join(" ");
                let handled = manager
                    .map(|m| m.dispatch_command(cmd_name, 0, &cmd_args))
                    .unwrap_or(false);
                if !handled {
                    out(&format!(
                        "[GoldSrc.rs] Command '{}' was not handled by any plugin.\n",
                        cmd_name
                    ));
                }
            }
        }
        "version" => {
            while let Ok(Some(arg)) = parser.next() {
                if let Arg::Short('h') | Arg::Long("help") = arg {
                    print_command_help(spec, out);
                    return;
                }
            }
            out(&format!(
                "[GoldSrc.rs] meta-rs v{} (git: {}, target: {})\n",
                pkg_version, git_hash, build_target
            ));
        }
        _ => {
            print_host_help(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_command_spec() {
        let list_spec = find_command_spec("list").expect("list spec not found");
        assert_eq!(list_spec.name, "list");
        assert!(list_spec.matches("ls"));
        assert!(list_spec.matches("PS"));

        let help_spec = find_command_spec("help").expect("help spec not found");
        assert!(help_spec.matches("?"));
    }

    #[test]
    fn test_print_command_help() {
        let list_spec = find_command_spec("list").unwrap();
        let mut buffer = String::new();
        print_command_help(list_spec, |msg| buffer.push_str(msg));

        assert!(buffer.contains("--- GoldSrc.rs Help: grs list ---"));
        assert!(buffer.contains("-p, --page <N>"));
        assert!(buffer.contains("--paused"));
        assert!(buffer.contains("Examples:"));
        assert!(buffer.contains("grs list -p 2"));
    }

    #[test]
    fn test_print_global_help() {
        let mut buffer = String::new();
        print_host_help(|msg| buffer.push_str(msg));

        assert!(buffer.contains("--- GoldSrc.rs Management CLI ---"));
        assert!(buffer.contains("Plugin Lifecycle:"));
        assert!(buffer.contains("Execution Control:"));
        assert!(buffer.contains("Inspection & Debugging:"));
        assert!(buffer.contains("System:"));
        assert!(buffer.contains("grs help <COMMAND>"));
    }

    #[test]
    fn test_dispatch_command_help() {
        let mut buffer = String::new();
        let args = vec![
            std::ffi::OsString::from("grs"),
            std::ffi::OsString::from("list"),
            std::ffi::OsString::from("--help"),
        ];
        dispatch_host_command(
            args,
            None,
            ("0.11.0", "test", "i686-pc-windows-msvc"),
            |msg| buffer.push_str(msg),
        );
        assert!(buffer.contains("--- GoldSrc.rs Help: grs list ---"));
        assert!(buffer.contains("-p, --page <N>"));
        assert!(buffer.contains("Examples:"));

        let mut buffer2 = String::new();
        let args2 = vec![
            std::ffi::OsString::from("grs"),
            std::ffi::OsString::from("help"),
            std::ffi::OsString::from("reload"),
        ];
        dispatch_host_command(
            args2,
            None,
            ("0.11.0", "test", "i686-pc-windows-msvc"),
            |msg| buffer2.push_str(msg),
        );
        assert!(buffer2.contains("--- GoldSrc.rs Help: grs reload ---"));
        assert!(buffer2.contains("-a, --all"));
    }
}
