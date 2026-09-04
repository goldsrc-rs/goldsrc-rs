//! Pure Rust traits (interfaces) for GoldSrc engine interaction.
//!
//! This crate defines the abstract interface that plugin developers use.
//! It has no dependency on any specific backend (Metamod or Standalone).

/// Capability-based access control, registry, and hierarchical DSL.
pub mod auth;
/// Generated WASM bindings (wasm32 only).
pub mod bindings;
/// In-game chat interception, formatting, and packet splitting.
pub mod chat;
/// Core player and client domain abstractions, states, and typestate guards.
pub mod client;
/// Command routing targets, scope filters, programmatic builder, and errors.
pub mod command;
/// Global constants for the engine and framework.
pub mod consts;
/// Typed CVar bindings and flags.
pub mod cvar;
/// Unified Expression DSL lexer, parser, and grammar primitives.
pub mod dsl;
/// Validated `edict_t` handle.
pub mod edict;
/// Modular engine sub-system traits, unified engine bridge, and API facade.
pub mod engine;
/// Event subscription, priority ordering, and local guest event dispatching.
pub mod event;
/// Gamedata definitions, signature scanning, and VTable offset configurations.
pub mod gamedata;
/// Screen HUD and DHUD message builders and styling.
pub mod hud;
/// Mod descriptor manifest (`liblist.gam`) parser and model.
pub mod liblist;
/// Declarative multi-page menu system.
pub mod menu;
/// Dynamic contextual placeholders and function calls.
pub mod placeholders;
/// High-level ReAPI capability flags, detection, and queries.
pub mod reapi;
/// Unified requirements DSL.
pub mod requirements;
/// Generic Reactive Rule & Provider Engine.
pub mod rules;
/// Dual Storage Port Abstraction & Typed Bucket Facade.
pub mod storage;

pub use auth::{Auth, CapExpr, CapabilityRegistry};
pub use chat::{ChatMessage, ChatScope, MAX_SAYTEXT_PAYLOAD_LEN, split_chat_chunks};
pub use client::{
    Alive, AsLangCode, Bot, ClientKind, ConnectionState, Dead, HLTV, LifeState, Player,
    PrintTarget, Spectator, Team,
};
pub use command::{
    Command, CommandBuilder, CommandContext, CommandError, CommandHandler, CommandRegistry,
    CommandResult, CommandTarget, FromArg, PlayerStateFilter, clear_commands, dispatch_command,
    register_command, split_command_args,
};
pub use cvar::{Cvar, CvarFlags};
pub use dsl::{Lexer, Token};
pub use edict::EDict;
pub use engine::{
    Engine, EngineConsole, EngineCvars, EngineEntities, EngineMessages, EnginePhysics,
    EnginePrecache, EngineSound, HUD_PRINTCENTER, HUD_PRINTCHAT, HUD_PRINTCONSOLE, HUD_PRINTNOTIFY,
    HUD_PRINTRADIO, MAX_EDICTS, MAX_PLAYERS, MessageBuilder, MessageDest, PRINT_CENTER, PRINT_CHAT,
    PRINT_CONSOLE, PRINT_NOTIFY, SAFE_SAYTEXT_LIMIT, TraceResult, cyrillic_to_latin, engine_api,
    format_center_text, format_notify_text, format_say_text, utf8_to_cp1251,
};
pub use event::{
    Event, EventHandler, EventPriority, EventRegistry, EventSubscriberBuilder, EventSubscription,
    clear_events, dispatch_event, subscribe_event,
};
pub use gamedata::{GameData, MemorySignature, VTableFunc};
pub use hud::{
    FadeFlags, HudColor, HudCoord, HudEffect, HudKind, HudMessage, HudMessageBuilder, ScreenFade,
    ScreenFadeBuilder, ScreenShake, ScreenShakeBuilder,
};
pub use liblist::{LIBLIST_FILENAME, LibList};
pub use menu::{
    AntiSpamAction, Condition, DenyAction, DenyPolicy, ExitBehavior, Feedback, ItemKind, ItemTitle,
    Menu, MenuActionHandler, MenuActionRegistry, MenuBuilder, MenuContext, MenuItem,
    MenuPageBuilder, MenuRendererKind, MenuStyle, RenderedMenuPage, SlotAction, VisualDeny,
    clear_menu_actions, dispatch_menu_action, register_menu_action_id, register_menu_action_name,
};
pub use placeholders::{
    CallArg, Placeholder, PlaceholderBuilder, PlaceholderCall, PlaceholderHandler,
    PlaceholderMetadata, PlaceholderRegistry, PlayerTarget, clear_placeholders,
    dispatch_local_placeholder, parse_placeholder_call, register_placeholder,
};
pub use reapi::{ReApiStatus, ReGameCapabilities, RehldsCapabilities};
pub use requirements::{CvarOp, Requirement};
pub use rules::{Rule, RuleAction, RuleCondition, RuleEngine, RuleRegistry};
pub use storage::{SqlDatabase, StorageError, StorageProvider};

