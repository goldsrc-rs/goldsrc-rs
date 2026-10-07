//! Staff RBAC model and staff registry persistence.

use crate::error::AdministrationError;
#[allow(unused_imports)]
use goldsrc::api::bindings::goldsrc::engine::api as host_api;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
const STAFF_BUCKET: &str = "admins";
#[allow(dead_code)]
const STAFF_KEY: &str = "staff_registry_v1";

/// Role hierarchy for server staff members.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaffRole {
    Moderator,
    Admin,
    SuperAdmin,
    HeadAdmin,
}

impl StaffRole {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Moderator => "moderator",
            Self::Admin => "admin",
            Self::SuperAdmin => "super_admin",
            Self::HeadAdmin => "head_admin",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "moderator" | "mod" => Some(Self::Moderator),
            "admin" | "adm" => Some(Self::Admin),
            "superadmin" | "super_admin" => Some(Self::SuperAdmin),
            "headadmin" | "head_admin" | "root" => Some(Self::HeadAdmin),
            _ => None,
        }
    }
}

/// Persistent record of an authorized staff member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffMember {
    /// SteamID or canonical authorization token.
    pub auth_id: String,
    /// Staff role level.
    pub role: StaffRole,
    /// Granular granted capabilities.
    pub capabilities: Vec<String>,
    /// Authorized by.
    pub added_by: String,
    /// UNIX timestamp.
    pub added_at: u64,
    /// Expiration timestamp (0 = permanent).
    pub expires_at: u64,
}

/// Registry of authorized staff accounts.
#[derive(Default)]
pub struct StaffRegistry {
    members: Vec<StaffMember>,
}

impl StaffRegistry {
    pub fn new() -> Self {
        let mut reg = Self::default();
        reg.load();
        reg
    }

    pub fn load(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(bytes) = host_api::host_storage_get(STAFF_BUCKET, STAFF_KEY) {
                if let Ok(members) = serde_json::from_slice::<Vec<StaffMember>>(&bytes) {
                    self.members = members;
                }
            }
        }
    }

    pub fn save(&self) -> Result<(), AdministrationError> {
        #[cfg(target_arch = "wasm32")]
        {
            let data = serde_json::to_vec(&self.members)
                .map_err(|e| AdministrationError::Storage(e.to_string()))?;
            if !host_api::host_storage_set(STAFF_BUCKET, STAFF_KEY, &data) {
                return Err(AdministrationError::Storage(
                    "host_storage_set rejected by host sandbox policy".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn add(&mut self, member: StaffMember) -> Result<(), AdministrationError> {
        self.members.retain(|m| m.auth_id != member.auth_id);
        self.members.push(member);
        self.save()
    }

    pub fn revoke(&mut self, auth_id: &str) -> Result<bool, AdministrationError> {
        let prev = self.members.len();
        self.members.retain(|m| m.auth_id != auth_id);
        let changed = self.members.len() < prev;
        if changed {
            self.save()?;
        }
        Ok(changed)
    }

    pub fn list(&self) -> &[StaffMember] {
        &self.members
    }
}
