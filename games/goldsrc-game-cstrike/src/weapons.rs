//! Counter-Strike 1.6 weapon definitions, ammunition constants, and inventory slots.

/// Counter-Strike 1.6 weapon identifiers (matching CS 1.6 `CSW_*` constants).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    Galil = 140,
    Famas = 141,
    Ump45 = 142,
    Sg550 = 143,
}

/// Weapon slot inventory classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponSlot {
    Primary = 1,
    Secondary = 2,
    Knife = 3,
    Grenade = 4,
    C4 = 5,
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
            CsWeapon::Galil => "weapon_galil",
            CsWeapon::Famas => "weapon_famas",
            CsWeapon::Ump45 => "weapon_ump45",
            CsWeapon::Sg550 => "weapon_sg550",
            CsWeapon::None => "",
        }
    }

    /// Returns the weapon slot.
    pub const fn slot(&self) -> Option<WeaponSlot> {
        match self {
            CsWeapon::P228
            | CsWeapon::Glock
            | CsWeapon::Elite
            | CsWeapon::Fiveseven
            | CsWeapon::Usp
            | CsWeapon::Glock18
            | CsWeapon::Deagle => Some(WeaponSlot::Secondary),
            CsWeapon::Scout
            | CsWeapon::Xm1014
            | CsWeapon::Mac10
            | CsWeapon::Aug
            | CsWeapon::AwP
            | CsWeapon::Mp5Navy
            | CsWeapon::M249
            | CsWeapon::M3
            | CsWeapon::M4a1
            | CsWeapon::Tmp
            | CsWeapon::G3sg1
            | CsWeapon::Sg552
            | CsWeapon::Ak47
            | CsWeapon::P90
            | CsWeapon::Galil
            | CsWeapon::Famas
            | CsWeapon::Ump45
            | CsWeapon::Sg550 => Some(WeaponSlot::Primary),
            CsWeapon::Knife => Some(WeaponSlot::Knife),
            CsWeapon::HeGrenade | CsWeapon::Flashbang | CsWeapon::SmokeGrenade => {
                Some(WeaponSlot::Grenade)
            }
            CsWeapon::C4 => Some(WeaponSlot::C4),
            CsWeapon::None => None,
        }
    }
}

impl std::fmt::Display for CsWeapon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.classname())
    }
}
