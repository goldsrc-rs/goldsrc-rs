//! Centralized hook dispatchers, types, and entity VTable hooks.

pub mod dispatcher;

pub mod entity;
pub mod types;

pub use dispatcher::{dispatch_client_command, dispatch_command, emit};

pub use entity::{EntityHookRegistry, KilledContext, TakeDamageContext, entity_hooks};
pub use types::{HookResult, HookTiming};
