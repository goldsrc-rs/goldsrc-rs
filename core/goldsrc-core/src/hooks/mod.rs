//! Centralized hook dispatchers, types, and entity VTable hooks.

pub mod dispatcher;

pub mod entity;
pub mod types;

pub use dispatcher::{
    dispatch_client_command, dispatch_command, emit_event, emit_player_event,
    on_client_user_info_changed, on_server_activate, on_server_deactivate, on_server_frame,
};

pub use entity::{EntityHookRegistry, KilledContext, TakeDamageContext, entity_hooks};
pub use types::{HookResult, HookTiming};
