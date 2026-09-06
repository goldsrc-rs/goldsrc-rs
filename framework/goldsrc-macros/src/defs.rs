//! Structural definitions for plugin attributes, commands, and ECS systems.

/// Parsed `#[plugin(...)]` and stacked helper attributes values.
#[derive(Debug, Clone)]
pub struct PluginAttr {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub url: String,
    pub license: String,
    pub bundle: Option<String>,
    pub requires: Vec<String>,
    pub permissions: Vec<String>,
    pub load_time: String,
    pub unload_time: String,
}

/// Information about a registered command definition.
#[derive(Debug, Clone)]
pub struct CommandDefInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub aliases: Vec<String>,
    pub capability: Option<String>,
    pub requires: Vec<String>,
}

/// Information about a registered ECS system definition.
#[derive(Clone)]
pub struct SystemDefInfo {
    pub stage: String,
    pub phase: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
    pub ident: syn::Ident,
    pub inputs_len: usize,
    pub takes_player: bool,
}
