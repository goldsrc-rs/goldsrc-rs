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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VTableFunc {
    /// `CBaseEntity::Spawn` (Index 0)
    Spawn,
    /// `CBaseEntity::Precache` (Index 1)
    Precache,
    /// `CBaseEntity::KeyValue` (Index 2)
    KeyValue,
    /// `CBaseEntity::Save` (Index 3)
    Save,
    /// `CBaseEntity::Restore` (Index 4)
    Restore,
    /// `CBaseEntity::ObjectCaps` (Index 5)
    ObjectCaps,
    /// `CBaseEntity::Activate` (Index 6)
    Activate,
    /// `CBaseEntity::SetObjectCollisionBox` (Index 7)
    SetObjectCollisionBox,
    /// `CBaseEntity::Classify` (Index 8)
    Classify,
    /// `CBaseEntity::DeathNotice` (Index 9)
    DeathNotice,
    /// `CBaseEntity::TraceAttack` (Index 10)
    TraceAttack,
    /// `CBaseEntity::TakeDamage` (Index 11)
    TakeDamage,
    /// `CBaseEntity::TakeHealth` (Index 12)
    TakeHealth,
    /// `CBaseEntity::Killed` (Index 13)
    Killed,
    /// `CBaseEntity::BloodColor` (Index 14)
    BloodColor,
    /// `CBaseEntity::TraceBleed` (Index 15)
    TraceBleed,
    /// `CBaseEntity::IsTriggered` (Index 16)
    IsTriggered,
    /// `CBaseEntity::GetToggleState` (Index 19)
    GetToggleState,
    /// `CBaseEntity::AddPoints` (Index 20)
    AddPoints,
    /// `CBaseEntity::AddPointsToTeam` (Index 21)
    AddPointsToTeam,
    /// `CBaseEntity::AddPlayerItem` (Index 22)
    AddPlayerItem,
    /// `CBaseEntity::RemovePlayerItem` (Index 23)
    RemovePlayerItem,
    /// `CBaseEntity::GiveAmmo` (Index 24)
    GiveAmmo,
    /// `CBaseEntity::GetDelay` (Index 25)
    GetDelay,
    /// `CBaseEntity::IsMoving` (Index 26)
    IsMoving,
    /// `CBaseEntity::DamageDecal` (Index 28)
    DamageDecal,
    /// `CBaseEntity::SetToggleState` (Index 29)
    SetToggleState,
    /// `CBaseEntity::StartSneaking` (Index 30)
    StartSneaking,
    /// `CBaseEntity::StopSneaking` (Index 31)
    StopSneaking,
    /// `CBaseEntity::OnControls` (Index 32)
    OnControls,
    /// `CBaseEntity::IsSneaking` (Index 33)
    IsSneaking,
    /// `CBaseEntity::IsAlive` (Index 34)
    IsAlive,
    /// `CBaseEntity::IsBSPModel` (Index 35)
    IsBSPModel,
    /// `CBaseEntity::ReflectGauss` (Index 36)
    ReflectGauss,
    /// `CBaseEntity::HasTarget` (Index 37)
    HasTarget,
    /// `CBaseEntity::IsInWorld` (Index 38)
    IsInWorld,
    /// `CBaseEntity::IsPlayer` (Index 39)
    IsPlayer,
    /// `CBaseEntity::IsNetClient` (Index 40)
    IsNetClient,
    /// `CBaseEntity::TeamID` (Index 41)
    TeamID,
    /// `CBaseEntity::GetNextTarget` (Index 42)
    GetNextTarget,
    /// `CBaseEntity::Think` (Index 43)
    Think,
    /// `CBaseEntity::Touch` (Index 44)
    Touch,
    /// `CBaseEntity::Use` (Index 45)
    Use,
    /// `CBaseEntity::Blocked` (Index 46)
    Blocked,
    /// `CBaseEntity::Respawn` (Index 47)
    Respawn,
    /// `CBaseEntity::UpdateOwner` (Index 48)
    UpdateOwner,
    /// `CBaseEntity::FBecomeProne` (Index 49)
    FBecomeProne,
    /// `CBaseEntity::Center` (Index 50)
    Center,
    /// `CBaseEntity::EyePosition` (Index 51)
    EyePosition,
    /// `CBaseEntity::EarPosition` (Index 52)
    EarPosition,
    /// `CBaseEntity::BodyTarget` (Index 53)
    BodyTarget,
    /// `CBaseEntity::Illumination` (Index 54)
    Illumination,
    /// `CBaseEntity::FVisible` (Index 55)
    FVisible,
    /// `CBasePlayer::Jump`
    Jump,
    /// `CBasePlayer::Duck`
    Duck,
    /// `CBasePlayer::PreThink`
    PreThink,
    /// `CBasePlayer::PostThink`
    PostThink,
    /// `CBasePlayer::GetGunPosition`
    GetGunPosition,
    /// `CBasePlayer::UpdateClientData`
    UpdateClientData,
    /// `CBasePlayer::ResetMaxSpeed` (Game-specific / CS 1.6)
    ResetMaxSpeed,
    /// `CBasePlayerItem::AddToPlayer`
    AddToPlayer,
    /// `CBasePlayerItem::AddDuplicate`
    AddDuplicate,
    /// `CBasePlayerItem::GetItemInfo`
    GetItemInfo,
    /// `CBasePlayerItem::CanDeploy`
    CanDeploy,
    /// `CBasePlayerItem::Deploy`
    Deploy,
    /// `CBasePlayerItem::CanHolster`
    CanHolster,
    /// `CBasePlayerItem::Holster`
    Holster,
    /// `CBasePlayerItem::UpdateItemInfo`
    UpdateItemInfo,
    /// `CBasePlayerItem::ItemPreFrame`
    ItemPreFrame,
    /// `CBasePlayerItem::ItemPostFrame`
    ItemPostFrame,
    /// `CBasePlayerItem::Drop`
    Drop,
    /// `CBasePlayerItem::Kill`
    Kill,
    /// `CBasePlayerItem::AttachToPlayer`
    AttachToPlayer,
    /// `CBasePlayerWeapon::ExtractAmmo`
    ExtractAmmo,
    /// `CBasePlayerWeapon::ExtractClipAmmo`
    ExtractClipAmmo,
    /// `CBasePlayerWeapon::AddWeapon`
    AddWeapon,
    /// `CBasePlayerWeapon::PlayEmptySound`
    PlayEmptySound,
    /// `CBasePlayerWeapon::ResetEmptySound`
    ResetEmptySound,
    /// `CBasePlayerWeapon::SendWeaponAnim`
    SendWeaponAnim,
    /// `CBasePlayerWeapon::IsUseable`
    IsUseable,
    /// `CBasePlayerWeapon::PrimaryAttack`
    PrimaryAttack,
    /// `CBasePlayerWeapon::SecondaryAttack`
    SecondaryAttack,
    /// `CBasePlayerWeapon::Reload`
    Reload,
    /// `CBasePlayerWeapon::WeaponIdle`
    WeaponIdle,
    /// `CBasePlayerWeapon::RetireWeapon`
    RetireWeapon,
    /// `CBasePlayerWeapon::ShouldWeaponIdle`
    ShouldWeaponIdle,
    /// `CBasePlayerWeapon::UseDecrement`
    UseDecrement,
}

