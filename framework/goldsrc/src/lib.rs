//! Public framework (SDK) for GoldSrc.rs plugin developers.
//!
//! This is the main entry point for plugin developers. It provides
//! ergonomic abstractions, macros, ECS, and helpers for writing plugins.

/// Flat ECS for plugin state storage.
#[cfg(feature = "ecs")]
pub mod ecs;

/// Foolproof asynchronous task dispatch and worker synchronization.
pub mod task;

/// Unified structured logger for plugins and transparent WASM guest logger.
pub mod logging;
pub use logging::init_guest_logger;

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            #[cfg(target_arch = "wasm32")]
            $crate::logging::init_guest_logger();
            $crate::log::info!(target: $crate::api::consts::log_targets::PLUGIN, $($arg)*)
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        {
            #[cfg(target_arch = "wasm32")]
            $crate::logging::init_guest_logger();
            $crate::log::warn!(target: $crate::api::consts::log_targets::PLUGIN, $($arg)*)
        }
    };
}

#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {
        {
            #[cfg(target_arch = "wasm32")]
            $crate::logging::init_guest_logger();
            $crate::log::error!(target: $crate::api::consts::log_targets::PLUGIN, $($arg)*)
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            #[cfg(target_arch = "wasm32")]
            $crate::logging::init_guest_logger();
            $crate::log::debug!(target: $crate::api::consts::log_targets::PLUGIN, $($arg)*)
        }
    };
}

/// Internal helper for plugin frame hook dispatch (ECS and task queue).
#[doc(hidden)]
#[inline(always)]
pub fn __plugin_frame_dispatch() {
    #[cfg(feature = "task")]
    crate::task::drain_main_tasks(64);

    #[cfg(feature = "ecs")]
    crate::ecs::run_frame_systems();
}

/// Internal helper for menu selection event dispatch.
#[doc(hidden)]
#[inline(always)]
pub fn __plugin_dispatch_menu_select(caller: i32, slot: u32) {
    #[cfg(feature = "menu")]
    {
        let handled = crate::menu::handle_player_menu_select(caller, slot as u8);
        if !handled {
            crate::menu::dispatch_menu_action(crate::Player::new(caller), Some(slot), None);
        }
    }
    #[cfg(not(feature = "menu"))]
    let _ = (caller, slot);
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
        $crate::Player::new(0).print($crate::PrintTarget::Chat, $fmt)
    };
    ($fmt:expr, $( $k:ident = $v:expr ),* $(,)?) => {{
        let __owned_vals = [ $( $v.to_string() ),* ];
        let mut __owned_iter = __owned_vals.iter();
        let __named: &[(&str, &str)] = &[
            $( (stringify!($k), __owned_iter.next().unwrap().as_str()) ),*
        ];
        let __s = $crate::substitute_named($fmt, __named);
        $crate::Player::new(0).print($crate::PrintTarget::Chat, &__s)
    }};
}

pub mod chat {
    pub use goldsrc_api::chat::*;
    use std::sync::RwLock;

    type ChatMiddlewareFn = Box<dyn Fn(&mut ChatMessage) -> bool + Send + Sync + 'static>;
    static CHAT_MIDDLEWARE: RwLock<Vec<ChatMiddlewareFn>> = RwLock::new(Vec::new());

    /// Registers a local chat middleware inside a WASM plugin.
    pub fn register_chat_middleware<F>(middleware: F)
    where
        F: Fn(&mut ChatMessage) -> bool + Send + Sync + 'static,
    {
        if let Ok(mut list) = CHAT_MIDDLEWARE.write() {
            list.push(Box::new(middleware));
        }
    }

