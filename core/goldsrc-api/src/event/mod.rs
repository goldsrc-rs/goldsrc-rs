//! Event subscription, priority ordering, and local guest event dispatching.

pub mod registry;

pub use crate::dag::EventPhase;
pub use registry::{
    Event, EventHandler, EventRegistry, EventSubscriberBuilder, EventSubscription, clear_events,
    dispatch_event, subscribe_event,
};
