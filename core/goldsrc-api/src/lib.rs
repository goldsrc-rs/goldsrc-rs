//! Pure Rust traits (interfaces) for GoldSrc engine interaction.
//!
//! This crate defines the abstract interface that plugin developers use.
//! It has no dependency on any specific backend (Metamod or Standalone).

/// Universal Entity and Player Action System and Value Objects.
pub mod action;
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
/// Universal Phased Directed Acyclic Graph (PhasedDag) ordering engine.
pub mod dag;
/// Unified Expression DSL lexer, parser, and grammar primitives.
pub mod dsl;
/// Modular engine sub-system traits, unified engine bridge, and API facade.
pub mod engine;
/// Safe wrapper around engine entities and entity extension traits.
pub mod entity;
/// Event subscription, priority ordering, and local guest event dispatching.
pub mod event;
/// Gamedata definitions, signature scanning, and VTable offset configurations.
pub mod gamedata;
/// Screen HUD and DHUD message builders and styling.
pub mod hud;
/// Declarative multi-page menu system.
pub mod menu;
/// Commutative state modifiers and typed context blackboard.
pub mod modifiers;
/// Universal Interceptor Pipeline and Chain of Responsibility Pattern.
pub mod pipeline;
/// Dynamic contextual placeholders and function calls.
pub mod placeholders;
/// Universal Entity and Player Property System (`Property` & `MutProperty`).
pub mod property;
/// High-level ReAPI capability flags, detection, and queries.
pub mod reapi;
/// Unified requirements DSL.
pub mod requirements;
/// Generic Reactive Rule & Provider Engine.
pub mod rules;
/// Compile-time specifications, logical combinators, and state-guarded refinement.
pub mod spec;
/// Dual Storage Port Abstraction & Typed Bucket Facade.
pub mod storage;
/// Fundamental game data types, spatial mathematics, and engine descriptors.
pub mod types;

pub use action::{Action, CancellationToken, PlayerAction};
pub use auth::{Auth, CapExpr, CapabilityRegistry, CheckCapability};
pub use chat::{ChatMessage, ChatScope, MAX_SAYTEXT_PAYLOAD_LEN, split_chat_chunks};
pub use client::{
    Alive, AsLangCode, Bot, ClientExt, ClientKind, ConnectionState, Dead, Hltv, LifeState, Player,
    PlayerExt, PrintTarget, Spectator, Team,
};
pub use command::{
    Command, CommandBuilder, CommandContext, CommandError, CommandHandler, CommandRegistry,
    CommandResult, CommandTarget, FromArg, PlayerStateFilter, clear_commands, dispatch_command,
    register_command, split_command_args, use_command_interceptor,
};
pub use cvar::{Cvar, CvarFlags};
pub use dag::{DagError, EventPhase, NodeBuilder, OrderNode, Phase, PhasedDag, PluginTier};
pub use dsl::{Lexer, Token};

pub use engine::{
    Engine, EngineConsole, EngineCvars, EngineEntities, EngineMessages, EnginePhysics,
    EnginePrecache, EngineSound, HUD_PRINTCENTER, HUD_PRINTCHAT, HUD_PRINTCONSOLE, HUD_PRINTNOTIFY,
    HUD_PRINTRADIO, MAX_EDICTS, MAX_PLAYERS, MessageBuilder, MessageDest, PRINT_CENTER, PRINT_CHAT,
    PRINT_CONSOLE, PRINT_NOTIFY, SAFE_SAYTEXT_LIMIT, TraceResult, cyrillic_to_latin, engine_api,
    format_center_text, format_notify_text, format_say_text, utf8_to_cp1251,
};
pub use entity::{Entity, EntityExt};
pub use event::{
    Event, EventHandler, EventRegistry, EventSubscriberBuilder, EventSubscription, clear_events,
    dispatch_event, subscribe_event,
};
pub use gamedata::{GameData, MemorySignature, VTableFunc};
pub use hud::{
    FadeFlags, HudColor, HudCoord, HudEffect, HudKind, HudMessage, HudMessageBuilder, ScreenFade,
    ScreenFadeBuilder, ScreenShake, ScreenShakeBuilder,
};

pub use menu::{
    AntiSpamAction, Condition, DenyAction, DenyPolicy, ExitBehavior, Feedback, ItemKind, ItemTitle,
    Menu, MenuActionHandler, MenuActionRegistry, MenuBuilder, MenuContext, MenuItem,
    MenuPageBuilder, MenuRendererKind, MenuStyle, RenderedMenuPage, SlotAction, VisualDeny,
    clear_menu_actions, dispatch_menu_action, register_menu_action_id, register_menu_action_name,
};
pub use modifiers::{BlackboardValue, CommutativeModifier, ModifierContribution, TypedBlackboard};
pub use pipeline::{Interceptor, Pipeline, PipelineFlow};
pub use placeholders::{
    CallArg, Placeholder, PlaceholderBuilder, PlaceholderCall, PlaceholderHandler,
    PlaceholderMetadata, PlaceholderRegistry, PlayerTarget, clear_placeholders,
    dispatch_local_placeholder, parse_placeholder_call, register_placeholder,
};
pub use property::{
    Angles, Armor, Classname, Health, Origin, Prop, PropGet, PropSet, Velocity, prop,
};
pub use reapi::{ReApiStatus, ReGameCapabilities, RehldsCapabilities};
pub use requirements::{CvarOp, Requirement};
pub use rules::{Rule, RuleAction, RuleCondition, RuleEngine, RuleRegistry, RuleScope};
pub use spec::{
    All, Any, Connected, Dormant, Human, NoneOf, Not, RefineExt, Refined, Solid, Spawned, Spec,
    SpecError,
};
pub use storage::{SqlDatabase, StorageError, StorageProvider};
pub use types::{EDict, LIBLIST_FILENAME, LibList, Vector3, bump_map_generation};