    /// Dispatches incoming chat through local middleware pipeline.
    /// Returns Some(final_text) if allowed, or None if blocked/suppressed.
    pub fn dispatch_local_chat(sender: i32, text: &str, is_team: bool) -> Option<String> {
        let Ok(list) = CHAT_MIDDLEWARE.read() else {
            return Some(text.to_string());
        };
        if list.is_empty() {
            return Some(text.to_string());
        }
        let scope = if is_team {
            ChatScope::same_team()
        } else {
            ChatScope::all()
        };
        let mut msg = ChatMessage::new(crate::Player::new(sender), text, scope);
        for mw in list.iter() {
            let allow = mw(&mut msg);
            if !allow || msg.is_blocked {
                return None;
            }
        }
        let final_text = if let Some(ref p) = msg.prefix {
            format!("{p}{}", msg.formatted_text)
        } else {
            msg.formatted_text
        };
        Some(final_text)
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

pub mod pipeline {
    pub use goldsrc_api::pipeline::*;
}

pub mod spec {
    pub use goldsrc_api::spec::*;
}

pub mod menu {
    pub use goldsrc_api::menu::*;
}

pub mod modifiers {
    pub use goldsrc_api::modifiers::*;
}

pub mod client {
    pub use goldsrc_api::client::*;
}

pub mod entity {
    pub use goldsrc_api::entity::*;
}

pub mod action {
    pub use goldsrc_api::action::*;
}

pub mod prop {
    pub use goldsrc_api::prop::*;
}

pub mod property {}

pub mod cvar {
    pub use goldsrc_api::cvar::*;
}

pub mod extension;
pub mod reapi;

pub use ::log;
#[cfg(feature = "ecs")]
pub use ecs::*;
pub use goldsrc_api as api;
pub use goldsrc_api;
pub use goldsrc_api::bindings;
pub use goldsrc_api::bundle as bundle_api;
pub use goldsrc_api::bundle::{
    BundleComponentSpec, BundleInfo, BundleManifest, BundleValidationError, ComponentRole,
};
pub use goldsrc_api::hud as hud_api;
pub use goldsrc_api::menu as menu_api;
pub use goldsrc_api::modifiers as modifiers_api;
pub use goldsrc_api::pipeline as pipeline_api;
pub use goldsrc_api::spec as spec_api;
pub use goldsrc_api::{
    Action, AdminCaps, Alive, All, Angles, AntiSpamAction, Any, Armor, AsLangCode, Auth,
    BlackboardValue, Bot, ButtonInputDriver, CancellationToken, CapExpr, ChatMenuRenderer,
    ChatScope, ChatTarget, CheckCapability, Checkbox, ClassicMenuRenderer, Classname, Client,
    ClientExt, ClientKind, Command, CommandBuilder, CommandContext, CommandError, CommandHandler,
    CommandRegistry, CommandResult, CommandTarget, CommutativeModifier, Condition, Connected,
    ConnectedClient, ConnectionState, DagError, Dead, DeadPlayer, DenyAction, DenyPolicy,
    DhudMenuRenderer, Dormant, Entity, EntityExt, EntityId, Event, EventHandler, EventPhase,
    EventRegistry, EventSubscriberBuilder, EventSubscription, ExitBehavior, Feedback, FromArg,
    GhostSlotTrap, Health, Hltv, HudColor, HudCoord, HudEffect, HudKind, HudMessage,
    HudMessageBuilder, Human, HumanClient, HybridInputDriver, Interceptor, ItemKind, ItemTitle,
    LifeState, LivingHuman, LivingPlayer, Menu, MenuActionHandler, MenuActionRegistry, MenuBuilder,
    MenuComponent, MenuContext, MenuInputAction, MenuInputDriver, MenuItem, MenuPageBuilder,
    MenuRenderer, MenuRendererKind, MenuStyle, ModifierContribution, MotdMenuRenderer, NodeBuilder,
    NoneOf, Not, OrderNode, Origin, Phase, PhasedDag, Pipeline, PipelineFlow, Placeholder,
    PlaceholderBuilder, PlaceholderCall, PlaceholderHandler, PlaceholderMetadata,
    PlaceholderRegistry, Player, PlayerAction, PlayerExt, PlayerSlot, PlayerStateFilter,
    PluginTier, PrintTarget, Prop, PropGet, PropSet, RefineExt, Refined, RenderedMenuPage, Slider,
    SlotAction, SlotInputDriver, Solid, SolidEntity, Spawned, SpawnedEntity, Spec, SpecError,
    SpectatingPlayer, Spectator, Team, TeamTarget, TerminalTuiRenderer, TextInput, TypedBlackboard,
    ValidationResult, Vector3, Velocity, VipCaps, VisualDeny, clear_commands, clear_events,
    clear_menu_actions, clear_placeholders, dispatch_command, dispatch_event,
    dispatch_local_placeholder, dispatch_menu_action, register_command, register_menu_action_id,
    register_menu_action_name, register_placeholder, split_command_args, subscribe_event,
    use_command_interceptor,
};
pub use goldsrc_macros as macros;
pub use goldsrc_macros::{
    ConfigModel, bundle, command, event, menu_action, on_frame, on_load, on_unload, permission,
    permissions, plugin, requires, role, system,
};

/// Convenient prelude module for plugin authors.
pub mod prelude {
    pub use crate::cvar::{self, ConfigModel, Cvar, CvarFlags};
    #[cfg(feature = "ecs")]
    pub use crate::ecs::*;
    pub use crate::hud_api as hud;
    pub use crate::menu_api;
    pub use crate::modifiers_api as modifiers;
    pub use crate::task;
    pub use crate::tr;
    pub use crate::{
        Action, AdminCaps, Alive, All, Angles, AntiSpamAction, Any, Armor, AsLangCode, Auth,
        BlackboardValue, Bot, ButtonInputDriver, CancellationToken, CapExpr, ChatMenuRenderer,
        ChatScope, ChatTarget, CheckCapability, Checkbox, ClassicMenuRenderer, Classname, Client,
        ClientExt, ClientKind, Command, CommandBuilder, CommandContext, CommandError,
        CommandHandler, CommandResult, CommandTarget, CommutativeModifier, Condition, Connected,
        ConnectedClient, ConnectionState, Dead, DeadPlayer, DenyAction, DenyPolicy,
        DhudMenuRenderer, Dormant, Entity, EntityExt, EntityId, Event, EventHandler, EventPhase,
        EventSubscriberBuilder, ExitBehavior, Feedback, FromArg, GhostSlotTrap, Health, Hltv,
        HudColor, HudCoord, HudEffect, HudKind, HudMessage, HudMessageBuilder, Human, HumanClient,
        HybridInputDriver, Interceptor, ItemKind, ItemTitle, LifeState, LivingHuman, LivingPlayer,
        Menu, MenuBuilder, MenuComponent, MenuContext, MenuInputAction, MenuInputDriver, MenuItem,
        MenuPageBuilder, MenuRenderer, MenuRendererKind, MenuStyle, ModifierContribution,
        MotdMenuRenderer, NoneOf, Not, Origin, Pipeline, PipelineFlow, Placeholder,
        PlaceholderBuilder, Player, PlayerAction, PlayerExt, PlayerSlot, PlayerStateFilter,
        PrintTarget, Prop, PropGet, PropSet, RefineExt, Refined, RenderedMenuPage, Slider,
        SlotAction, SlotInputDriver, Solid, SolidEntity, Spawned, SpawnedEntity, Spec, SpecError,
        SpectatingPlayer, Spectator, Team, TeamTarget, TerminalTuiRenderer, TextInput,
        TypedBlackboard, ValidationResult, Vector3, Velocity, VipCaps, VisualDeny, action, prop,
        use_command_interceptor,
    };
    pub use crate::{
        bundle, chat_broadcast, chat_print, command, event, extension, menu_action, on_frame,
        on_load, on_unload, plugin, reapi, role, system,
    };
    pub use crate::{log_debug, log_err, log_info, log_warn};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cvar::ConfigModel;

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

    #[derive(Debug, Clone, PartialEq, ConfigModel)]
    struct DemoVipConfig {
        #[cvar(name = "vip_bonus_hp", flags = "archive", description = "Bonus HP")]
        pub bonus_hp: i32,
        #[cvar(name = "vip_tag", flags = "server", description = "VIP Tag")]
        pub tag: String,
    }

    #[test]
    fn test_derive_config_model_to_toml_and_cvars() {
        let cfg = DemoVipConfig {
            bonus_hp: 50,
            tag: "VIP".to_string(),
        };

        let toml_str = cfg.to_toml();
        assert!(toml_str.contains("# Bonus HP"));
        assert!(toml_str.contains("bonus_hp = 50"));
        assert!(toml_str.contains("# VIP Tag"));
        assert!(toml_str.contains("tag = \"VIP\""));

        let cvars_str = cfg.to_cvars();
        assert!(cvars_str.contains("vip_bonus_hp \"50\" // Bonus HP"));
        assert!(cvars_str.contains("vip_tag \"VIP\" // VIP Tag"));
    }

    #[test]
    fn test_extension_and_reapi_host_queries() {
        assert!(!crate::extension::is_available("reapi", None));
        assert_eq!(crate::extension::version("reapi"), None);
        assert!(!crate::reapi::has_reapi());
        assert_eq!(crate::reapi::reapi_version(), None);
    }
}
