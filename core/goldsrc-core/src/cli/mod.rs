//! Host CLI dispatch, C-ABI bindings, and declarative commands for GoldSrc.rs.

pub mod handlers;
pub mod response;
pub mod router;
pub mod specs;

pub use response::{CliResponse, CommandStatus};
pub use router::dispatch_host_command;
pub use specs::{
    BUILTIN_CATEGORIES, BUILTIN_COMMANDS, CommandSpec, find_command_spec, print_category_help,
    print_command_help, print_host_help,
};

use std::ffi::{CStr, OsString, c_char};
use std::sync::OnceLock;

/// Backend accessors needed to run the host CLI as a server command.
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
    goldsrc_sys::ffi::catch_ffi_panic("handle_host_command", (), || {
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
        // Dispatch directly; commands requiring PluginManager will acquire it with
        // narrow scope, preventing re-entrant deadlocks with WatcherService or HostRuntime.
        dispatch_host_command(raw_args, None, backend.version, backend.print);
    });
}

/// Shared server-command handler for WASM plugin commands.
///
/// # Safety
/// Registered as a C server command via `pfnAddServerCommand`.
pub unsafe extern "C" fn handle_plugin_server_command() {
    goldsrc_sys::ffi::catch_ffi_panic("handle_plugin_server_command", (), || {
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

        let mut args = String::new();
        for i in 1..argc {
            let arg_ptr = (backend.argv)(i);
            if !arg_ptr.is_null()
                && let Ok(cstr) = unsafe { CStr::from_ptr(arg_ptr) }.to_str()
            {
                if !args.is_empty() {
                    args.push(' ');
                }
                args.push_str(cstr);
            }
        }

        crate::host::HostRuntime::with_manager(|manager| {
            if let Some(m) = manager {
                m.dispatch_command(cmd_name, 0, &args);
            }
        });
    });
}

