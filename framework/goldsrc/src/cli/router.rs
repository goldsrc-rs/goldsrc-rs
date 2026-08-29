//! Router and dispatcher for host server commands.

use crate::cli::handlers;
use crate::cli::specs::{find_command_spec, print_command_help, print_host_help};
use goldsrc_wasm_host::PluginManager;
use lexopt::Arg;

pub fn dispatch_host_command<F: FnMut(&str)>(
    raw_args: Vec<std::ffi::OsString>,
    manager: Option<&mut PluginManager>,
    version_info: (&str, &str, &str),
    mut out: F,
) {
    let (pkg_version, git_hash, build_target) = version_info;

    let mut parser = lexopt::Parser::from_args(raw_args);
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

    match spec.name {
        "list" => handlers::handle_list(spec, parser, manager, out),
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
            let mut cmd_name = None;
            let mut cmd_args = Vec::new();

            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Arg::Short('h') | Arg::Long("help") => {
                        print_command_help(spec, out);
                        return;
                    }
                    Arg::Value(val) => {
                        let s = val.to_string_lossy().into_owned();
                        if cmd_name.is_none() {
                            cmd_name = Some(s);
                        } else {
                            cmd_args.push(s);
                        }
                    }
                    _ => {}
                }
            }

            let Some(name) = cmd_name else {
                print_command_help(spec, out);
                return;
            };

            let Some(manager) = manager else {
                out("[GoldSrc.rs] Error: WASM Host not initialized.\n");
                return;
            };

            let args_str = cmd_args.join(" ");
            let handled = manager.dispatch_command(&name, 0, &args_str);
            if !handled {
                out(&format!(
                    "[GoldSrc.rs] Command '{}' was not handled by any active plugin.\n",
                    name
                ));
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
                "GoldSrc.rs Host v{} (git: {})\nBuilt for target: {}\n",
                pkg_version, git_hash, build_target
            ));
        }
        _ => {
            print_host_help(out);
        }
    }
}
