//! Persistent repository for moderation records using the host key-value sandbox.
//!
//! NOTE: The specification defines an SQLite WAL backend (`cstrike/data/goldsrc.db`).
//! However, the Wasmtime Component Model host currently does not expose a relational database
//! or SQLite driver interface across the WIT boundary.
//!
//! This repository implements a fallback typed KV-backed store (`moderation/records`),
//! while exposing a clean domain interface that can seamlessly upgrade to SQL once available.

use crate::error::ModerationError;
use crate::record::ModerationRecord;
#[allow(unused_imports)]
use goldsrc::api::bindings::goldsrc::engine::api as host_api;
use std::collections::HashMap;

#[allow(dead_code)]
const STORAGE_BUCKET: &str = "records";
#[allow(dead_code)]
const RECORDS_KEY: &str = "sanctions_v1";

/// Thread-safe in-memory cache and synchronization facade for moderation data.
#[derive(Default)]
pub struct ModerationRepository {
    records: Vec<ModerationRecord>,
    active_freezes: HashMap<i32, f32>, // player_index -> unfreeze_host_time
    active_chat_mutes: HashMap<i32, f32>, // player_index -> unmute_host_time
}

impl ModerationRepository {
    pub fn new() -> Self {
        let mut repo = Self::default();
        repo.load_from_storage();
        repo
    }

    /// Loads persisted sanctions from the host storage sandbox.
    pub fn load_from_storage(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(bytes) = host_api::host_storage_get(STORAGE_BUCKET, RECORDS_KEY) {
                if let Ok(records) = serde_json::from_slice::<Vec<ModerationRecord>>(&bytes) {
                    self.records = records;
                    goldsrc::log_info!(
                        "[Moderation] Loaded {} persistent sanctions from storage",
                        self.records.len()
                    );
                }
            }
        }
    }

    /// Flushes records to host storage.
    pub fn save_to_storage(&self) -> Result<(), ModerationError> {
        #[cfg(target_arch = "wasm32")]
        {
            let data = serde_json::to_vec(&self.records)
                .map_err(|e| ModerationError::Storage(e.to_string()))?;
            let success = host_api::host_storage_set(STORAGE_BUCKET, RECORDS_KEY, &data);
            if !success {
                return Err(ModerationError::Storage(
                    "host_storage_set rejected by host sandbox policy".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Appends a new moderation sanction record.
    pub fn add_record(&mut self, record: ModerationRecord) -> Result<(), ModerationError> {
        self.records.push(record);
        self.save_to_storage()
    }

    /// Deactivates all active sanctions matching target_auth or target_ip.
    pub fn revoke_by_target(&mut self, auth_or_ip: &str) -> Result<usize, ModerationError> {
        let mut count = 0;
        for r in &mut self.records {
            if r.active && (r.target_auth == auth_or_ip || r.target_ip == auth_or_ip) {
                r.active = false;
                count += 1;
            }
        }
        if count > 0 {
            self.save_to_storage()?;
        }
        Ok(count)
    }

    /// Returns all records.
    pub fn all_records(&self) -> &[ModerationRecord] {
        &self.records
    }

    /// In-memory freeze management.
    pub fn set_player_freeze(&mut self, player_index: i32, until_time: f32) {
        self.active_freezes.insert(player_index, until_time);
    }

    pub fn is_player_frozen(&self, player_index: i32, current_time: f32) -> bool {
        if let Some(&until) = self.active_freezes.get(&player_index) {
            current_time < until
        } else {
            false
        }
    }

    pub fn unfreeze_player(&mut self, player_index: i32) {
        self.active_freezes.remove(&player_index);
    }

    /// In-memory chat mute / gag management.
    pub fn set_chat_mute(&mut self, player_index: i32, until_time: f32) {
        self.active_chat_mutes.insert(player_index, until_time);
    }

    pub fn is_chat_muted(&self, player_index: i32, current_time: f32) -> bool {
        if let Some(&until) = self.active_chat_mutes.get(&player_index) {
            current_time < until
        } else {
            false
        }
    }

    pub fn unmute_chat(&mut self, player_index: i32) {
        self.active_chat_mutes.remove(&player_index);
    }

    /// Removes expired session flags on tick.
    pub fn tick_cleanup(&mut self, current_time: f32) {
        self.active_freezes
            .retain(|_, &mut until| current_time < until);
        self.active_chat_mutes
            .retain(|_, &mut until| current_time < until);
    }
}