/// Register server commands pointing at the shared host CLI handler.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_command_spec() {
        assert!(find_command_spec("plugins").is_some());
        assert!(find_command_spec("pl").is_some());
        assert!(find_command_spec("p").is_some());
        assert!(find_command_spec("ps").is_some());
        assert!(find_command_spec("rld").is_some());
        assert!(find_command_spec("reload").is_some());
        assert!(find_command_spec("watchers").is_some());
        assert!(find_command_spec("watch").is_some());
        assert!(find_command_spec("w").is_some());
        assert!(find_command_spec("cmd").is_some());
        assert!(find_command_spec("exec").is_some());
        assert!(find_command_spec("c").is_some());
        assert!(find_command_spec("status").is_some());
        assert!(find_command_spec("st").is_some());
        assert!(find_command_spec("s").is_some());
        assert!(find_command_spec("version").is_some());
        assert!(find_command_spec("ver").is_some());
        assert!(find_command_spec("v").is_some());
        assert!(find_command_spec("help").is_some());
        assert!(find_command_spec("nonexistent").is_none());
        assert!(find_command_spec("foobar_xyz").is_none());
    }

    #[test]
    fn test_print_command_help() {
        let spec = find_command_spec("plugins").unwrap();
        let mut output = String::new();
        print_command_help(spec, |s| output.push_str(s));
        assert!(output.contains("grs plugins"));
        assert!(output.contains("list"));
        assert!(output.contains("--flat"));
        assert!(output.contains("--paused"));

        let watcher_spec = find_command_spec("watchers").unwrap();
        let mut watcher_output = String::new();
        print_command_help(watcher_spec, |s| watcher_output.push_str(s));
        assert!(watcher_output.contains("grs watchers"));
        assert!(watcher_output.contains("list"));
        assert!(watcher_output.contains("pause"));
        assert!(watcher_output.contains("resume"));
    }

    #[test]
    fn test_print_global_help() {
        let mut output = String::new();
        print_host_help(|s| output.push_str(s));
        assert!(output.contains("GoldSrc.rs Management CLI"));
        assert!(output.contains("[plugin:lifecycle]"));
        assert!(output.contains("[watcher:fs]"));
        assert!(output.contains("[exec:dispatch]"));
        assert!(output.contains("[sys:runtime]"));
        assert!(output.contains("[sys:help]"));

        let mut cat_output = String::new();
        let matched = print_category_help("plugin", |s| cat_output.push_str(s));
        assert!(matched);
        assert!(cat_output.contains("plugins"));
        assert!(cat_output.contains("plugin:lifecycle"));
    }

    #[test]
    fn test_dispatch_command_help() {
        let mut output = String::new();
        let args = vec![OsString::from("grs"), OsString::from("--help")];
        dispatch_host_command(args, None, ("0.10.0", "abc", "x86"), |s| output.push_str(s));
        assert!(output.contains("GoldSrc.rs Management CLI"));

        let mut output_cmd = String::new();
        let args_cmd = vec![
            OsString::from("grs"),
            OsString::from("help"),
            OsString::from("plugins"),
        ];
        dispatch_host_command(args_cmd, None, ("0.10.0", "abc", "x86"), |s| {
            output_cmd.push_str(s)
        });
        assert!(output_cmd.contains("grs plugins"));
    }

    #[test]
    fn test_levenshtein_and_command_suggestions() {
        use router::{levenshtein_distance, suggest_command, suggest_subcommand};

        assert_eq!(levenshtein_distance("plugin", "plugins"), 1);
        assert_eq!(levenshtein_distance("plguins", "plugins"), 2);
        assert_eq!(levenshtein_distance("watchr", "watchers"), 2);
        assert_eq!(levenshtein_distance("completely_different", "plugins"), 16);
        assert_eq!(
            levenshtein_distance(
                "this_is_a_very_long_string_that_exceeds_thirty_two_chars",
                "plugins"
            ),
            usize::MAX
        );

        assert_eq!(suggest_command("plugin"), Some("plugins"));
        assert_eq!(suggest_command("plguins"), Some("plugins"));
        assert_eq!(suggest_command("watcher"), Some("watchers"));
        assert_eq!(suggest_command("unknown_xyz"), None);

        let subcmds = &[
            "list", "info", "load", "unload", "reload", "pause", "unpause",
        ];
        assert_eq!(suggest_subcommand("reloadd", subcmds), Some("reload"));
        assert_eq!(suggest_subcommand("paws", subcmds), Some("pause")); // distance is 2
        assert_eq!(suggest_subcommand("completely_unrelated", subcmds), None);

        let mut output = String::new();
        let args_typo = vec![OsString::from("grs"), OsString::from("plugin")];
        dispatch_host_command(args_typo, None, ("0.10.0", "abc", "x86"), |s| {
            output.push_str(s)
        });
        assert!(output.contains("Unknown command 'plugin'. Did you mean 'plugins'?"));
    }

    #[test]
    fn test_shortcuts_and_watchers_dispatch() {
        // Test `grs watchers` dispatch with None manager (should not deadlock or error)
        let mut out_watchers = String::new();
        let args_watchers = vec![OsString::from("grs"), OsString::from("watchers")];
        dispatch_host_command(args_watchers, None, ("0.10.0", "abc", "x86"), |s| {
            out_watchers.push_str(s)
        });
        assert!(out_watchers.contains("Watchers (0)"));

        // Test `grs w` alias
        let mut out_w = String::new();
        let args_w = vec![OsString::from("grs"), OsString::from("w")];
        dispatch_host_command(args_w, None, ("0.10.0", "abc", "x86"), |s| {
            out_w.push_str(s)
        });
        assert!(out_w.contains("Watchers (0)"));

        // Test `grs ps` top-level shortcut
        let mut out_ps = String::new();
        let args_ps = vec![OsString::from("grs"), OsString::from("ps")];
        dispatch_host_command(args_ps, None, ("0.10.0", "abc", "x86"), |s| {
            out_ps.push_str(s)
        });
        assert!(out_ps.contains("WASM Host not initialized.") || out_ps.contains("WASM plugins"));

        // Test `grs pl ps` (plugins alias + ps list subcommand)
        let mut out_pl_ps = String::new();
        let args_pl_ps = vec![
            OsString::from("grs"),
            OsString::from("pl"),
            OsString::from("ps"),
        ];
        dispatch_host_command(args_pl_ps, None, ("0.10.0", "abc", "x86"), |s| {
            out_pl_ps.push_str(s)
        });
        assert!(
            out_pl_ps.contains("WASM Host not initialized.") || out_pl_ps.contains("WASM plugins")
        );

        // Test `grs status`
        let mut out_status = String::new();
        let args_status = vec![OsString::from("grs"), OsString::from("status")];
        dispatch_host_command(args_status, None, ("0.10.0", "abc", "x86"), |s| {
            out_status.push_str(s)
        });
        assert!(out_status.contains("GoldSrc.rs Host Engine Status"));
        assert!(out_status.contains("Watchers:"));
    }
}
