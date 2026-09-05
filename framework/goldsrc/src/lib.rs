//! Public framework (SDK) for GoldSrc.rs plugin developers.
//!
//! This is the main entry point for plugin developers. It provides
//! ergonomic abstractions, macros, ECS, and helpers for writing plugins.

/// Flat ECS for plugin state storage.
pub mod ecs;

/// Unified structured logger for plugins and transparent WASM guest logger.
pub mod logging;
pub use logging::init_guest_logger;

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            #[cfg(target_arch = "wasm32")]
            $crate::logging::init_guest_logger();
            $crate::log::info!(target: "plugin", $($arg)*)
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        {
            #[cfg(target_arch = "wasm32")]
            $crate::logging::init_guest_logger();
            $crate::log::warn!(target: "plugin", $($arg)*)
        }
    };
}

#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {
        {
            #[cfg(target_arch = "wasm32")]
            $crate::logging::init_guest_logger();
            $crate::log::error!(target: "plugin", $($arg)*)
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            #[cfg(target_arch = "wasm32")]
            $crate::logging::init_guest_logger();
            $crate::log::debug!(target: "plugin", $($arg)*)
        }
    };
}

/// Performs single-pass substitution of named `{key}` placeholders without intermediate string reallocations.
#[doc(hidden)]
pub fn substitute_named(template: &str, named: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after_brace = &rest[start + 1..];
        if let Some(end) = after_brace.find('}') {
            let key = &after_brace[..end];
            if let Some((_, val)) = named.iter().find(|(k, _)| *k == key) {
                out.push_str(val);
            } else {
                out.push('{');
                out.push_str(key);
                out.push('}');
            }
            rest = &after_brace[end + 1..];
        } else {
            out.push('{');
            rest = after_brace;
        }
    }
    out.push_str(rest);
    out
}

/// Performs single-pass substitution of 1-based positional `{1}`, `{2}` placeholders without intermediate string reallocations.
#[doc(hidden)]
pub fn substitute_positional(template: &str, pos: &[&str]) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after_brace = &rest[start + 1..];
        if let Some(end) = after_brace.find('}') {
            let key = &after_brace[..end];
            if let Ok(idx) = key.parse::<usize>()
                && idx >= 1
                && idx <= pos.len()
            {
                out.push_str(pos[idx - 1]);
            } else {
                out.push('{');
                out.push_str(key);
                out.push('}');
            }
            rest = &after_brace[end + 1..];
        } else {
            out.push('{');
            rest = after_brace;
        }
    }
    out.push_str(rest);
    out
}

/// Macro for translating keys from dictionaries in WASM plugins.
#[macro_export]
macro_rules! tr {
    ($dict:expr, $lang:expr, $key:expr) => {{
        use $crate::AsLangCode as _;
        $crate::api::bindings::goldsrc::engine::api::host_translate($dict, (&$lang).as_lang_code().as_ref(), $key)
    }};
    ($dict:expr, $lang:expr, $key:expr, $( $k:ident = $v:expr ),* $(,)?) => {{
        use $crate::AsLangCode as _;
        let raw = $crate::api::bindings::goldsrc::engine::api::host_translate($dict, (&$lang).as_lang_code().as_ref(), $key);
        let __owned_vals = [ $( $v.to_string() ),* ];
        let mut __owned_iter = __owned_vals.iter();
        let __named: &[(&str, &str)] = &[
            $( (stringify!($k), __owned_iter.next().unwrap().as_str()) ),*
        ];
        $crate::substitute_named(&raw, __named)
    }};
    ($dict:expr, $lang:expr, $key:expr, $( $pos:expr ),* $(,)?) => {{
        use $crate::AsLangCode as _;
        let raw = $crate::api::bindings::goldsrc::engine::api::host_translate($dict, (&$lang).as_lang_code().as_ref(), $key);
        let __owned_vals = [ $( $pos.to_string() ),* ];
        let mut __owned_iter = __owned_vals.iter();
        let __pos: &[&str] = &[
            $( __owned_iter.next().unwrap().as_str() ),*
        ];
        $crate::substitute_positional(&raw, __pos)
    }};
}

/// Macro for printing chat message to a specific player with formatting and placeholders.
#[macro_export]
macro_rules! chat_print {
    ($player:expr, $fmt:expr) => {
        $player.print($crate::PrintTarget::Chat, $fmt)
    };
    ($player:expr, $fmt:expr, $( $k:ident = $v:expr ),* $(,)?) => {{
        let __owned_vals = [ $( $v.to_string() ),* ];
        let mut __owned_iter = __owned_vals.iter();
        let __named: &[(&str, &str)] = &[
            $( (stringify!($k), __owned_iter.next().unwrap().as_str()) ),*
        ];
        let __s = $crate::substitute_named($fmt, __named);
        $player.print($crate::PrintTarget::Chat, &__s)
    }};
}

/// Macro for broadcasting chat message to all players with formatting and placeholders.
#[macro_export]
macro_rules! chat_broadcast {
    ($fmt:expr) => {
        $crate::engine::client_print(0, $crate::engine::PRINT_CHAT, $fmt)
    };
    ($fmt:expr, $( $k:ident = $v:expr ),* $(,)?) => {{
        let __owned_vals = [ $( $v.to_string() ),* ];
        let mut __owned_iter = __owned_vals.iter();
        let __named: &[(&str, &str)] = &[
            $( (stringify!($k), __owned_iter.next().unwrap().as_str()) ),*
        ];
        let __s = $crate::substitute_named($fmt, __named);
        $crate::engine::client_print(0, $crate::engine::PRINT_CHAT, &__s)
    }};
}

