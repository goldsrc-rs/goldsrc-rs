//! Plugin manifest TOML generator.

use crate::defs::{CommandDefInfo, PluginAttr};
use crate::utils::toml_escape;

/// Generates the GoldSrc WASM plugin manifest TOML string.
pub fn generate_manifest_toml(attr: &PluginAttr, command_defs: &[CommandDefInfo]) -> String {
    let mut requires_toml = String::new();
    if !attr.requires.is_empty() {
        let reqs: Vec<String> = attr
            .requires
            .iter()
            .map(|d| format!("\"{}\"", toml_escape(d)))
            .collect();
        requires_toml = format!("requires = [{}]\n", reqs.join(", "));
    }

    let mut permissions_toml = String::new();
    if !attr.permissions.is_empty() {
        let perms: Vec<String> = attr
            .permissions
            .iter()
            .map(|p| format!("\"{}\"", toml_escape(p)))
            .collect();
        permissions_toml = format!("permissions = [{}]\n", perms.join(", "));
    }

    let mut commands_toml = String::new();
    if !command_defs.is_empty() {
        commands_toml.push('\n');
        for cmd in command_defs {
            commands_toml.push_str("[[commands]]\n");
            commands_toml.push_str(&format!("name = \"{}\"\n", toml_escape(&cmd.name)));
            if !cmd.description.is_empty() {
                commands_toml.push_str(&format!(
                    "description = \"{}\"\n",
                    toml_escape(&cmd.description)
                ));
            }
            if !cmd.usage.is_empty() {
                commands_toml.push_str(&format!("usage = \"{}\"\n", toml_escape(&cmd.usage)));
            }
            if !cmd.aliases.is_empty() {
                let aliases_str: Vec<String> = cmd
                    .aliases
                    .iter()
                    .map(|a| format!("\"{}\"", toml_escape(a)))
                    .collect();
                commands_toml.push_str(&format!("aliases = [{}]\n", aliases_str.join(", ")));
            }
            if let Some(cap) = &cmd.capability {
                commands_toml.push_str(&format!("capability = \"{}\"\n", toml_escape(cap)));
            }
            if !cmd.requires.is_empty() {
                let req_str: Vec<String> = cmd
                    .requires
                    .iter()
                    .map(|r| format!("\"{}\"", toml_escape(r)))
                    .collect();
                commands_toml.push_str(&format!("requires = [{}]\n", req_str.join(", ")));
            }
            commands_toml.push('\n');
        }
    }

    let bundle_field = match &attr.bundle {
        Some(b) => format!("bundle = \"{}\"\n", toml_escape(b)),
        None => String::new(),
    };

    let lifecycle_toml = format!(
        "[lifecycle]\nload = \"{}\"\nunload = \"{}\"\n",
        toml_escape(&attr.load_time),
        toml_escape(&attr.unload_time)
    );

    format!(
        "[plugin]\nname = \"{}\"\nversion = \"{}\"\nauthor = \"{}\"\ndescription = \"{}\"\nurl = \"{}\"\nlicense = \"{}\"\n{}{}{}{}{}",
        toml_escape(&attr.name),
        toml_escape(&attr.version),
        toml_escape(&attr.author),
        toml_escape(&attr.description),
        toml_escape(&attr.url),
        toml_escape(&attr.license),
        bundle_field,
        requires_toml,
        permissions_toml,
        lifecycle_toml,
        commands_toml
    )
}
