//! Counter-Strike 1.6 domain abstractions, weapon constants, equipment, and gameplay rules.

use crate::client::{Player, Team};

/// Counter-Strike 1.6 weapon identifiers (matching CS 1.6 `CSW_*` constants).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum CsWeapon {
    None = 0,
    P228 = 1,
    Glock = 2,
    Scout = 3,
    HeGrenade = 4,
    Xm1014 = 5,
    C4 = 6,
    Mac10 = 7,
    Aug = 8,
    SmokeGrenade = 9,
    Elite = 10,
    Fiveseven = 11,
    Usp = 12,
    Glock18 = 13,
    AwP = 14,
    Mp5Navy = 15,
    M249 = 16,
    M3 = 17,
    M4a1 = 18,
    Tmp = 19,
    G3sg1 = 20,
    Flashbang = 21,
    Deagle = 22,
    Sg552 = 23,
    Ak47 = 24,
    Knife = 25,
    P90 = 26,
}

impl CsWeapon {
    /// Returns the canonical entity classname for spawning (e.g. `"weapon_m4a1"`).
    pub const fn classname(&self) -> &'static str {
        match self {
            CsWeapon::P228 => "weapon_p228",
            CsWeapon::Scout => "weapon_scout",
            CsWeapon::HeGrenade => "weapon_hegrenade",
            CsWeapon::Xm1014 => "weapon_xm1014",
            CsWeapon::C4 => "weapon_c4",
            CsWeapon::Mac10 => "weapon_mac10",
            CsWeapon::Aug => "weapon_aug",
            CsWeapon::SmokeGrenade => "weapon_smokegrenade",
            CsWeapon::Elite => "weapon_elite",
            CsWeapon::Fiveseven => "weapon_fiveseven",
            CsWeapon::Usp => "weapon_usp",
            CsWeapon::Glock18 | CsWeapon::Glock => "weapon_glock18",
            CsWeapon::AwP => "weapon_awp",
            CsWeapon::Mp5Navy => "weapon_mp5navy",
            CsWeapon::M249 => "weapon_m249",
            CsWeapon::M3 => "weapon_m3",
            CsWeapon::M4a1 => "weapon_m4a1",
            CsWeapon::Tmp => "weapon_tmp",
            CsWeapon::G3sg1 => "weapon_g3sg1",
            CsWeapon::Flashbang => "weapon_flashbang",
            CsWeapon::Deagle => "weapon_deagle",
            CsWeapon::Sg552 => "weapon_sg552",
            CsWeapon::Ak47 => "weapon_ak47",
            CsWeapon::Knife => "weapon_knife",
            CsWeapon::P90 => "weapon_p90",
            CsWeapon::None => "",
        }
    }
}

impl std::fmt::Display for CsWeapon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.classname())
    }
}

/// Helper extension trait providing Counter-Strike specific operations on [`Player`].
pub trait CsPlayerExt {
    /// Returns whether the player has a defuse kit equipped.
    fn has_defuse_kit(&self) -> bool;
    /// Gives the player a specific CS weapon by enum.
    fn give_weapon(&self, weapon: CsWeapon) -> Option<i32>;
    /// Returns the player's team formatted as standard CS abbreviation (`"T"`, `"CT"`, `"SPEC"`).
    fn cs_team_str(&self) -> &'static str;
}

impl CsPlayerExt for Player {
    fn has_defuse_kit(&self) -> bool {
        false
    }

    fn give_weapon(&self, weapon: CsWeapon) -> Option<i32> {
        let cls = weapon.classname();
        if cls.is_empty() {
            None
        } else {
            self.give_item(cls)
        }
    }

    fn cs_team_str(&self) -> &'static str {
        match self.team() {
            Team::Terrorist => "TERRORIST",
            Team::CounterTerrorist => "CT",
            Team::Spectator => "SPECTATOR",
            Team::Unassigned => "UNASSIGNED",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cs_weapon_classnames() {
        assert_eq!(CsWeapon::M4a1.classname(), "weapon_m4a1");
        assert_eq!(CsWeapon::Ak47.classname(), "weapon_ak47");
        assert_eq!(CsWeapon::Deagle.classname(), "weapon_deagle");
    }
}
