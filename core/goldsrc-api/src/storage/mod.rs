//! Dual Storage Port Abstraction & Typed Bucket Facade.
//!
//! Provides decoupled KV storage ([`StorageProvider`]) with atomic operations,
//! relational query interface ([`SqlDatabase`]), and strongly-typed [`Bucket<T>`]
//! for plugin state persistence without main-thread I/O stalls.

use std::marker::PhantomData;
use std::sync::Arc;

/// Domain errors occurring during storage operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StorageError {
    /// Requested key or bucket was not found.
    #[error("Key not found in bucket '{bucket}': '{key}'")]
    NotFound { bucket: String, key: String },
    /// Unauthorized access to a private bucket outside allowlist.
    #[error("Unauthorized bucket access: '{bucket}' (caller '{caller}')")]
    Unauthorized { bucket: String, caller: String },
    /// Serialization or deserialization failure.
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// Underlying storage backend error.
    #[error("Backend error: {0}")]
    Backend(String),
}

/// Key-Value Storage Provider trait (KV Port).
///
/// Implemented by host engines (e.g. SQLite WAL, Redb, Mock).
pub trait StorageProvider: Send + Sync {
    /// Retrieves a binary value by key in the specified bucket.
    fn get(&self, bucket: &str, key: &str) -> Result<Option<Vec<u8>>, StorageError>;

    /// Stores a binary value by key in the specified bucket.
    fn set(&self, bucket: &str, key: &str, value: &[u8]) -> Result<(), StorageError>;

    /// Deletes a key from the specified bucket.
    fn delete(&self, bucket: &str, key: &str) -> Result<bool, StorageError>;

    /// Atomically increments/decrements a 64-bit signed integer value and returns the new value.
    fn fetch_add(&self, bucket: &str, key: &str, delta: i64) -> Result<i64, StorageError>;
}

/// Relational / Analytical Query trait (SQL Port).
///
/// Implemented by host engines for complex aggregations (e.g. TOP15, ELO rankings).
pub trait SqlDatabase: Send + Sync {
    /// Executes a SQL statement with parameters and returns affected rows.
    fn execute(
        &self,
        sql: &str,
        params: &[&dyn rusqlite_param::ToSqlParam],
    ) -> Result<usize, StorageError>;
}

/// Mock parameter marker for SQL params across FFI boundaries.
pub mod rusqlite_param {
    pub trait ToSqlParam: Send + Sync {
        fn as_str(&self) -> Option<&str> {
            None
        }
        fn as_i64(&self) -> Option<i64> {
            None
        }
        fn as_f64(&self) -> Option<f64> {
            None
        }
        fn as_bytes(&self) -> Option<&[u8]> {
            None
        }
    }

    impl ToSqlParam for String {
        fn as_str(&self) -> Option<&str> {
            Some(self.as_str())
        }
    }
    impl ToSqlParam for &str {
        fn as_str(&self) -> Option<&str> {
            Some(self)
        }
    }
    impl ToSqlParam for i64 {
        fn as_i64(&self) -> Option<i64> {
            Some(*self)
        }
    }
    impl ToSqlParam for i32 {
        fn as_i64(&self) -> Option<i64> {
            Some(*self as i64)
        }
    }
    impl ToSqlParam for f64 {
        fn as_f64(&self) -> Option<f64> {
            Some(*self)
        }
    }
    impl ToSqlParam for f32 {
        fn as_f64(&self) -> Option<f64> {
            Some(*self as f64)
        }
    }
    impl ToSqlParam for Vec<u8> {
        fn as_bytes(&self) -> Option<&[u8]> {
            Some(self.as_slice())
        }
    }
    impl ToSqlParam for &[u8] {
        fn as_bytes(&self) -> Option<&[u8]> {
            Some(self)
        }
    }
}

/// Strongly-typed Key-Value Bucket facade for plugins.
///
/// Wraps a [`StorageProvider`] reference with JSON serialization.
#[derive(Clone)]
pub struct Bucket<T> {
    provider: Arc<dyn StorageProvider>,
    name: String,
    _marker: PhantomData<T>,
}

