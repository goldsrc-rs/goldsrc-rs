//! Handlers for specific CLI commands.

use crate::cli::specs::{CommandSpec, print_command_help};
use goldsrc_host_wasm::PluginManager;
use lexopt::Arg;

pub fn handle_list<F: FnMut(&str)>(
    spec: &CommandSpec,
    mut parser: lexopt::Parser,
    manager: Option<&mut PluginManager>,
    mut out: F,
) {
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
        plugins.retain(|p| matches!(p.status, goldsrc_host_wasm::PluginStatus::Paused { .. }));
    }

    let total_plugins = plugins.len();
    if plugins.is_empty() {
        out("[GoldSrc.rs] WASM plugins (0):\n  (No plugins found)\n");
        return;
    }

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

        let mut root_plugins = Vec::new();
        let mut bundle_groups: std::collections::BTreeMap<
            String,
            Vec<&goldsrc_host_wasm::PluginInfo>,
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

        for (bundle_name, bundle_plugins) in bundle_groups {
            out(&format!(
                "  [{}] ({} plugins)\n",
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

/// Handles `grs watchers list` inspection command.
pub fn handle_watchers_list<F: FnMut(&str)>(
    spec: &CommandSpec,
    mut parser: lexopt::Parser,
    mut out: F,
) {
    let mut as_json = false;
    while let Ok(Some(arg)) = parser.next() {
        match arg {
            Arg::Short('h') | Arg::Long("help") => {
                print_command_help(spec, out);
                return;
            }
            Arg::Long("json") => as_json = true,
            _ => {}
        }
    }

    let watchers = crate::HostRuntime::with_watcher_service(|w| {
        w.map(|service| service.list_watchers()).unwrap_or_default()
    });

    if as_json {
        let json = serde_json::to_string_pretty(&watchers).unwrap_or_else(|_| "[]".into());
        out(&json);
        out("\n");
        return;
    }

    if watchers.is_empty() {
        out("[GoldSrc.rs] Watchers (0):\n  (No registered filesystem watchers)\n");
        return;
    }

    out(&format!(
        "[GoldSrc.rs] Filesystem Watchers ({} registered):\n",
        watchers.len()
    ));
    for w in &watchers {
        let status_str = if w.is_paused { "[PAUSED]" } else { "[ACTIVE]" };
        let rec_str = if w.recursive { " (recursive)" } else { "" };
        out(&format!(
            "  - {:<16} {:<8} debounce: {}ms\n      target: {} \"{}\"{} (filter: {})\n",
            w.id,
            status_str,
            w.debounce_ms,
            w.target_type,
            crate::paths::PathResolver::normalize(&w.path),
            rec_str,
            w.filter_desc
        ));
    }
}

/// Handles `grs watchers pause <id>` command.
pub fn handle_watchers_pause<F: FnMut(&str)>(id: &str, mut out: F) {
    let res = crate::HostRuntime::with_watcher_service(|w| w.map(|s| s.pause(id)));
    match res {
        Some(true) => out(&format!("[GoldSrc.rs] Watcher '{id}' is now PAUSED.\n")),
        Some(false) => out(&format!("[GoldSrc.rs] Error: Watcher '{id}' not found.\n")),
        None => out("[GoldSrc.rs] Error: Watcher service not available.\n"),
    }
}

/// Handles `grs watchers resume <id>` command.
pub fn handle_watchers_resume<F: FnMut(&str)>(id: &str, mut out: F) {
    let res = crate::HostRuntime::with_watcher_service(|w| w.map(|s| s.resume(id)));
    match res {
        Some(true) => out(&format!("[GoldSrc.rs] Watcher '{id}' is now ACTIVE.\n")),
        Some(false) => out(&format!("[GoldSrc.rs] Error: Watcher '{id}' not found.\n")),
        None => out("[GoldSrc.rs] Error: Watcher service not available.\n"),
    }
}

/// Handles `grs plugins reload` and `grs rld` command.
pub fn handle_reload<F: FnMut(&str)>(
    spec: &CommandSpec,
    mut parser: lexopt::Parser,
    manager: Option<&mut PluginManager>,
    mut out: F,
) {
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
        out(&crate::cli::CliResponse::error("WASM Host not initialized.").format_console());
        return;
    };
    if all {
        let msg = manager.reload_all_plugins();
        out(&crate::cli::CliResponse::success(msg).format_console());
    } else if !targets.is_empty() {
        for t in targets {
            match manager.reload_plugin_by_query(&t) {
                Ok(msg) => out(
                    &crate::cli::CliResponse::success(format!("{msg} successfully."))
                        .format_console(),
                ),
                Err(err) => out(&crate::cli::CliResponse::error(err.to_string()).format_console()),
            }
        }
    } else {
        out("[GoldSrc.rs] Usage: grs plugins reload <name|index...> [-a|--all]\n");
    }
}

pub fn handle_extensions<F: FnMut(&str)>(
    spec: &CommandSpec,
    mut parser: lexopt::Parser,
    mut out: F,
) {
    let mut subcommand = None;
    let mut target_name = None;

    while let Ok(Some(arg)) = parser.next() {
        match arg {
            Arg::Short('h') | Arg::Long("help") => {
                print_command_help(spec, out);
                return;
            }
            Arg::Value(val) => {
                let s = val.to_string_lossy().to_string();
                if subcommand.is_none() {
                    subcommand = Some(s);
                } else if target_name.is_none() {
                    target_name = Some(s);
                }
            }
            _ => {}
        }
    }

    let sub = subcommand.as_deref().unwrap_or("list");
    let registry = crate::extension::extension_registry();

    match sub {
        "list" | "ls" => {
            let mut extensions = registry.all();
            extensions.sort_by(|a, b| a.name().cmp(b.name()));
            if extensions.is_empty() {
                out("[GoldSrc.rs] Engine Extensions (0):\n  (No extensions registered)\n");
                return;
            }
            out(&format!(
                "[GoldSrc.rs] Engine Extensions ({}):\n",
                extensions.len()
            ));
            out(&format!(
                "  {:<12} {:<10} {:<10} {}\n",
                "NAME", "STATUS", "VERSION", "DESCRIPTION"
            ));
            out(&format!(
                "  {:-<12} {:-<10} {:-<10} {:-<30}\n",
                "", "", "", ""
            ));
            for ext in extensions {
                let status_str = if ext.is_available() {
                    "ACTIVE"
                } else {
                    "DISABLED"
                };
                out(&format!(
                    "  {:<12} {:<10} {:<10} {}\n",
                    ext.name(),
                    status_str,
                    ext.version(),
                    ext.description()
                ));
            }
        }
        "info" => {
            let Some(name) = target_name else {
                out("[GoldSrc.rs] Usage: grs extensions info <name>\n");
                return;
            };
            if let Some(ext) = registry.get(&name) {
                let status_str = if ext.is_available() {
                    "ACTIVE"
                } else {
                    "DISABLED"
                };
                out(&format!("[GoldSrc.rs] Extension: {}\n", ext.name()));
                out(&format!("  Name:        {}\n", ext.name()));
                out(&format!("  Status:      {}\n", status_str));
                out(&format!("  Version:     {}\n", ext.version()));
                out(&format!("  Description: {}\n", ext.description()));
            } else {
                out(&format!(
                    "[GoldSrc.rs] Error: Engine extension '{}' not found.\n",
                    name
                ));
            }
        }
        unknown => {
            out(&format!(
                "[GoldSrc.rs] Unknown extensions subcommand '{}'. Valid: list, info.\n",
                unknown
            ));
        }
    }
}

pub fn handle_hardware<F: FnMut(&str)>(spec: &CommandSpec, mut parser: lexopt::Parser, mut out: F) {
    let mut json_output = false;
    while let Ok(Some(arg)) = parser.next() {
        match arg {
            Arg::Short('h') | Arg::Long("help") => {
                print_command_help(spec, out);
                return;
            }
            Arg::Long("json") => {
                json_output = true;
            }
            _ => {}
        }
    }

    let snapshot = crate::hardware::system_info().sample_metrics();

    if json_output {
        let json = serde_json::json!({
            "cpu": {
                "brand": snapshot.cpu_brand,
                "vendor": snapshot.cpu_vendor,
                "physical_cores": snapshot.physical_cores,
                "logical_cores": snapshot.logical_cores,
                "global_usage_pct": snapshot.global_cpu_usage,
                "process_usage_pct": snapshot.process_cpu_usage
            },
            "memory": {
                "process_rss_mb": snapshot.process_memory_rss_bytes as f64 / (1024.0 * 1024.0),
                "process_virtual_mb": snapshot.process_memory_virtual_bytes as f64 / (1024.0 * 1024.0),
                "total_ram_mb": snapshot.total_memory_bytes as f64 / (1024.0 * 1024.0),
                "available_ram_mb": snapshot.available_memory_bytes as f64 / (1024.0 * 1024.0),
                "total_swap_mb": snapshot.total_swap_bytes as f64 / (1024.0 * 1024.0),
                "used_swap_mb": snapshot.used_swap_bytes as f64 / (1024.0 * 1024.0)
            },
            "performance": {
                "measured_fps": snapshot.measured_fps,
                "jitter_ms": snapshot.frame_jitter_ms,
                "uptime_secs": snapshot.uptime_secs
            }
        });
        out(&format!("{}\n", json));
        return;
    }

    let rss_mb = snapshot.process_memory_rss_bytes as f64 / (1024.0 * 1024.0);
    let total_ram_mb = snapshot.total_memory_bytes as f64 / (1024.0 * 1024.0);
    let avail_ram_mb = snapshot.available_memory_bytes as f64 / (1024.0 * 1024.0);
    let phys_str = snapshot
        .physical_cores
        .map(|c| c.to_string())
        .unwrap_or_else(|| "N/A".to_string());

    out("====================================================\n");
    out("       GoldSrc.rs Host Hardware & Diagnostics       \n");
    out("====================================================\n");
    out(&format!(
        "  CPU Model     : {}\n",
        snapshot.cpu_brand.trim()
    ));
    out(&format!("  Vendor / Arch : {}\n", snapshot.cpu_vendor));
    out(&format!(
        "  Cores / Threads: {} physical / {} logical\n",
        phys_str, snapshot.logical_cores
    ));
    out(&format!(
        "  Host CPU Load : {:.1}%\n",
        snapshot.global_cpu_usage
    ));
    out(&format!(
        "  HLDS CPU Load : {:.1}%\n",
        snapshot.process_cpu_usage
    ));
    out("----------------------------------------------------\n");
    out(&format!("  HLDS Process  : {:.1} MB RSS\n", rss_mb));
    out(&format!(
        "  Host RAM Free : {:.0} MB / {:.0} MB total ({:.1}% free)\n",
        avail_ram_mb,
        total_ram_mb,
        (avail_ram_mb / total_ram_mb.max(1.0)) * 100.0
    ));
    out("----------------------------------------------------\n");
    let fps_display = if snapshot.measured_fps > 0.0 {
        format!("{:.1} FPS", snapshot.measured_fps)
    } else {
        "Calibrating...".to_string()
    };
    out(&format!("  Engine Rate   : {}\n", fps_display));
    out(&format!(
        "  Frame Jitter  : {:.2} ms\n",
        snapshot.frame_jitter_ms
    ));
    out(&format!("  Host Uptime   : {}s\n", snapshot.uptime_secs));
    out("====================================================\n");
}
