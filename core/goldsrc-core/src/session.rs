//! Unified client session manager, userinfo overrides, and ephemeral player state.

use std::collections::HashMap;

/// Ephemeral session state for a connected client slot (1..=32).
#[derive(Debug, Clone, Default)]
pub struct ClientSession {
    /// Slot index of the client (1..=32).
    pub slot: i32,
    /// Overridden or cached userinfo key-value pairs (e.g. "_lang", "rate", "name").
    pub userinfo_overrides: HashMap<String, String>,
    /// Custom plugin/system metadata or tags associated with this session.
    pub metadata: HashMap<String, String>,
}

impl ClientSession {
    /// Creates a new empty session for the given player slot.
    pub fn new(slot: i32) -> Self {
        Self {
            slot,
            userinfo_overrides: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Gets an overridden userinfo value by key (case-insensitive).
    pub fn get_userinfo(&self, key: &str) -> Option<&str> {
        let key_lower = key.to_ascii_lowercase();
        self.userinfo_overrides.get(&key_lower).map(|s| s.as_str())
    }

    /// Sets or overrides a userinfo value by key (case-insensitive).
    pub fn set_userinfo(&mut self, key: &str, value: impl Into<String>) {
        self.userinfo_overrides
            .insert(key.to_ascii_lowercase(), value.into());
    }

    /// Removes a userinfo override by key (case-insensitive).
    pub fn remove_userinfo(&mut self, key: &str) -> Option<String> {
        self.userinfo_overrides.remove(&key.to_ascii_lowercase())
    }

    /// Returns the active language for this session if overridden.
    pub fn lang(&self) -> Option<&str> {
        for key in ["_lang", "lang", "_cl_lang", "cl_lang"] {
            if let Some(val) = self.get_userinfo(key) {
                return Some(val);
            }
        }
        None
    }

    /// Sets the language preference for this session.
    pub fn set_lang(&mut self, lang: &str) {
        self.set_userinfo("_lang", lang);
        self.set_userinfo("lang", lang);
    }
}

/// Thread-safe manager for all connected client sessions (slots 1..=32).
#[derive(Debug, Default)]
pub struct ClientSessionManager {
    sessions: HashMap<i32, ClientSession>,
}

impl ClientSessionManager {
    /// Creates an empty session manager.
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Retrieves a reference to the client session for the given slot.
    pub fn get(&self, slot: i32) -> Option<&ClientSession> {
        self.sessions.get(&slot)
    }

    /// Retrieves a mutable reference to the client session for the given slot, creating it if absent.
    pub fn get_or_create_mut(&mut self, slot: i32) -> &mut ClientSession {
        self.sessions
            .entry(slot)
            .or_insert_with(|| ClientSession::new(slot))
    }

    /// Removes session state when a client disconnects, preventing memory leaks.
    pub fn on_disconnect(&mut self, slot: i32) -> Option<ClientSession> {
        self.sessions.remove(&slot)
    }

    /// Clears all active client sessions (e.g. on map change or server shutdown).
    pub fn clear(&mut self) {
        self.sessions.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_session_userinfo_and_lang() {
        let mut mgr = ClientSessionManager::new();
        let session = mgr.get_or_create_mut(1);

        assert_eq!(session.lang(), None);
        session.set_lang("ru");
        assert_eq!(session.lang(), Some("ru"));
        assert_eq!(session.get_userinfo("lang"), Some("ru"));
        assert_eq!(session.get_userinfo("_lang"), Some("ru"));

        session.set_userinfo("rate", "25000");
        assert_eq!(session.get_userinfo("RATE"), Some("25000"));

        mgr.on_disconnect(1);
        assert!(mgr.get(1).is_none());
    }
}