/// Safe wrapper around `edict_t` (entity dictionary).
///
/// Delegates field access to [`EDict`] which validates the serial number on
/// every access, preventing use-after-free from stale handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    /// Entity index (0 = world, 1..=N = players).
    pub index: i32,
    #[cfg(not(target_arch = "wasm32"))]
    inner: EDict,
}

impl Entity {
    /// Creates an Entity from a raw index and edict_t pointer.
    ///
    /// # Safety
    /// The caller must ensure that `edict` is a valid pointer to an entity in the engine.
    #[cfg(all(not(target_arch = "wasm32"), feature = "unsafe-sys"))]
    pub unsafe fn from_raw(index: i32, edict: *mut goldsrc_sys::edict_t) -> Self {
        // SAFETY: propagated from caller.
        Self {
            index,
            inner: unsafe { EDict::from_raw(index, edict) },
        }
    }

    /// Creates an `Entity` handle for `index`.
    #[cfg(target_arch = "wasm32")]
    pub fn new(index: i32) -> Self {
        Self { index }
    }

    /// Creates an `Entity` handle for `index` with an invalid backing edict
    /// (host-only placeholder; use [`Entity::from_raw`] with a real pointer).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(index: i32) -> Self {
        Self {
            index,
            inner: EDict::invalid(),
        }
    }

    /// Returns the entity index.
    pub fn index(&self) -> i32 {
        self.index
    }

    /// Returns `true` if the underlying edict slot is still valid.
    pub fn is_valid(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_is_valid(self.index)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.is_valid()
        }
    }

    /// Returns the entity classname (e.g. `"player"`, `"func_door"`), if set.
    pub fn classname(&self) -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_classname(self.index)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.classname()
        }
    }

    /// Prints a chat message to the player. On native hosts this is a no-op
    /// (chat printing is not yet part of the engine bridge surface).
    pub fn print_chat(&self, msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_log(&format!(
                "Print to player {}: {}",
                self.index, msg
            ));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = msg;
        }
    }

    /// Checks if the player has the specified capability.
    pub fn has_capability(&self, name: &str) -> bool {
        crate::auth::Auth::has_capability(self.index, name)
    }

    /// Grants a capability to the player dynamically.
    pub fn grant_capability(&self, name: &str) -> bool {
        crate::auth::Auth::grant_capability(self.index, name)
    }

    /// Returns the entity origin as a flat `[x, y, z]` array.
    pub fn origin(&self) -> [f32; 3] {
        #[cfg(target_arch = "wasm32")]
        {
            let v = crate::bindings::goldsrc::engine::api::host_entity_origin(self.index);
            [v.x, v.y, v.z]
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.origin().unwrap_or([0.0, 0.0, 0.0])
        }
    }

    /// Returns the entity health.
    pub fn health(&self) -> f32 {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_health(self.index)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.health().unwrap_or(0.0)
        }
    }

    /// Returns the entity's rotation angles (pitch, yaw, roll).
    pub fn angles(&self) -> Vector3 {
        #[cfg(target_arch = "wasm32")]
        {
            let v = crate::bindings::goldsrc::engine::api::host_entity_angles(self.index);
            Vector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.angles().unwrap_or([0.0, 0.0, 0.0]).into()
        }
    }

    /// Sets the entity's rotation angles.
    pub fn set_angles(&mut self, angles: Vector3) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_set_angles(
                self.index,
                crate::bindings::goldsrc::engine::api::Vector3 {
                    x: angles.x,
                    y: angles.y,
                    z: angles.z,
                },
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.set_angles(angles.into());
        }
    }

    /// Returns the raw `edict_t` pointer, or null if the handle is stale.
    #[cfg(all(not(target_arch = "wasm32"), feature = "unsafe-sys"))]
    pub fn as_ptr(&self) -> *mut goldsrc_sys::edict_t {
        self.inner.as_ptr().unwrap_or(std::ptr::null_mut())
    }

    /// Access the underlying [`EDict`] handle directly.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn edict(&self) -> EDict {
        self.inner
    }
}

/// 3D Vector type for GoldSrc positions and velocities.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector3 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
    /// Z component.
    pub z: f32,
}

impl Vector3 {
    /// Create a new 3D vector.
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl From<[f32; 3]> for Vector3 {
    fn from(arr: [f32; 3]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
            z: arr[2],
        }
    }
}

impl From<Vector3> for [f32; 3] {
    fn from(v: Vector3) -> Self {
        [v.x, v.y, v.z]
    }
}

// SAFETY: Entity is just a wrapper around raw pointers / integer index.
// The caller must ensure the pointer is valid when used.
unsafe impl Send for Entity {}
unsafe impl Sync for Entity {}
