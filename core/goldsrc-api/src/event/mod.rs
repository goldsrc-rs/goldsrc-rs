//! Event subscription, priority ordering, and local guest event dispatching.

pub mod registry;

pub use registry::{
    Event, EventHandler, EventPriority, EventRegistry, EventSubscriberBuilder, EventSubscription,
    clear_events, dispatch_event, subscribe_event,
};
