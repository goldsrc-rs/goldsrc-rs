//! Counter-Strike 1.6 player extension traits, money, armor, and inventory operations.

use crate::weapons::CsWeapon;
use goldsrc_api::Player;
use goldsrc_api::client::Team;

/// Helper extension trait providing Counter-Strike specific operations on [`Player`].
pub trait CsPlayerExt {
    /// Returns whether the player has a defuse kit equipped.
    fn has_defuse_kit(&self) -> bool;

    /// Gives the player a specific CS weapon by enum.
    fn give_weapon(&self, weapon: CsWeapon) -> Option<i32>;

    /// Returns the player's team formatted as standard CS abbreviation (`"TERRORIST"`, `"CT"`, `"SPECTATOR"`).
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
