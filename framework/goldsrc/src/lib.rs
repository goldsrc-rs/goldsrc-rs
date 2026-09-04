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
        let mut __res = raw;
        $(
            let __pat = concat!("{", stringify!($k), "}");
            __res = __res.replace(__pat, &$v.to_string());
        )*
        __res
    }};
    ($dict:expr, $lang:expr, $key:expr, $( $pos:expr ),* $(,)?) => {{
        use $crate::AsLangCode as _;
        let raw = $crate::api::bindings::goldsrc::engine::api::host_translate($dict, (&$lang).as_lang_code().as_ref(), $key);
        let mut __res = raw;
        let mut __idx = 1;
        $(
            let __pat = format!("{{{}}}", __idx);
            __res = __res.replace(&__pat, &$pos.to_string());
            __idx += 1;
        )*
        __res
    }};
}

/// Macro for printing chat message to a specific player with formatting and placeholders.
#[macro_export]
macro_rules! chat_print {
    ($player:expr, $fmt:expr) => {
        $player.print($crate::PrintTarget::Chat, $fmt)
    };
    ($player:expr, $fmt:expr, $( $k:ident = $v:expr ),* $(,)?) => {{
        let mut __s = $fmt.to_string();
        $(
            let __pat = concat!("{", stringify!($k), "}");
            __s = __s.replace(__pat, &$v.to_string());
        )*
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
        let mut __s = $fmt.to_string();
        $(
            let __pat = concat!("{", stringify!($k), "}");
            __s = __s.replace(__pat, &$v.to_string());
        )*
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

pub use ::log;
pub use ecs::*;
pub use goldsrc_api as api;
pub use goldsrc_api;
pub use goldsrc_api::bindings;
pub use goldsrc_api::engine_api as engine;
pub use goldsrc_api::hud as hud_api;
pub use goldsrc_api::menu as menu_api;
pub use goldsrc_api::{
    Alive, AntiSpamAction, AsLangCode, Auth, Bot, CapExpr, ChatScope, ClientKind, Command,
    CommandBuilder, CommandContext, CommandError, CommandHandler, CommandRegistry, CommandResult,
    CommandTarget, Condition, ConnectionState, Dead, DenyAction, DenyPolicy, Engine, Entity, Event,
    EventHandler, EventPriority, EventRegistry, EventSubscriberBuilder, EventSubscription,
    ExitBehavior, Feedback, FromArg, HLTV, HudColor, HudCoord, HudEffect, HudKind, HudMessage,
    HudMessageBuilder, ItemKind, ItemTitle, LifeState, Menu, MenuActionHandler, MenuActionRegistry,
    MenuBuilder, MenuContext, MenuItem, MenuPageBuilder, MenuRendererKind, MenuStyle, Placeholder,
    PlaceholderBuilder, PlaceholderCall, PlaceholderHandler, PlaceholderMetadata,
    PlaceholderRegistry, Player, PlayerStateFilter, PrintTarget, RenderedMenuPage, SlotAction,
    Spectator, SqlDatabase, StorageError, StorageProvider, Team, Vector3, VisualDeny,
    clear_commands, clear_events, clear_menu_actions, clear_placeholders, dispatch_command,
    dispatch_event, dispatch_local_placeholder, dispatch_menu_action, register_command,
    register_menu_action_id, register_menu_action_name, register_placeholder, split_command_args,
    subscribe_event,
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
    pub use crate::tr;
    pub use crate::{
        Alive, AntiSpamAction, AsLangCode, Auth, Bot, CapExpr, ChatScope, ClientKind, Command,
        CommandBuilder, CommandContext, CommandError, CommandHandler, CommandResult, CommandTarget,
        Condition, ConnectionState, Dead, DenyAction, DenyPolicy, Engine, Entity, Event,
        EventHandler, EventPriority, EventSubscriberBuilder, ExitBehavior, Feedback, FromArg, HLTV,
        HudColor, HudCoord, HudEffect, HudKind, HudMessage, HudMessageBuilder, ItemKind, ItemTitle,
        LifeState, Menu, MenuBuilder, MenuContext, MenuItem, MenuPageBuilder, MenuRendererKind,
        MenuStyle, Placeholder, PlaceholderBuilder, Player, PlayerStateFilter, PrintTarget,
        RenderedMenuPage, SlotAction, Spectator, SqlDatabase, StorageError, StorageProvider,
        System, SystemBuilder, Team, Vector3, VisualDeny,
    };
    pub use crate::{
        chat_broadcast, chat_print, command, event, menu_action, on_frame, on_load, on_unload,
        plugin, system,
    };
    pub use crate::{log_debug, log_err, log_info, log_warn};
}