pub mod chat {
    pub use goldsrc_api::chat::*;

    /// Registers a local chat middleware inside a WASM plugin.
    pub fn register_chat_middleware<F>(_middleware: F)
    where
        F: Fn(&mut ChatMessage) -> bool + Send + Sync + 'static,
    {
        // Handled transparently by runtime dispatcher
    }
}

pub mod placeholders {
    pub use goldsrc_api::placeholders::*;
}

pub mod command {
    pub use goldsrc_api::command::*;
}

pub mod event {
    pub use goldsrc_api::event::*;
}

pub mod menu {
    pub use goldsrc_api::menu::*;
}

pub mod modifiers {
    pub use goldsrc_api::modifiers::*;
}

pub use ::log;
pub use ecs::*;
pub use goldsrc_api as api;
pub use goldsrc_api;
pub use goldsrc_api::bindings;
pub use goldsrc_api::engine_api as engine;
pub use goldsrc_api::hud as hud_api;
pub use goldsrc_api::menu as menu_api;
pub use goldsrc_api::modifiers as modifiers_api;
pub use goldsrc_api::{
    Alive, AntiSpamAction, AsLangCode, Auth, BlackboardValue, Bot, CapExpr, ChatScope, ClientKind,
    Command, CommandBuilder, CommandContext, CommandError, CommandHandler, CommandRegistry,
    CommandResult, CommandTarget, CommutativeModifier, Condition, ConnectionState, DagError, Dead,
    DenyAction, DenyPolicy, Engine, Entity, Event, EventHandler, EventPhase, EventRegistry,
    EventSubscriberBuilder, EventSubscription, ExitBehavior, Feedback, FromArg, HLTV, HudColor,
    HudCoord, HudEffect, HudKind, HudMessage, HudMessageBuilder, ItemKind, ItemTitle, LifeState,
    Menu, MenuActionHandler, MenuActionRegistry, MenuBuilder, MenuContext, MenuItem,
    MenuPageBuilder, MenuRendererKind, MenuStyle, ModifierContribution, NodeBuilder, OrderNode,
    Phase, PhasedDag, Placeholder, PlaceholderBuilder, PlaceholderCall, PlaceholderHandler,
    PlaceholderMetadata, PlaceholderRegistry, Player, PlayerStateFilter, PluginTier, PrintTarget,
    RenderedMenuPage, SlotAction, Spectator, SqlDatabase, StorageError, StorageProvider, Team,
    TypedBlackboard, Vector3, VisualDeny, clear_commands, clear_events, clear_menu_actions,
    clear_placeholders, dispatch_command, dispatch_event, dispatch_local_placeholder,
    dispatch_menu_action, register_command, register_menu_action_id, register_menu_action_name,
    register_placeholder, split_command_args, subscribe_event,
};
pub use goldsrc_macros as macros;
pub use goldsrc_macros::{
    command, event, menu_action, on_frame, on_load, on_unload, plugin, system,
};

/// Convenient prelude module for plugin authors.
pub mod prelude {
    pub use crate::ecs::*;
    pub use crate::engine;
    pub use crate::hud_api as hud;
    pub use crate::menu_api;
    pub use crate::modifiers_api as modifiers;
    pub use crate::tr;
    pub use crate::{
        Alive, AntiSpamAction, AsLangCode, Auth, BlackboardValue, Bot, CapExpr, ChatScope,
        ClientKind, Command, CommandBuilder, CommandContext, CommandError, CommandHandler,
        CommandResult, CommandTarget, CommutativeModifier, Condition, ConnectionState, Dead,
        DenyAction, DenyPolicy, Engine, Entity, Event, EventHandler, EventPhase,
        EventSubscriberBuilder, ExitBehavior, Feedback, FromArg, HLTV, HudColor, HudCoord,
        HudEffect, HudKind, HudMessage, HudMessageBuilder, ItemKind, ItemTitle, LifeState, Menu,
        MenuBuilder, MenuContext, MenuItem, MenuPageBuilder, MenuRendererKind, MenuStyle,
        ModifierContribution, Placeholder, PlaceholderBuilder, Player, PlayerStateFilter,
        PrintTarget, RenderedMenuPage, SlotAction, Spectator, SqlDatabase, StorageError,
        StorageProvider, System, SystemBuilder, Team, TypedBlackboard, Vector3, VisualDeny,
    };
    pub use crate::{
        chat_broadcast, chat_print, command, event, menu_action, on_frame, on_load, on_unload,
        plugin, system,
    };
    pub use crate::{log_debug, log_err, log_info, log_warn};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_named_replaces_keys_correctly() {
        let tmpl = "Hello {name}, your balance is {amount}!";
        let named = &[("name", "Alice"), ("amount", "500")];
        let res = substitute_named(tmpl, named);
        assert_eq!(res, "Hello Alice, your balance is 500!");
    }

    #[test]
    fn test_substitute_named_preserves_unmatched_braces() {
        let tmpl = "Hello {name}, keep {unknown} as is, and {escaped.";
        let named = &[("name", "Bob")];
        let res = substitute_named(tmpl, named);
        assert_eq!(res, "Hello Bob, keep {unknown} as is, and {escaped.");
    }

    #[test]
    fn test_substitute_positional_replaces_1_based_indices() {
        let tmpl = "Player {1} killed {2} with {3}";
        let pos = &["Alice", "Bob", "AWP"];
        let res = substitute_positional(tmpl, pos);
        assert_eq!(res, "Player Alice killed Bob with AWP");
    }
}
