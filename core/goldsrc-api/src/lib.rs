//! Pure Rust traits (interfaces) for GoldSrc engine interaction.
//!
//! This crate defines the abstract domain model and consumer interfaces that plugin developers use.
//! It has no dependency on any specific host or backend (Metamod, Standalone, or Engine HAL).

/// Universal Entity and Player Action System and Value Objects.
pub mod action;
/// Capability-based access control, registry, and hierarchical DSL.
pub mod auth;
/// Generated WASM bindings (wasm32 only).
pub mod bindings;
/// Autonomous Bundle Component Model and Metadata Abstractions.
pub mod bundle;
/// In-game chat interception, formatting, and packet splitting.
pub mod chat;
/// Core player and client domain abstractions, states, and typestate guards.
pub mod client;
/// Command routing targets, scope filters, programmatic builder, and errors.
pub mod command;
/// Global constants for the engine and framework.
pub mod consts;
/// Typed CVar bindings, builder, flags, and ConfigModel sync abstractions.
pub mod cvar;
/// Universal Phased Directed Acyclic Graph (PhasedDag) ordering engine.
pub mod dag;
/// Unified Expression DSL lexer, parser, and grammar primitives.
pub mod dsl;
/// Engine console commands, client commands, and configuration presets.
pub mod engine;
/// Safe wrapper around engine entities, spawner, and entity extension traits.
pub mod entity;
/// Event subscription, priority ordering, and local guest event dispatching.
pub mod event;
/// Gamedata definitions, signature scanning, and VTable offset configurations.
#[cfg(feature = "gamedata")]
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
/// Unified requirements DSL.
pub mod requirements;
/// Generic Reactive Rule & Provider Engine.
#[cfg(feature = "rules")]
pub mod rules;
/// Compile-time specifications, logical combinators, and state-guarded refinement.
pub mod spec;
/// Text encoding utilities and chat color escape code converters.
pub mod text;
/// Discrete tick and continuous duration task scheduling abstractions.
pub mod timer;
/// Fundamental game data types, spatial mathematics, and engine descriptors.
pub mod types;

pub use action::{Action, CancellationToken, PlayerAction};
pub use auth::{
    AdminCaps, Auth, CapExpr, CapabilityRegistry, CheckCapability, ValidationResult, VipCaps,
};
pub use bundle::{
    BundleComponentSpec, BundleInfo, BundleManifest, BundleValidationError, ComponentRole,
};
pub use chat::{
    ChatMessage, ChatScope, ChatTarget, LifeStateFilter, MAX_SAYTEXT_PAYLOAD_LEN, TeamTarget,
    split_chat_chunks,
};
pub use client::{
    Alive, AsLangCode, Bot, Client, ClientExt, ClientKind, Connected, ConnectedClient,
    ConnectionState, Dead, DeadPlayer, Hltv, Human, HumanClient, LifeState, LivingHuman,
    LivingPlayer, Player, PlayerExt, PlayerSlot, PrintTarget, SpectatingPlayer, Spectator, Team,
};
pub use command::{
    Command, CommandBuilder, CommandContext, CommandError, CommandHandler, CommandRegistry,
    CommandResult, CommandTarget, FromArg, PlayerStateFilter, clear_commands, dispatch_command,
    register_command, split_command_args, use_command_interceptor,
};
pub use consts::*;
pub use cvar::{ConfigModel, Cvar, CvarEngine, CvarField, CvarFlags, FromCvarEngine};
pub use dag::{DagError, EventPhase, NodeBuilder, OrderNode, Phase, PhasedDag, PluginTier};
pub use dsl::{Lexer, Token};
pub use engine::{client_command, config_exec, server_command};
pub use entity::{
    Entity, EntityBuilder, EntityExt, EntityId, EntitySpawner, SolidEntity, SpawnedEntity,
};
pub use event::{
    Event, EventHandler, EventRegistry, EventSubscriberBuilder, EventSubscription, clear_events,
    dispatch_event, subscribe_event,
};
#[cfg(feature = "gamedata")]
pub use gamedata::{GameData, MemorySignature, VTableFunc};
pub use hud::{
    FadeFlags, HudColor, HudCoord, HudEffect, HudKind, HudMessage, HudMessageBuilder, ScreenFade,
    ScreenFadeBuilder, ScreenShake, ScreenShakeBuilder,
};
pub use menu::{
    AntiSpamAction, ClassicMenuRenderer, Condition, DenyAction, DenyPolicy, DhudMenuRenderer,
    ExitBehavior, Feedback, ItemKind, ItemTitle, Menu, MenuActionHandler, MenuActionRegistry,
    MenuBuilder, MenuContext, MenuItem, MenuPageBuilder, MenuRenderer, MenuRendererKind, MenuStyle,
    RenderedMenuPage, SlotAction, VisualDeny, clear_menu_actions, dispatch_menu_action,
    register_menu_action_id, register_menu_action_name,
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
pub use requirements::{CvarOp, Requirement};
#[cfg(feature = "rules")]
pub use rules::{Rule, RuleAction, RuleCondition, RuleEngine, RuleRegistry, RuleScope};
pub use spec::{
    All, Any, Dormant, NoneOf, Not, RefineExt, Refined, Solid, Spawned, Spec, SpecError,
};
pub use text::{
    cp1251_to_utf8, cyrillic_to_latin, format_center_text, format_notify_text, format_say_text,
    utf8_to_cp1251,
};
pub use timer::{
    IntoScheduleDelay, ScheduleDelay, Ticks, TimerAction, TimerBound, TimerId, TimerMode,
};

pub use types::{EDict, LIBLIST_FILENAME, LibList, Vector3, bump_map_generation};
