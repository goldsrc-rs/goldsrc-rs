//! Counter-Strike 1.6 game domain logic, weapon constants, equipment, and ReGameDLL extensions.

pub mod player;
pub mod rules;
pub mod weapons;

pub use player::CsPlayerExt;
pub use rules::{RoundEndReason, RoundState};
pub use weapons::{CsWeapon, WeaponSlot};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cs_weapon_properties() {
        assert_eq!(CsWeapon::M4a1.classname(), "weapon_m4a1");
        assert_eq!(CsWeapon::M4a1.slot(), Some(WeaponSlot::Primary));
        assert_eq!(CsWeapon::Deagle.slot(), Some(WeaponSlot::Secondary));
        assert_eq!(CsWeapon::Knife.slot(), Some(WeaponSlot::Knife));
        assert_eq!(CsWeapon::HeGrenade.slot(), Some(WeaponSlot::Grenade));
        assert_eq!(CsWeapon::C4.slot(), Some(WeaponSlot::C4));
    }
}
