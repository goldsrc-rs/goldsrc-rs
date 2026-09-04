//! Event subscription, semantic phase ordering, and local guest event dispatching.

use crate::dag::{EventPhase, PhasedDag};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

/// Legacy execution priority for event subscribers.
#[deprecated(
    since = "0.17.0",
    note = "Use `EventPhase` instead of numeric `EventPriority`"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    /// Executed first before normal listeners (maps to `EventPhase::Filter`).
    First = 0,
    /// Standard execution priority (maps to `EventPhase::Handle`).
    Normal = 10,
    /// Executed after normal listeners (maps to `EventPhase::Observe`).
    Last = 20,
}

#[allow(deprecated, clippy::derivable_impls)]
impl Default for EventPriority {
    fn default() -> Self {
        Self::Normal
    }
}

#[allow(deprecated)]
impl From<EventPriority> for EventPhase {
    fn from(p: EventPriority) -> Self {
        match p {
            EventPriority::First => EventPhase::Filter,
            EventPriority::Normal => EventPhase::Handle,
            EventPriority::Last => EventPhase::Observe,
        }
    }
}

/// Dynamic event handler closure receiving the raw event payload.
pub type EventHandler = Arc<dyn Fn(&[u8]) + Send + Sync + 'static>;

/// Represents a registered event subscription descriptor.
#[derive(Clone)]
pub struct EventSubscription {
    /// Subscription identifier (used for relative before/after dependencies).
    pub id: String,
    /// Semantic phase of execution.
    pub phase: EventPhase,
    /// Relative dependencies.
    pub before: Vec<String>,
    pub after: Vec<String>,
    /// Event handler closure.
    pub handler: EventHandler,
}

/// Registry of event subscribers indexed by event name and ordered via `PhasedDag`.
#[derive(Default)]
pub struct EventRegistry {
    raw_subscribers: HashMap<String, Vec<EventSubscription>>,
    resolved_cache: HashMap<String, Vec<EventHandler>>,
}

impl EventRegistry {
    /// Creates a new empty event registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribes a handler to the specified event name with a semantic phase and relative order.
    pub fn subscribe(
        &mut self,
        event_name: impl Into<String>,
        id: Option<String>,
        phase: EventPhase,
        before: Vec<String>,
        after: Vec<String>,
        handler: EventHandler,
    ) {
        let name = event_name.into().to_ascii_lowercase();
        let subs = self.raw_subscribers.entry(name.clone()).or_default();
        let sub_id = id.unwrap_or_else(|| format!("{}_{}", name, subs.len()));

        subs.push(EventSubscription {
            id: sub_id,
            phase,
            before,
            after,
            handler,
        });

        // Rebuild resolved cache for this event name using PhasedDag
        self.rebuild_cache_for(&name);
    }

    fn rebuild_cache_for(&mut self, event_name: &str) {
        if let Some(subs) = self.raw_subscribers.get(event_name) {
            let mut dag = PhasedDag::<EventPhase, String, EventHandler>::new();
            for sub in subs {
                dag.add(sub.id.clone(), sub.handler.clone())
                    .phase(sub.phase)
                    .befores(sub.before.clone())
                    .afters(sub.after.clone())
                    .register();
            }

            match dag.resolve_data() {
                Ok(handlers) => {
                    self.resolved_cache.insert(event_name.to_string(), handlers);
                }
                Err(err) => {
                    log::error!(
                        target: "events",
                        "[EventRegistry] Dependency resolution failed for event '{event_name}': {err}"
                    );
                    // Fallback to insertion order if dag resolution errors
                    let handlers: Vec<EventHandler> =
                        subs.iter().map(|s| s.handler.clone()).collect();
                    self.resolved_cache.insert(event_name.to_string(), handlers);
                }
            }
        }
    }

    /// Dispatches an event by name to all registered subscribers in deterministic order.
    pub fn dispatch(&self, event_name: &str, payload: &[u8]) {
        let name = event_name.to_ascii_lowercase();
        if let Some(handlers) = self.resolved_cache.get(&name) {
            for handler in handlers {
                handler(payload);
            }
        }
    }

