//! Registry for menu action callback handlers.

use crate::client::Player;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

/// Type alias for menu action handler closures.
pub type MenuActionHandler = Arc<dyn Fn(Player, Option<&str>) + Send + Sync + 'static>;

/// Registry mapping action IDs and action names to callbacks.
#[derive(Default)]
pub struct MenuActionRegistry {
    by_id: HashMap<u32, MenuActionHandler>,
    by_name: HashMap<String, MenuActionHandler>,
}

impl MenuActionRegistry {
    /// Creates a new empty menu action registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a handler for a specific numeric action ID.
    pub fn register_id(&mut self, id: u32, handler: MenuActionHandler) {
        self.by_id.insert(id, handler);
    }

    /// Registers a handler for a named action.
    pub fn register_name(&mut self, name: impl Into<String>, handler: MenuActionHandler) {
        self.by_name.insert(name.into(), handler);
    }

    /// Dispatches an action event by ID and/or name to the registered callback.
    pub fn dispatch(&self, caller: Player, id: Option<u32>, action_name: Option<&str>) {
        if let Some(id) = id
            && let Some(h) = self.by_id.get(&id)
        {
            h(caller, action_name);
            return;
        }
        if let Some(name) = action_name
            && let Some(h) = self.by_name.get(name)
        {
            h(caller, Some(name));
        }
    }

    /// Clears all registered action handlers.
    pub fn clear(&mut self) {
        self.by_id.clear();
        self.by_name.clear();
    }
}

static GLOBAL_REGISTRY: LazyLock<RwLock<MenuActionRegistry>> =
    LazyLock::new(|| RwLock::new(MenuActionRegistry::default()));

/// Registers a menu action callback by numeric ID in the global registry.
pub fn register_menu_action_id(
    id: u32,
    handler: impl Fn(Player, Option<&str>) + Send + Sync + 'static,
) {
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .register_id(id, Arc::new(handler));
}

/// Registers a menu action callback by string action name in the global registry.
pub fn register_menu_action_name(
    name: impl Into<String>,
    handler: impl Fn(Player, Option<&str>) + Send + Sync + 'static,
) {
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .register_name(name, Arc::new(handler));
}

/// Dispatches a menu action through the global registry.
pub fn dispatch_menu_action(caller: Player, id: Option<u32>, action_name: Option<&str>) {
    GLOBAL_REGISTRY
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .dispatch(caller, id, action_name);
}

/// Clears all menu action callbacks from the global registry.
pub fn clear_menu_actions() {
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
}
