//! Privileges configuration model derived into TOML file format and CVAR registrations.

use goldsrc::{ConfigModel, CvarFlags};

#[derive(Debug, Clone, PartialEq, ConfigModel)]
#[config(cvar_prefix = "grs_vip_")]
pub struct PrivilegesConfig {
    /// Starting round HP provided to VIP players (e.g. 100)
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 0.0..=250.0)]
    pub bonus_hp: f32,

    /// Starting round armor provided to VIP players (e.g. 100)
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 0.0..=250.0)]
    pub bonus_armor: f32,

    /// Passive health regeneration per post-think tick (0.0 to disable)
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 0.0..=10.0)]
    pub regen_hp: f32,

    /// Minimum round number before auto equipment perks are granted
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 1..=20)]
    pub auto_equip_round: i32,

    /// Minimum round number before AWP sniper rifle selection is allowed
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 1..=20)]
    pub awp_round: i32,

    /// Enable [VIP] prefix in global and team chat messages
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER)]
    pub chat_prefix: bool,

    /// Number of player slots reserved for VIP players
    #[cvar(flags = CvarFlags::ARCHIVE | CvarFlags::SERVER, range = 0..=16)]
    pub reserved_slots: i32,
}

impl Default for PrivilegesConfig {
    fn default() -> Self {
        Self {
            bonus_hp: 100.0,
            bonus_armor: 100.0,
            regen_hp: 0.1,
            auto_equip_round: 2,
            awp_round: 3,
            chat_prefix: true,
            reserved_slots: 2,
        }
    }
}
