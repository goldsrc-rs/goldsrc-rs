//! GoldSrc.rs Standard Privileges & VIP Suite (`goldsrc:privileges`).
//!
//! Provides player VIP privileges, equipment perks, round delivery limits,
//! passive regeneration, chat prefix customization, and reserved slot handling.

pub mod config;
pub mod error;
pub mod menu;
pub mod perks;

use config::PrivilegesConfig;
#[allow(unused_imports)]
use error::PrivilegesError;
use goldsrc::prelude::*;
use menu::*;
use perks::PerkService;
use std::sync::RwLock;

static CONFIG: RwLock<Option<PrivilegesConfig>> = RwLock::new(None);
static PERKS: RwLock<Option<PerkService>> = RwLock::new(None);

pub struct Privileges;

pub mod caps {
    pub const ACCESS: &str = "vip.access";
    pub const GIVE_ARMOR: &str = "vip.give_armor";
    pub const HEAL: &str = "vip.heal";
    pub const WEAPONS: &str = "vip.weapons";
    pub const PREFIX: &str = "vip.prefix";
    pub const SLOT: &str = "vip.slot";
}

#[plugin(
    name = "privileges",
    role = "feature",
    bundle = "gameplay",
    version = "0.19.0",
    author = "GoldSrc.rs Team",
    description = "Canonical GoldSrc VIP privileges, equipment perks, prefixes, and slot reservation",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl Privileges {
    #[on_load]
    fn init() {
        log_info!("[Privileges] Initializing canonical privileges suite (v0.19.0)...");

        // 1. Register capabilities
        Auth::register_capability(caps::ACCESS, "Grants access to general VIP features");
        Auth::register_capability(
            caps::GIVE_ARMOR,
            "Allows giving bonus armor to living VIP players",
        );
        Auth::register_capability(caps::HEAL, "Allows healing living VIP players to full HP");
        Auth::register_capability(
            caps::WEAPONS,
            "Allows receiving VIP round equipment and weapon perks",
        );
        Auth::register_capability(
            caps::PREFIX,
            "Displays [VIP] prefix in global chat messages",
        );
        Auth::register_capability(
            caps::SLOT,
            "Protects connection slot against server full kicks",
        );

        // 2. Initialize configuration
        if let Ok(mut lock) = CONFIG.write() {
            *lock = Some(PrivilegesConfig::default());
        }

        // 3. Initialize perk session service
        if let Ok(mut lock) = PERKS.write() {
            *lock = Some(PerkService::new());
        }

        // 4. Register chat middleware for [VIP] prefix
        goldsrc::chat::register_chat_middleware(|msg| {
            if msg.sender.has_capability(caps::ACCESS) || msg.sender.has_capability(caps::PREFIX) {
                msg.prefix = Some("[VIP] ".to_string());
            }
            true
        });

        log_info!("[Privileges] Registered 6 VIP capabilities and chat prefix middleware.");
    }

    /// Event handler for round start: resets per-round equipment locks.
    #[event("round_start")]
    fn on_round_start() {
        if let Ok(mut lock) = PERKS.write()
            && let Some(service) = lock.as_mut()
        {
            service.advance_round();
        }
    }

    /// Passive ECS system running during player post-think to regenerate health for living VIPs.
    #[system(stage = "post_think", phase = "modify")]
    fn vip_passive_regen(#[refined(Alive)] player: &mut Player) {
        if player.has_capability(caps::ACCESS) {
            let idx = player.index();
            let is_enabled = if let Ok(lock) = PERKS.read() {
                lock.as_ref()
                    .map(|s| s.is_regen_enabled(idx))
                    .unwrap_or(true)
            } else {
                true
            };

            if is_enabled {
                player.modify::<Health>(|hp| {
                    if hp.current() < 100.0 {
                        hp.heal(0.1);
                    }
                });
            }
        }
    }

    // --- Command Implementations ---

    /// Opens the interactive VIP privileges menu.
    #[command(
        name = "grs_privmenu",
        aliases = ["vipmenu", "/vip", "!vip", "privmenu", "/priv"],
        capability = "vip.access",
        description = "Opens the interactive VIP privileges and equipment menu",
        usage = "grs_privmenu"
    )]
    fn cmd_privmenu(player: Player) {
        if !player.is_valid() {
            return;
        }
        let round = if let Ok(lock) = PERKS.read() {
            lock.as_ref().map(|s| s.current_round()).unwrap_or(1)
        } else {
            1
        };
        let menu = build_vip_menu(round);
        player.open_menu(&menu);
    }

    /// Grants VIP capabilities to a player index.
    #[command(
        name = "vip_add",
        description = "Grants full VIP capabilities to a target player",
        usage = "vip_add <player_index>"
    )]
    fn cmd_vip_add(player: Player) {
        if !player.is_valid() {
            return;
        }
        player.grant_capability(caps::ACCESS);
        player.grant_capability(caps::GIVE_ARMOR);
        player.grant_capability(caps::HEAL);
        player.grant_capability(caps::WEAPONS);
        player.grant_capability(caps::PREFIX);
        player.grant_capability(caps::SLOT);

        player.play_sound("items/suitchargeno1.wav");
        player.print_center("[VIP] Вам успешно выданы привилегии VIP игрока!");
        log_info!(
            "[Privileges] Granted full VIP suite to player #{}",
            player.index()
        );
    }

    /// Checks VIP status for a target player.
    #[command(
        name = "vip_check",
        description = "Checks VIP capability status for a target player",
        usage = "vip_check <player_index>"
    )]
    fn cmd_vip_check(player: Player) {
        if !player.is_valid() {
            return;
        }
        let is_vip = player.has_capability(caps::ACCESS);
        let name = player
            .name()
            .unwrap_or_else(|| format!("Player #{}", player.index()));
        log_info!(
            "[Privileges] Player '{}' (#{}); VIP status: {}",
            name,
            player.index(),
            if is_vip { "ACTIVE" } else { "NONE" }
        );
    }

    /// Heals living VIP player to 100 HP.
    #[command(
        name = "vip_heal",
        capability = "vip.heal",
        description = "Restores target living player health to 100 HP",
        usage = "vip_heal <player_index>"
    )]
    fn cmd_vip_heal(mut player: Refined<'_, Player, Alive>) {
        player.set_health(100.0);
        player.print_center("[VIP] Здоровье полностью восстановлено (+100 HP)");
        log_info!("[Privileges] Restored 100 HP to player #{}", player.index());
    }

    /// Gives 100 armor to living VIP player.
    #[command(
        name = "vip_armor",
        capability = "vip.give_armor",
        description = "Restores target living player armor to 100 AP",
        usage = "vip_armor <player_index>"
    )]
    fn cmd_vip_armor(mut player: Refined<'_, Player, Alive>) {
        player.give_item("item_assaultsuit");
        player.set_armorvalue(100.0);
        player.print_center("[VIP] Броня полностью восстановлена (+100 AP)");
        log_info!("[Privileges] Given 100 armor to player #{}", player.index());
    }

    /// Reserved slots evaluation trigger: evicts non-privileged spectator if slots are full.
    #[command(
        name = "vip_slots_eval",
        capability = "vip.slot",
        description = "Evaluates server full slot protection and evicts non-VIP spectators if full",
        usage = "vip_slots_eval"
    )]
    fn cmd_slots_eval() {
        let max_players = cvar::cvar_get_float("maxplayers") as i32;
        let mut active_count = 0;
        let mut candidate: Option<Player> = None;

        for slot in 1..=32 {
            let p = Player::new(slot);
            if p.is_valid() {
                active_count += 1;
                if !p.has_capability(caps::ACCESS)
                    && !p.has_capability(caps::SLOT)
                    && (p.is_hltv() || candidate.is_none())
                {
                    candidate = Some(p);
                }
            }
        }

        if active_count >= max_players
            && let Some(target) = candidate
        {
            let name = target
                .name()
                .unwrap_or_else(|| format!("Player #{}", target.index()));
            let uid = target.user_id();
            log_info!(
                "[Privileges] Liberating reserved slot: disconnecting non-VIP '{}' (userid #{})",
                name,
                uid
            );
            server_command(format!(
                "kick #{} \"Слот зарезервирован для VIP игроков\"\n",
                uid
            ));
        } else {
            log_info!(
                "[Privileges] Slot check: {}/{} connected. Reservation intact.",
                active_count,
                max_players
            );
        }
    }

    // --- Menu Action Handlers ---

    #[menu_action(id = 4001)]
    fn on_menu_armor(player: &mut Player) {
        player.give_item("item_assaultsuit");
        player.set_armorvalue(100.0);
        player.play_sound("items/tr_kevlar.wav");
        player.print_center("[VIP] Получен комплект брони (+100 AP)");
    }

    #[menu_action(id = 4002)]
    fn on_menu_grenades(player: &mut Player) {
        player.give_item("weapon_hegrenade");
        player.give_item("weapon_flashbang");
        player.give_item("weapon_flashbang");
        player.give_item("weapon_smokegrenade");
        player.play_sound("items/gunpickup2.wav");
        player.print_center("[VIP] Получен комплект гранат");
    }

    #[menu_action(id = 4003)]
    fn on_menu_m4a1(player: &mut Player) {
        let idx = player.index();
        let already = if let Ok(lock) = PERKS.read() {
            lock.as_ref().map(|s| s.has_claimed(idx)).unwrap_or(false)
        } else {
            false
        };

        if already {
            player.print_center("[VIP Ошибка] Вы уже получали оружие в этом раунде!");
            return;
        }

        player.give_item("weapon_m4a1");
        player.give_item("weapon_deagle");
        player.play_sound("items/gunpickup2.wav");
        player.print_center("[VIP] Получен комплект M4A1 + Deagle");

        if let Ok(mut lock) = PERKS.write()
            && let Some(s) = lock.as_mut()
        {
            s.mark_claimed(idx);
        }
    }

    #[menu_action(id = 4004)]
    fn on_menu_ak47(player: &mut Player) {
        let idx = player.index();
        let already = if let Ok(lock) = PERKS.read() {
            lock.as_ref().map(|s| s.has_claimed(idx)).unwrap_or(false)
        } else {
            false
        };

        if already {
            player.print_center("[VIP Ошибка] Вы уже получали оружие в этом раунде!");
            return;
        }

        player.give_item("weapon_ak47");
        player.give_item("weapon_deagle");
        player.play_sound("items/gunpickup2.wav");
        player.print_center("[VIP] Получен комплект AK-47 + Deagle");

        if let Ok(mut lock) = PERKS.write()
            && let Some(s) = lock.as_mut()
        {
            s.mark_claimed(idx);
        }
    }

    #[menu_action(id = 4005)]
    fn on_menu_awp(player: &mut Player) {
        let (round, already) = if let Ok(lock) = PERKS.read() {
            lock.as_ref()
                .map(|s| (s.current_round(), s.has_claimed(player.index())))
                .unwrap_or((1, false))
        } else {
            (1, false)
        };

        if round < 3 {
            player.print_center(format!(
                "[VIP Ограничение] AWP доступна только с 3 раунда! (Сейчас: {round})"
            ));
            return;
        }

        if already {
            player.print_center("[VIP Ошибка] Вы уже получали оружие в этом раунде!");
            return;
        }

        player.give_item("weapon_awp");
        player.give_item("weapon_deagle");
        player.play_sound("items/gunpickup2.wav");
        player.print_center("[VIP] Получен снайперский комплект AWP + Deagle");

        if let Ok(mut lock) = PERKS.write()
            && let Some(s) = lock.as_mut()
        {
            s.mark_claimed(player.index());
        }
    }

    #[menu_action(id = 4006)]
    fn on_menu_deagle(player: &mut Player) {
        player.give_item("weapon_deagle");
        player.play_sound("items/gunpickup2.wav");
        player.print_center("[VIP] Выдан Desert Eagle");
    }

    #[menu_action(id = 4007)]
    fn on_menu_toggle_regen(player: &mut Player) {
        let idx = player.index();
        let new_state = if let Ok(mut lock) = PERKS.write() {
            lock.as_mut().map(|s| s.toggle_regen(idx)).unwrap_or(true)
        } else {
            true
        };

        player.print_center(format!(
            "[VIP] Авто-регенерация HP: {}",
            if new_state {
                "ВКЛЮЧЕНА"
            } else {
                "ОТКЛЮЧЕНА"
            }
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privileges_config_derives_toml() {
        let cfg = PrivilegesConfig::default();
        let toml_str = cfg.to_toml();
        assert!(toml_str.contains("bonus_hp = 100"));
        assert!(toml_str.contains("awp_round = 3"));

        let cvars_str = cfg.to_cvars();
        assert!(cvars_str.contains("grs_vip_bonus_hp"));
    }

    #[test]
    fn test_perk_service_rounds_and_claim_reset() {
        let mut service = PerkService::new();
        assert_eq!(service.current_round(), 1);
        assert!(!service.has_claimed(1));

        service.mark_claimed(1);
        assert!(service.has_claimed(1));

        service.advance_round();
        assert_eq!(service.current_round(), 2);
        assert!(!service.has_claimed(1));
    }

    #[test]
    fn test_perk_service_toggle_regen() {
        let mut service = PerkService::new();
        assert!(service.is_regen_enabled(1));
        assert!(!service.toggle_regen(1));
        assert!(!service.is_regen_enabled(1));
        assert!(service.toggle_regen(1));
        assert!(service.is_regen_enabled(1));
    }
}