    /// Clears all event subscriptions.
    pub fn clear(&mut self) {
        self.raw_subscribers.clear();
        self.resolved_cache.clear();
    }
}

static GLOBAL_REGISTRY: LazyLock<RwLock<EventRegistry>> =
    LazyLock::new(|| RwLock::new(EventRegistry::default()));

/// Subscribes to an event in the global registry with semantic phase and relative order.
pub fn subscribe_event(
    event_name: impl Into<String>,
    id: Option<String>,
    phase: EventPhase,
    before: Vec<String>,
    after: Vec<String>,
    handler: impl Fn(&[u8]) + Send + Sync + 'static,
) {
    GLOBAL_REGISTRY
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .subscribe(event_name, id, phase, before, after, Arc::new(handler));
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
    id: Option<String>,
    phase: EventPhase,
    before: Vec<String>,
    after: Vec<String>,
}

impl EventSubscriberBuilder {
    /// Creates a new event subscriber builder.
    pub fn new(event_name: impl Into<String>) -> Self {
        Self {
            event_name: event_name.into(),
            id: None,
            phase: EventPhase::Handle,
            before: Vec::new(),
            after: Vec::new(),
        }
    }

    /// Sets the subscriber ID for relative ordering dependencies.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the semantic phase of event execution.
    pub fn phase(mut self, phase: EventPhase) -> Self {
        self.phase = phase;
        self
    }

    /// Legacy method for priority (maps to `EventPhase`).
    #[allow(deprecated)]
    #[deprecated(since = "0.17.0", note = "Use `.phase(EventPhase)` instead")]
    pub fn priority(mut self, priority: EventPriority) -> Self {
        self.phase = priority.into();
        self
    }

    /// Declares that this subscriber must execute before `target_id`.
    pub fn before(mut self, target_id: impl Into<String>) -> Self {
        self.before.push(target_id.into());
        self
    }

    /// Declares that this subscriber must execute after `target_id`.
    pub fn after(mut self, target_id: impl Into<String>) -> Self {
        self.after.push(target_id.into());
        self
    }

    /// Subscribes the handler function to the event.
    pub fn subscribe<F>(self, handler: F)
    where
        F: Fn(&[u8]) + Send + Sync + 'static,
    {
        subscribe_event(
            self.event_name,
            self.id,
            self.phase,
            self.before,
            self.after,
            handler,
        );
    }
}

/// Event subscription helper entry point.
pub struct Event;

impl Event {
    /// Creates an [`EventSubscriberBuilder`] for the given event name.
    pub fn subscriber(event_name: impl Into<String>) -> EventSubscriberBuilder {
        EventSubscriberBuilder::new(event_name)
    }

    /// Subscribes to an event in the default (`Handle`) phase.
    pub fn subscribe<F>(event_name: impl Into<String>, handler: F)
    where
        F: Fn(&[u8]) + Send + Sync + 'static,
    {
        EventSubscriberBuilder::new(event_name).subscribe(handler);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_phases_and_dag_ordering() {
        let mut registry = EventRegistry::new();
        let log = Arc::new(RwLock::new(Vec::new()));

        let l1 = log.clone();
        registry.subscribe(
            "test_ev",
            Some("observe_logger".to_string()),
            EventPhase::Observe,
            vec![],
            vec![],
            Arc::new(move |_| l1.write().unwrap().push("observe")),
        );

        let l2 = log.clone();
        registry.subscribe(
            "test_ev",
            Some("filter_guard".to_string()),
            EventPhase::Filter,
            vec![],
            vec![],
            Arc::new(move |_| l2.write().unwrap().push("filter")),
        );

        let l3 = log.clone();
        registry.subscribe(
            "test_ev",
            Some("handler_action".to_string()),
            EventPhase::Handle,
            vec![],
            vec![],
            Arc::new(move |_| l3.write().unwrap().push("handle")),
        );

        registry.dispatch("test_ev", b"");
        let result = log.read().unwrap().clone();
        assert_eq!(result, vec!["filter", "handle", "observe"]);
    }
}
