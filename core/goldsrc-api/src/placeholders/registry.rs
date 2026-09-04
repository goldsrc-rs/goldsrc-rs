//! In-memory placeholder registry, fluent builder, and guest dispatcher.

use crate::client::Player;
use crate::placeholders::{
    PlaceholderCall, PlaceholderHandler, PlaceholderMetadata, parse_placeholder_call,
};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

/// Thread-safe in-memory placeholder registry.
#[derive(Default)]
pub struct PlaceholderRegistry {
    handlers: HashMap<String, (PlaceholderMetadata, Arc<dyn PlaceholderHandler>)>,
}

impl PlaceholderRegistry {
    /// Creates a new empty placeholder registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers placeholder metadata and an execution handler.
    pub fn register(
        &mut self,
        metadata: PlaceholderMetadata,
        handler: Arc<dyn PlaceholderHandler>,
    ) {
        let name_lower = metadata.name.to_ascii_lowercase();
        for alias in &metadata.aliases {
            self.handlers.insert(
                alias.to_ascii_lowercase(),
                (metadata.clone(), handler.clone()),
            );
        }
        self.handlers.insert(name_lower, (metadata, handler));
    }

    /// Resolves and evaluates a placeholder call.
    pub fn dispatch(&self, name: &str, caller: Player, param: &str) -> Option<String> {
        let clean_name = name.to_ascii_lowercase();
        let (meta, handler) = self.handlers.get(&clean_name)?;

        // Capability check if configured
        if let Some(cap) = &meta.capability
            && !caller.has_capability(cap)
        {
            return None;
        }

        let raw_expr = if param.is_empty() {
            name.to_string()
        } else {
            format!("{name}({param})")
        };

        let call = parse_placeholder_call(&raw_expr).ok()?;
        Some(handler.evaluate(caller, &call))
    }

    /// Clears all registered placeholders.
    pub fn clear(&mut self) {
        self.handlers.clear();
    }
}

static GLOBAL_REGISTRY: LazyLock<RwLock<PlaceholderRegistry>> =
    LazyLock::new(|| RwLock::new(PlaceholderRegistry::default()));

/// Registers a placeholder in the global registry.
pub fn register_placeholder<F>(name: &str, description: &str, handler: F)
where
    F: Fn(Player, &PlaceholderCall) -> String + Send + Sync + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_register_placeholder(name, description);
    }
    let meta = PlaceholderMetadata {
        name: name.to_string(),
        description: description.to_string(),
        usage: format!("{{{name}}}"),
        aliases: Vec::new(),
        capability: None,
    };
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .register(meta, Arc::new(handler));
}

/// Resolves a placeholder through the global registry.
pub fn dispatch_local_placeholder(name: &str, caller_idx: i32, param: &str) -> Option<String> {
    let caller = Player::new(caller_idx);
    GLOBAL_REGISTRY
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .dispatch(name, caller, param)
}

/// Clears all placeholders in the global registry.
pub fn clear_placeholders() {
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
}

/// Fluent builder for dynamic placeholders.
#[derive(Debug, Clone)]
pub struct PlaceholderBuilder {
    metadata: PlaceholderMetadata,
}

impl PlaceholderBuilder {
    /// Creates a new placeholder builder with the primary identifier name.
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            metadata: PlaceholderMetadata {
                name: name.clone(),
                description: String::new(),
                usage: format!("{{{name}}}"),
                aliases: Vec::new(),
                capability: None,
            },
        }
    }

    /// Sets human-readable description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.metadata.description = desc.into();
        self
    }

    /// Sets usage example syntax.
    pub fn usage(mut self, usage: impl Into<String>) -> Self {
        self.metadata.usage = usage.into();
        self
    }

    /// Adds an alias.
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.metadata.aliases.push(alias.into());
        self
    }

    /// Sets required capability.
    pub fn capability(mut self, cap: impl Into<String>) -> Self {
        self.metadata.capability = Some(cap.into());
        self
    }

    /// Registers the placeholder with an evaluation handler.
    pub fn register<F>(self, handler: F)
    where
        F: Fn(Player, &PlaceholderCall) -> String + Send + Sync + 'static,
    {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_register_placeholder(
                &self.metadata.name,
                &self.metadata.description,
            );
        }
        GLOBAL_REGISTRY
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .register(self.metadata, Arc::new(handler));
    }
}

/// Placeholder entry point.
pub struct Placeholder;

impl Placeholder {
    /// Starts building a new placeholder definition.
    pub fn builder(name: impl Into<String>) -> PlaceholderBuilder {
        PlaceholderBuilder::new(name)
    }

    /// Registers a simple placeholder with description and handler.
    pub fn register<F>(name: &str, description: &str, handler: F)
    where
        F: Fn(Player, &PlaceholderCall) -> String + Send + Sync + 'static,
    {
        register_placeholder(name, description, handler);
    }
}
