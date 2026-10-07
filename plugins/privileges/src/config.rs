//! Privileges configuration model derived into TOML file format and CVAR registrations.

use goldsrc::ConfigModel;

#[derive(Debug, Clone, PartialEq, ConfigModel)]
pub struct PrivilegesConfig {
    #[cvar(
        name = "grs_vip_bonus_hp",
        flags = "ARCHIVE|SERVER",
        description = "Starting round HP provided to VIP players (e.g. 100)"
    )]
    pub bonus_hp: f32,

    #[cvar(
        name = "grs_vip_bonus_armor",
        flags = "ARCHIVE|SERVER",
        description = "Starting round armor provided to VIP players (e.g. 100)"
    )]
    pub bonus_armor: f32,

    #[cvar(
        name = "grs_vip_regen_hp",
        flags = "ARCHIVE|SERVER",
        description = "Passive health regeneration per post-think tick (0.0 to disable)"
    )]
    pub regen_hp: f32,

    #[cvar(
        name = "grs_vip_auto_equip_round",
        flags = "ARCHIVE|SERVER",
        description = "Minimum round number before auto equipment perks are granted"
    )]
    pub auto_equip_round: i32,

    #[cvar(
        name = "grs_vip_awp_round",
        flags = "ARCHIVE|SERVER",
        description = "Minimum round number before AWP sniper rifle selection is allowed"
    )]
    pub awp_round: i32,

    #[cvar(
        name = "grs_vip_chat_prefix",
        flags = "ARCHIVE|SERVER",
        description = "Enable [VIP] prefix in global and team chat messages (1 = yes, 0 = no)"
    )]
    pub chat_prefix: i32,

    #[cvar(
        name = "grs_vip_reserved_slots",
        flags = "ARCHIVE|SERVER",
        description = "Number of player slots reserved for VIP players"
    )]
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
            chat_prefix: 1,
            reserved_slots: 2,
        }
    }
}
