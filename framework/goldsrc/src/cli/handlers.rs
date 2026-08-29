//! Handlers for specific CLI commands.

use crate::cli::specs::{CommandSpec, print_command_help};
use goldsrc_wasm_host::PluginManager;
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
        plugins.retain(|p| matches!(p.status, goldsrc_wasm_host::PluginStatus::Paused { .. }));
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
