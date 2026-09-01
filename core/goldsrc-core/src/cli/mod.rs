//! Host CLI dispatch, C-ABI bindings, and declarative commands for GoldSrc.rs.

pub mod handlers;
pub mod router;
pub mod specs;

pub use router::dispatch_host_command;
pub use specs::{
    BUILTIN_COMMANDS, CommandSpec, find_command_spec, print_command_help, print_host_help,
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
        crate::host::HostRuntime::with_manager(|manager| {
            dispatch_host_command(raw_args, manager, backend.version, backend.print);
        });
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
        assert!(find_command_spec("list").is_some());
        assert!(find_command_spec("ls").is_some());
        assert!(find_command_spec("ps").is_some());
        assert!(find_command_spec("reload").is_some());
        assert!(find_command_spec("nonexistent").is_none());
    }

    #[test]
    fn test_print_command_help() {
        let spec = find_command_spec("list").unwrap();
        let mut output = String::new();
        print_command_help(spec, |s| output.push_str(s));
        assert!(output.contains("grs list"));
        assert!(output.contains("--flat"));
        assert!(output.contains("--paused"));
    }

    #[test]
    fn test_print_global_help() {
        let mut output = String::new();
        print_host_help(|s| output.push_str(s));
        assert!(output.contains("GoldSrc.rs Management CLI"));
        assert!(output.contains("Plugin Lifecycle:"));
        assert!(output.contains("Execution Control:"));
        assert!(output.contains("Inspection & Debugging:"));
        assert!(output.contains("System:"));
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
            OsString::from("reload"),
        ];
        dispatch_host_command(args_cmd, None, ("0.10.0", "abc", "x86"), |s| {
            output_cmd.push_str(s)
        });
        assert!(output_cmd.contains("grs reload"));
    }
}
