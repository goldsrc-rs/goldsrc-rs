//! Host CLI dispatch and formatting logic for GoldSrc.rs.

use goldsrc_wasm_host::PluginManager;
use lexopt::Arg;
use std::ffi::{c_char, CStr, OsString};
use std::sync::OnceLock;

/// Backend accessors needed to run the host CLI as a server command.
///
/// The backend provides C-compatible argv access, a live handle to its
/// [`PluginManager`], an output callback and version metadata. Shared by both
/// backends so the `meta-rs`/`mrs` command set behaves identically.
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

/// Shared server-command handler for `meta-rs` / `mrs`.
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
            if !arg_ptr.is_null() {
                if let Ok(cstr) = CStr::from_ptr(arg_ptr).to_str() {
                    raw_args.push(OsString::from(cstr));
                }
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

/// Register server commands pointing at the shared handler.
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

pub fn print_host_help<F: FnMut(&str)>(mut out: F) {
    out("--- GoldSrc.rs Management CLI ---\n");
    out("Usage: grs <COMMAND> [OPTIONS] [TARGET]\n");
    out("Aliases: goldsrc-rs, mrs, meta-rs\n\n");
    out("Commands:\n");
    out("  Plugin Lifecycle:\n");
    out("    load <file>             Load a WASM plugin from plugins/ directory\n");
    out("    unload <target>         Gracefully unload plugin(s) (-a, --all supported)\n");
    out("    reload <target>         Reload plugin(s) (-a, --all supported)\n\n");
    out("  Execution Control:\n");
    out("    pause <target>          Suspend plugin execution (-a, --all supported)\n");
    out("    unpause <target>        Resume plugin execution (-a, --all supported)\n\n");
    out("  Inspection & Debugging:\n");
    out("    list [OPTIONS]          List loaded plugins. Options:\n");
    out("                              -p, --page <N>    Show specific page (default: 1)\n");
    out("                              -s, --size <N>    Set page size (default: 5)\n");
    out("                              -a, --all         Show all plugins (ignore pagination)\n");
    out("                              --paused          Show only paused plugins\n");
    out("    info <target>           Show detailed metadata and exports\n\n");
    out("  System:\n");
    out("    status                  Show host runtime stats (RAM, watchers, count)\n");
    out("    version                 Show host version info\n\n");
    out("Options:\n");
    out("  -h, --help                Print this help message\n");
    out("  -a, --all                 Target all loaded plugins (for unload/reload/pause)\n");
}

pub fn dispatch_host_command<F: FnMut(&str)>(
    raw_args: Vec<std::ffi::OsString>,
    manager: Option<&mut PluginManager>,
    version_info: (&str, &str, &str),
    mut out: F,
) {
    let (pkg_version, git_hash, build_target) = version_info;

    let manager = match manager {
        Some(m) => m,
        None => {
            out("[GoldSrc.rs] Error: WASM Host not initialized.\n");
            return;
        }
    };

    let mut parser = lexopt::Parser::from_args(raw_args);
    // Skip binary name ("mrs" or "meta-rs")
    let _ = parser.next();

    let command = match parser.next() {
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

    match command.as_str() {
        "list" => {
            let mut page: usize = 1;
            let mut size: Option<usize> = None;
            let mut only_paused = false;
            let mut all = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
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
                    Arg::Long("paused") => only_paused = true,
                    _ => {}
                }
            }

            let mut plugins = manager.get_plugins_info();
            if only_paused {
                plugins.retain(|p| p.is_paused);
            }

            let total_plugins = plugins.len();
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

            if plugins.is_empty() {
                out("  (No plugins found)\n");
                return;
            }

            let start = (page_idx * page_size).min(total_plugins);
            let end = (start + page_size).min(total_plugins);

            for p in &plugins[start..end] {
                let status = if p.is_paused { "PAUSED" } else { "RUNNING" };
                let mut exports = Vec::new();
                if p.has_on_load {
                    exports.push("on_load");
                }
                if p.has_on_unload {
                    exports.push("on_unload");
                }
                if p.has_on_frame {
                    exports.push("on_frame");
                }
                let exports_str = if exports.is_empty() {
                    "none".to_string()
                } else {
                    exports.join(", ")
                };
                out(&format!(
                    "  [#{}] {:<20} | Status: {:<7} | Exports: {}\n",
                    p.index, p.name, status, exports_str
                ));
            }
        }
        "load" => {
            let mut targets = Vec::new();
            while let Ok(Some(arg)) = parser.next() {
                if let Arg::Value(val) = arg {
                    targets.push(val.to_string_lossy().into_owned());
                }
            }
            if targets.is_empty() {
                out("Usage: grs load <file1> [file2...]\n");
                return;
            }
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
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Value(val) => targets.push(val.to_string_lossy().into_owned()),
                    _ => {}
                }
            }
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
                out("Usage: grs unload <name|index...> or grs unload -a/--all\n");
            }
        }
        "reload" => {
            let mut targets = Vec::new();
            let mut all = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Value(val) => targets.push(val.to_string_lossy().into_owned()),
                    _ => {}
                }
            }
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
                out("Usage: grs reload <name|index...> or grs reload -a/--all\n");
            }
        }
        "pause" => {
            let mut targets = Vec::new();
            let mut all = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Value(val) => targets.push(val.to_string_lossy().into_owned()),
                    _ => {}
                }
            }
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
                out("Usage: grs pause <name|index...> or grs pause -a/--all\n");
            }
        }
        "unpause" => {
            let mut targets = Vec::new();
            let mut all = false;
            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('a') | Arg::Long("all") => all = true,
                    Arg::Value(val) => targets.push(val.to_string_lossy().into_owned()),
                    _ => {}
                }
            }
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
                out("Usage: grs unpause <name|index...> or grs unpause -a/--all\n");
            }
        }
        "info" => {
            let mut targets = Vec::new();
            while let Ok(Some(arg)) = parser.next() {
                if let Arg::Value(val) = arg {
                    targets.push(val.to_string_lossy().into_owned());
                }
            }
            if targets.is_empty() {
                out("Usage: grs info <name|index...>\n");
                return;
            }
            for t in targets {
                if let Some(idx) = manager.find_plugin(&t) {
                    let info = &manager.get_plugins_info()[idx];
                    let clean_path = crate::paths::PathResolver::normalize(&info.path);
                    out(&format!("--- Plugin Info: {} ---\n", info.name));
                    out(&format!("  Index:        #{}\n", info.index));
                    out(&format!("  Path:         \"{}\"\n", clean_path));
                    out(&format!(
                        "  Status:       {}\n",
                        if info.is_paused { "Paused" } else { "Running" }
                    ));
                    if let Some(meta) = &info.metadata {
                        out(&format!("  Meta Name:    {}\n", meta.name));
                        out(&format!("  Version:      {}\n", meta.version));
                        let systems_str = if meta.systems.is_empty() {
                            "none".to_string()
                        } else {
                            meta.systems.join(", ")
                        };
                        out(&format!("  Systems:      {}\n", systems_str));
                        let deps_str = if meta.dependencies.is_empty() {
                            "none".to_string()
                        } else {
                            meta.dependencies
                                .iter()
                                .map(|(k, v)| format!("{} ({})", k, v))
                                .collect::<Vec<_>>()
                                .join(", ")
                        };
                        out(&format!("  Dependencies: {}\n", deps_str));
                    }
                    out(&format!("  on_load:      {}\n", info.has_on_load));
                    out(&format!("  on_unload:    {}\n", info.has_on_unload));
                    out(&format!("  on_frame:     {}\n", info.has_on_frame));
                } else {
                    out(&format!("[GoldSrc.rs] Plugin '{}' not found.\n", t));
                }
            }
        }
        "status" => {
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
        "version" => {
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
