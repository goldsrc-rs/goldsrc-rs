//! Event subscription, priority ordering, and local guest event dispatching.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

/// Execution priority for event subscribers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum EventPriority {
    /// Executed first before normal listeners.
    First = 0,
    /// Standard execution priority.
    #[default]
    Normal = 10,
    /// Executed after normal listeners.
    Last = 20,
}

/// Dynamic event handler closure receiving the raw event payload.
pub type EventHandler = Arc<dyn Fn(&[u8]) + Send + Sync + 'static>;

/// Represents a registered event subscription.
#[derive(Clone)]
pub struct EventSubscription {
    /// Subscription execution priority.
    pub priority: EventPriority,
    /// Event handler closure.
    pub handler: EventHandler,
}

/// Registry of event subscribers indexed by event name.
#[derive(Default)]
pub struct EventRegistry {
    subscribers: HashMap<String, Vec<EventSubscription>>,
}

impl EventRegistry {
    /// Creates a new empty event registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribes a handler to the specified event name with a given priority.
    pub fn subscribe(
        &mut self,
        event_name: impl Into<String>,
        priority: EventPriority,
        handler: EventHandler,
    ) {
        let name = event_name.into().to_ascii_lowercase();
        let subs = self.subscribers.entry(name).or_default();
        subs.push(EventSubscription { priority, handler });
        subs.sort_by_key(|s| s.priority);
    }

    /// Dispatches an event by name to all registered subscribers.
    pub fn dispatch(&self, event_name: &str, payload: &[u8]) {
        let name = event_name.to_ascii_lowercase();
        if let Some(subs) = self.subscribers.get(&name) {
            for sub in subs {
                (sub.handler)(payload);
            }
        }
    }

    /// Clears all event subscriptions.
    pub fn clear(&mut self) {
        self.subscribers.clear();
    }
}

static GLOBAL_REGISTRY: LazyLock<RwLock<EventRegistry>> =
    LazyLock::new(|| RwLock::new(EventRegistry::default()));

/// Subscribes to an event in the global registry.
pub fn subscribe_event(
    event_name: impl Into<String>,
    priority: EventPriority,
    handler: impl Fn(&[u8]) + Send + Sync + 'static,
) {
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .subscribe(event_name, priority, Arc::new(handler));
}

/// Dispatches an event through the global registry.
pub fn dispatch_event(event_name: &str, payload: &[u8]) {
    GLOBAL_REGISTRY
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .dispatch(event_name, payload);
}

/// Clears all events in the global registry.
pub fn clear_events() {
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
}

/// Fluent builder for event subscriptions.
#[derive(Debug, Clone)]
pub struct EventSubscriberBuilder {
    event_name: String,
    priority: EventPriority,
}

impl EventSubscriberBuilder {
    /// Creates a new event subscriber builder.
    pub fn new(event_name: impl Into<String>) -> Self {
        Self {
            event_name: event_name.into(),
            priority: EventPriority::Normal,
        }
    }

    /// Sets the subscriber priority.
    pub fn priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Subscribes the handler function to the event.
    pub fn subscribe<F>(self, handler: F)
    where
        F: Fn(&[u8]) + Send + Sync + 'static,
    {
        subscribe_event(self.event_name, self.priority, handler);
    }
}

/// Event subscription helper entry point.
pub struct Event;

impl Event {
    /// Creates an [`EventSubscriberBuilder`] for the given event name.
    pub fn subscriber(event_name: impl Into<String>) -> EventSubscriberBuilder {
        EventSubscriberBuilder::new(event_name)
    }

    /// Subscribes to an event with default (`Normal`) priority.
    pub fn subscribe<F>(event_name: impl Into<String>, handler: F)
    where
        F: Fn(&[u8]) + Send + Sync + 'static,
    {
        subscribe_event(event_name, EventPriority::Normal, handler);
    }
}
