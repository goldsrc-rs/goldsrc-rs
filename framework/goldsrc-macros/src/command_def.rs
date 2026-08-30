//! Definitions and parsing for command attributes.

/// Information about a registered command definition.
#[derive(Debug, Clone)]
pub struct CommandDefInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub aliases: Vec<String>,
    pub capability: Option<String>,
}
