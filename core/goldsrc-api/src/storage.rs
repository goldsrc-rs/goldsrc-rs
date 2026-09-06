//! Dual Storage Port Abstraction (raw bytes, no serialization format imposed).
//!
//! Provides decoupled KV storage ([`StorageProvider`]) with atomic operations
//! and relational query interface ([`SqlDatabase`]). Typed serialization
//! (`Bucket<T>`) lives in `framework/goldsrc` where the format can be chosen
//! per-plugin (JSON, bincode, raw bytes).

/// Domain errors occurring during storage operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    /// Requested key or bucket was not found.
    NotFound { bucket: String, key: String },
    /// Unauthorized access to a private bucket outside allowlist.
    Unauthorized { bucket: String, caller: String },
    /// Serialization or deserialization failure.
    Serialization(String),
    /// Underlying storage backend error.
    Backend(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { bucket, key } => {
                write!(f, "Key not found in bucket '{bucket}': '{key}'")
            }
            Self::Unauthorized { bucket, caller } => {
                write!(
                    f,
                    "Unauthorized bucket access: '{bucket}' (caller '{caller}')"
                )
            }
            Self::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            Self::Backend(msg) => write!(f, "Backend error: {msg}"),
        }
    }
}

impl std::error::Error for StorageError {}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

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

    #[test]
    fn test_raw_storage_lifecycle() {
        let mock = Arc::new(MockStorage::default());
        mock.set("prefs", "k1", b"hello").unwrap();
        assert_eq!(mock.get("prefs", "k1").unwrap(), Some(b"hello".to_vec()));
        assert_eq!(mock.get("prefs", "missing").unwrap(), None);
        assert!(mock.delete("prefs", "k1").unwrap());
        assert_eq!(mock.get("prefs", "k1").unwrap(), None);
    }

    #[test]
    fn test_atomic_fetch_add() {
        let mock = Arc::new(MockStorage::default());
        let b1 = mock.fetch_add("bank", "STEAM_0:0:1", 500).unwrap();
        assert_eq!(b1, 500);
        let b2 = mock.fetch_add("bank", "STEAM_0:0:1", -150).unwrap();
        assert_eq!(b2, 350);
    }
}