impl VTableFunc {
    /// Returns the standard canonical method name in gamedata TOML configs.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Spawn => "Spawn",
            Self::Precache => "Precache",
            Self::KeyValue => "KeyValue",
            Self::Save => "Save",
            Self::Restore => "Restore",
            Self::ObjectCaps => "ObjectCaps",
            Self::Activate => "Activate",
            Self::SetObjectCollisionBox => "SetObjectCollisionBox",
            Self::Classify => "Classify",
            Self::DeathNotice => "DeathNotice",
            Self::TraceAttack => "TraceAttack",
            Self::TakeDamage => "TakeDamage",
            Self::TakeHealth => "TakeHealth",
            Self::Killed => "Killed",
            Self::BloodColor => "BloodColor",
            Self::TraceBleed => "TraceBleed",
            Self::IsTriggered => "IsTriggered",
            Self::GetToggleState => "GetToggleState",
            Self::AddPoints => "AddPoints",
            Self::AddPointsToTeam => "AddPointsToTeam",
            Self::AddPlayerItem => "AddPlayerItem",
            Self::RemovePlayerItem => "RemovePlayerItem",
            Self::GiveAmmo => "GiveAmmo",
            Self::GetDelay => "GetDelay",
            Self::IsMoving => "IsMoving",
            Self::DamageDecal => "DamageDecal",
            Self::SetToggleState => "SetToggleState",
            Self::StartSneaking => "StartSneaking",
            Self::StopSneaking => "StopSneaking",
            Self::OnControls => "OnControls",
            Self::IsSneaking => "IsSneaking",
            Self::IsAlive => "IsAlive",
            Self::IsBSPModel => "IsBSPModel",
            Self::ReflectGauss => "ReflectGauss",
            Self::HasTarget => "HasTarget",
            Self::IsInWorld => "IsInWorld",
            Self::IsPlayer => "IsPlayer",
            Self::IsNetClient => "IsNetClient",
            Self::TeamID => "TeamID",
            Self::GetNextTarget => "GetNextTarget",
            Self::Think => "Think",
            Self::Touch => "Touch",
            Self::Use => "Use",
            Self::Blocked => "Blocked",
            Self::Respawn => "Respawn",
            Self::UpdateOwner => "UpdateOwner",
            Self::FBecomeProne => "FBecomeProne",
            Self::Center => "Center",
            Self::EyePosition => "EyePosition",
            Self::EarPosition => "EarPosition",
            Self::BodyTarget => "BodyTarget",
            Self::Illumination => "Illumination",
            Self::FVisible => "FVisible",
            Self::Jump => "Jump",
            Self::Duck => "Duck",
            Self::PreThink => "PreThink",
            Self::PostThink => "PostThink",
            Self::GetGunPosition => "GetGunPosition",
            Self::UpdateClientData => "UpdateClientData",
            Self::ResetMaxSpeed => "ResetMaxSpeed",
            Self::AddToPlayer => "AddToPlayer",
            Self::AddDuplicate => "AddDuplicate",
            Self::GetItemInfo => "GetItemInfo",
            Self::CanDeploy => "CanDeploy",
            Self::Deploy => "Deploy",
            Self::CanHolster => "CanHolster",
            Self::Holster => "Holster",
            Self::UpdateItemInfo => "UpdateItemInfo",
            Self::ItemPreFrame => "ItemPreFrame",
            Self::ItemPostFrame => "ItemPostFrame",
            Self::Drop => "Drop",
            Self::Kill => "Kill",
            Self::AttachToPlayer => "AttachToPlayer",
            Self::ExtractAmmo => "ExtractAmmo",
            Self::ExtractClipAmmo => "ExtractClipAmmo",
            Self::AddWeapon => "AddWeapon",
            Self::PlayEmptySound => "PlayEmptySound",
            Self::ResetEmptySound => "ResetEmptySound",
            Self::SendWeaponAnim => "SendWeaponAnim",
            Self::IsUseable => "IsUseable",
            Self::PrimaryAttack => "PrimaryAttack",
            Self::SecondaryAttack => "SecondaryAttack",
            Self::Reload => "Reload",
            Self::WeaponIdle => "WeaponIdle",
            Self::RetireWeapon => "RetireWeapon",
            Self::ShouldWeaponIdle => "ShouldWeaponIdle",
            Self::UseDecrement => "UseDecrement",
        }
    }
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
