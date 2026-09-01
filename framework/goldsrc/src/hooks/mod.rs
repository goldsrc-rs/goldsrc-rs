//! Centralized hook dispatchers, types, and entity VTable hooks.

#[cfg(feature = "host")]
pub mod dispatcher;
#[cfg(feature = "host")]
pub mod entity;
pub mod types;

#[cfg(feature = "host")]
pub use dispatcher::{
    dispatch_client_command, dispatch_command, emit_event, emit_player_event,
    on_client_user_info_changed, on_server_activate, on_server_deactivate, on_server_frame,
};
#[cfg(feature = "host")]
pub use entity::{EntityHookRegistry, KilledContext, TakeDamageContext, entity_hooks};
pub use types::{HookResult, HookTiming};
