//! Definitions and sorting for ECS systems.

/// Information about a registered ECS system definition.
#[derive(Clone)]
pub struct SystemDefInfo {
    pub stage: String,
    pub phase: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
    pub ident: syn::Ident,
    pub inputs_len: usize,
}