impl<T> Bucket<T> {
    /// Creates a new typed bucket handle.
    pub fn new<S: Into<String>>(provider: Arc<dyn StorageProvider>, name: S) -> Self {
        Self {
            provider,
            name: name.into(),
            _marker: PhantomData,
        }
    }

    /// Returns the bucket name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<T: serde::de::DeserializeOwned> Bucket<T> {
    /// Retrieves and deserializes a value by key.
    pub fn get(&self, key: &str) -> Result<Option<T>, StorageError> {
        let raw = self.provider.get(&self.name, key)?;
        match raw {
            Some(bytes) => {
                let val: T = serde_json::from_slice(&bytes)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }
}

impl<T: serde::Serialize> Bucket<T> {
    /// Serializes and stores a value by key.
    pub fn set(&self, key: &str, value: &T) -> Result<(), StorageError> {
        let bytes =
            serde_json::to_vec(value).map_err(|e| StorageError::Serialization(e.to_string()))?;
        self.provider.set(&self.name, key, &bytes)
    }

    /// Deletes a key from the bucket.
    pub fn delete(&self, key: &str) -> Result<bool, StorageError> {
        self.provider.delete(&self.name, key)
    }
}

impl Bucket<i64> {
    /// Atomically increments/decrements an integer balance and returns the new value.
    pub fn fetch_add(&self, key: &str, delta: i64) -> Result<i64, StorageError> {
        self.provider.fetch_add(&self.name, key, delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::RwLock;

    #[derive(Default)]
    struct MockStorage(RwLock<HashMap<(String, String), Vec<u8>>>);

    impl StorageProvider for MockStorage {
        fn get(&self, bucket: &str, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
            let map = self.0.read().unwrap();
            Ok(map.get(&(bucket.to_string(), key.to_string())).cloned())
        }

        fn set(&self, bucket: &str, key: &str, value: &[u8]) -> Result<(), StorageError> {
            let mut map = self.0.write().unwrap();
            map.insert((bucket.to_string(), key.to_string()), value.to_vec());
            Ok(())
        }

        fn delete(&self, bucket: &str, key: &str) -> Result<bool, StorageError> {
            let mut map = self.0.write().unwrap();
            Ok(map.remove(&(bucket.to_string(), key.to_string())).is_some())
        }

        fn fetch_add(&self, bucket: &str, key: &str, delta: i64) -> Result<i64, StorageError> {
            let mut map = self.0.write().unwrap();
            let key_tuple = (bucket.to_string(), key.to_string());
            let current = match map.get(&key_tuple) {
                Some(bytes) => i64::from_le_bytes(bytes.as_slice().try_into().unwrap_or([0; 8])),
                None => 0,
            };
            let new_val = current + delta;
            map.insert(key_tuple, new_val.to_le_bytes().to_vec());
            Ok(new_val)
        }
    }

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct PlayerPref {
        hud_enabled: bool,
        chat_color: String,
    }

    #[test]
    fn test_typed_bucket_lifecycle() {
        let mock = Arc::new(MockStorage::default());
        let bucket = Bucket::<PlayerPref>::new(mock.clone(), "player_prefs");

        let pref = PlayerPref {
            hud_enabled: true,
            chat_color: "green".to_string(),
        };

        bucket.set("STEAM_0:0:12345", &pref).unwrap();

        let retrieved = bucket.get("STEAM_0:0:12345").unwrap();
        assert_eq!(retrieved, Some(pref));

        let missing = bucket.get("STEAM_0:0:99999").unwrap();
        assert_eq!(missing, None);

        let deleted = bucket.delete("STEAM_0:0:12345").unwrap();
        assert!(deleted);
        assert_eq!(bucket.get("STEAM_0:0:12345").unwrap(), None);
    }

    #[test]
    fn test_atomic_fetch_add() {
        let mock = Arc::new(MockStorage::default());
        let bank = Bucket::<i64>::new(mock.clone(), "bank");

        let b1 = bank.fetch_add("STEAM_0:0:1", 500).unwrap();
        assert_eq!(b1, 500);

        let b2 = bank.fetch_add("STEAM_0:0:1", -150).unwrap();
        assert_eq!(b2, 350);
    }
}
