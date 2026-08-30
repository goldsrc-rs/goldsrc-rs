//! Gamedata definitions, signature scanning, and VTable offset configurations.

use std::collections::HashMap;

/// Memory signature pattern for binary scanning.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct MemorySignature {
    /// Name or function symbol identifier.
    pub name: String,
    /// Signature byte string (e.g. `\x55\x8B\xEC\x83\xEC\x20` or hex mask `55 8B EC 83 EC 20 ? ?`).
    pub pattern: String,
    /// Offset from pattern start to target address.
    #[serde(default)]
    pub offset: isize,
}

/// Gamedata definition file model for game and engine offsets.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct GameData {
    /// Mod/Game identifier (e.g. `cstrike`, `valve`, `czero`).
    pub game: String,
    /// Operating system target (`windows` or `linux`).
    pub os: String,
    /// VTable method index mappings (e.g. `TakeDamage => 32`).
    #[serde(default)]
    pub vtable_offsets: HashMap<String, usize>,
    /// Memory signatures for dynamic function resolution.
    #[serde(default)]
    pub signatures: HashMap<String, MemorySignature>,
}

impl GameData {
    /// Parses a gamedata definition from a TOML string.
    pub fn from_toml(content: &str) -> Result<Self, String> {
        toml::from_str(content).map_err(|e| format!("Failed to parse GameData TOML: {e}"))
    }

    /// Gets a VTable offset by method name.
    pub fn get_vtable_offset(&self, method: &str) -> Option<usize> {
        self.vtable_offsets.get(method).copied()
    }
}

/// VTable index constants for standard CBaseEntity / CBasePlayer methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VTableFunc {
    /// `CBaseEntity::Spawn`
    Spawn,
    /// `CBaseEntity::TakeDamage`
    TakeDamage,
    /// `CBaseEntity::Killed`
    Killed,
    /// `CBasePlayer::TraceAttack`
    TraceAttack,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gamedata_toml() {
        let toml_data = r#"
game = "cstrike"
os = "windows"

[vtable_offsets]
Spawn = 0
TakeDamage = 32
Killed = 34
TraceAttack = 48

[signatures.CBasePlayer_TakeDamage]
name = "TakeDamage"
pattern = "55 8B EC 83 EC 20"
offset = 0
"#;
        let gd = GameData::from_toml(toml_data).unwrap();
        assert_eq!(gd.game, "cstrike");
        assert_eq!(gd.get_vtable_offset("TakeDamage"), Some(32));
        assert_eq!(gd.get_vtable_offset("Killed"), Some(34));
    }
}
