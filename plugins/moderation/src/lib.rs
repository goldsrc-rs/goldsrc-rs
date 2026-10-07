//! GoldSrc.rs Standard Disciplinary & Moderation Suite (`goldsrc:moderation`).
//!
//! Provides strict player discipline, sanctions (kick, ban, slap, slay, mute, gag, freeze),
//! inspection diagnostics, and interactive moderator menus.
//!
//! Designed strictly for generic GoldSrc engines (HLDS, CS, TFC, AG, DMC, Ricochet).

pub mod config;
pub mod error;
pub mod menu;
pub mod record;
pub mod repository;

use config::ModerationConfig;
#[allow(unused_imports)]
use error::ModerationError;
use goldsrc::prelude::*;
use goldsrc_api::timer::host_time;
use menu::*;
use record::{ActionType, ModerationRecord};
use repository::ModerationRepository;
use std::sync::RwLock;

static CONFIG: RwLock<Option<ModerationConfig>> = RwLock::new(None);
static REPO: RwLock<Option<ModerationRepository>> = RwLock::new(None);

pub struct Moderation;

pub mod caps {
    pub const KICK: &str = "moderation:action:kick";
    pub const BAN: &str = "moderation:action:ban";
    pub const SLAP: &str = "moderation:action:slap";
    pub const SLAY: &str = "moderation:action:slay";
    pub const MUTE_VOICE: &str = "moderation:action:mute:voice";
    pub const MUTE_CHAT: &str = "moderation:action:mute:chat";
    pub const FREEZE: &str = "moderation:action:freeze";
    pub const INSPECT: &str = "moderation:inspect";
}

#[plugin(
    name = "moderation",
    role = "coordinator",
    bundle = "staff",
    version = "0.19.0",
    author = "GoldSrc.rs Team",
    description = "Canonical GoldSrc disciplinary sanctions, player moderation, and inspection suite",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl Moderation {
    #[on_load]
    fn init() {
        log_info!("[Moderation] Initializing canonical moderation suite (v0.19.0)...");

        // 1. Register security capabilities with PBAC engine
        Auth::register_capability(caps::KICK, "Allows disconnecting players from the server");
        Auth::register_capability(caps::BAN, "Allows banning players from joining the server");
        Auth::register_capability(
            caps::SLAP,
            "Allows slapping players with physical displacement",
        );
        Auth::register_capability(caps::SLAY, "Allows instantly terminating living players");
        Auth::register_capability(
            caps::MUTE_VOICE,
            "Allows blocking player voice communications",
        );
        Auth::register_capability(
            caps::MUTE_CHAT,
            "Allows blocking player text chat communications",
        );
        Auth::register_capability(
            caps::FREEZE,
            "Allows temporarily freezing player movement for inspection",
        );
        Auth::register_capability(
            caps::INSPECT,
            "Allows viewing player network diagnostics and identity",
        );

        // 2. Initialize configuration model
        let cfg = ModerationConfig::default();
        if let Ok(mut lock) = CONFIG.write() {
            *lock = Some(cfg);
        }

        // 3. Initialize repository
        let repo = ModerationRepository::new();
        if let Ok(mut lock) = REPO.write() {
            *lock = Some(repo);
        }

        // 4. Register chat middleware for gag enforcement
        goldsrc::chat::register_chat_middleware(|msg| {
            let sender = msg.sender.index();
            let current_time = host_time();
            if let Ok(lock) = REPO.read()
                && let Some(repo) = lock.as_ref()
                && repo.is_chat_muted(sender, current_time)
            {
                msg.sender
                    .print_center("[САНКЦИЯ] Ваш текстовый чат заблокирован модератором!");
                msg.sender
                    .print_notify("[Moderation] Ваше сообщение заблокировано (активен gag).");
                return false;
            }
            true
        });

        log_info!("[Moderation] Registered 8 PBAC capabilities. Storage & Chat middleware online.");
    }

    #[on_frame]
    fn frame_tick() {
        let current_time = host_time();
        if let Ok(mut lock) = REPO.write()
            && let Some(repo) = lock.as_mut()
        {
            // Keep frozen players in place (set velocity to zero)
            for i in 1..=32 {
                if repo.is_player_frozen(i, current_time) {
                    let mut p = Player::new(i);
                    if p.is_valid() {
                        p.set_velocity(Vector3::new(0.0, 0.0, 0.0));
                    }
                }
            }
            repo.tick_cleanup(current_time);
        }
    }

    // --- Command Implementations ---

    /// Slaps a target player with displacement and configurable damage.
    #[command(
        name = "grs_slap",
        aliases = ["slap", "/slap"],
        capability = "moderation:action:slap",
        description = "Slaps a target player, inflicting damage and vertical impulse",
        usage = "grs_slap <#userid|name> [damage=0]"
    )]
    fn cmd_slap(target: Player, damage: Option<f32>) {
        if !target.is_valid() {
            log_warn!("[Moderation] Invalid slap target index: {}", target.index());
            return;
        }

        let dmg = damage.unwrap_or(0.0);
        let cur_hp = target.health().current;
        let new_hp = (cur_hp - dmg).max(1.0);

        let mut target_mut = target;
        target_mut.set_health(new_hp);

        let mut vel = target.velocity();
        vel.z += 250.0;
        vel.x += (target.index() % 5) as f32 * 30.0 - 60.0;
        target_mut.set_velocity(vel);

        target.play_sound("player/pl_pain2.wav");

        let name = target
            .name()
            .unwrap_or_else(|| format!("Player #{}", target.index()));
        log_info!(
            "[Moderation] Slapped '{}' (#{}), dealt {:.0} HP damage",
            name,
            target.index(),
            dmg
        );
    }

    /// Slay: instantly terminates a player entity.
    #[command(
        name = "grs_slay",
        aliases = ["slay", "/slay"],
        capability = "moderation:action:slay",
        description = "Instantly slays a target living player",
        usage = "grs_slay <#userid|name>"
    )]
    fn cmd_slay(mut target: Refined<'_, Player, Alive>) {
        target.set_health(0.0);
        target.play_sound("weapons/explode3.wav");

        // STUB: Spawning lightning sprite or explosion particles (TE_EXPLOSION / TE_LIGHTNING)
        // is currently unavailable because generic TempEntity network messages are not exposed in goldsrc.wit.
        let name = target
            .name()
            .unwrap_or_else(|| format!("Player #{}", target.index()));
        chat_broadcast!(&format!(
            "[Moderation] Игрок {name} был уничтожен администрацией"
        ));
        log_info!(
            "[Moderation] Slayed player '{}' (#{})",
            name,
            target.index()
        );
    }

    /// Freeze: immobilizes a player for cheat inspection or verification.
    #[command(
        name = "grs_freeze",
        aliases = ["freeze", "/freeze"],
        capability = "moderation:action:freeze",
        description = "Temporarily immobilizes a player for inspection",
        usage = "grs_freeze <#userid|name> [seconds=30]"
    )]
    fn cmd_freeze(mut target: Player, seconds: Option<i32>) {
        if !target.is_valid() {
            return;
        }

        let dur = seconds.unwrap_or(30).clamp(1, 300) as f32;
        let expire = host_time() + dur;

        if let Ok(mut lock) = REPO.write()
            && let Some(repo) = lock.as_mut()
        {
            if repo.is_player_frozen(target.index(), host_time()) {
                repo.unfreeze_player(target.index());
                target.print_chat("[Moderation] Вы были разморожены.");
                log_info!("[Moderation] Unfroze player #{}", target.index());
                return;
            } else {
                repo.set_player_freeze(target.index(), expire);
            }
        }

        // Zero out current velocity
        target.set_velocity(Vector3::new(0.0, 0.0, 0.0));
        target.print_center("[ПРОВЕРКА] Вы заморожены администратором! Оставайтесь на месте.");

        // STUB: Direct usercmd/input button masking (blocking IN_ATTACK, IN_JUMP at engine CmdStart level)
        // is not exposed in WIT. We enforce velocity nullification in on_frame instead.
        log_info!(
            "[Moderation] Froze player #{} for {:.0}s",
            target.index(),
            dur
        );
    }

    /// Gag: blocks a player from sending text chat and radio messages.
    #[command(
        name = "grs_gag",
        aliases = ["gag", "/gag"],
        capability = "moderation:action:mute:chat",
        description = "Blocks player text chat communications",
        usage = "grs_gag <#userid|name> [duration_mins=10] [reason]"
    )]
    fn cmd_gag(target: Player, duration_mins: Option<i32>, reason: Option<String>) {
        if !target.is_valid() {
            return;
        }

        let mins = duration_mins.unwrap_or(10).max(1);
        let why = reason.unwrap_or_else(|| "Нарушение правил общения".to_string());
        let expire = host_time() + (mins as f32 * 60.0);

        if let Ok(mut lock) = REPO.write()
            && let Some(repo) = lock.as_mut()
        {
            repo.set_chat_mute(target.index(), expire);
            let _ = repo.add_record(ModerationRecord {
                id: host_time() as u64,
                target_auth: target.auth_id(),
                target_ip: target.ip(),
                target_name: target.name().unwrap_or_default(),
                issuer_auth: "MODERATOR".to_string(),
                action_type: ActionType::Gag,
                reason: why.clone(),
                created_at: host_time() as u64,
                expires_at: (host_time() as u64) + (mins as u64 * 60),
                active: true,
            });
        }

        let name = target
            .name()
            .unwrap_or_else(|| format!("Player #{}", target.index()));
        chat_broadcast!(&format!(
            "[Moderation] Игроку {name} заблокирован чат на {mins} мин. Причина: {why}"
        ));
        target.print_center(format!("[GAG] Вам заблокирован чат на {mins} мин ({why})"));
    }

    /// Mute: blocks player voice transmission.
    #[command(
        name = "grs_mute",
        aliases = ["mute", "/mute"],
        capability = "moderation:action:mute:voice",
        description = "Blocks player voice transmission via engine SetClientListening",
        usage = "grs_mute <#userid|name> [duration_mins=10] [reason]"
    )]
    fn cmd_mute(target: Player, duration_mins: Option<i32>, reason: Option<String>) {
        if !target.is_valid() {
            return;
        }

        let mins = duration_mins.unwrap_or(10).max(1);
        let why = reason.unwrap_or_else(|| "Голосовой спам".to_string());
        let expire = host_time() + (mins as f32 * 60.0);

        // Enforce engine voice silence across all active player receivers
        for slot in 1..=32 {
            if slot != target.index() {
                let receiver = Player::new(slot);
                if receiver.is_valid() {
                    receiver.set_listening(&target, false);
                }
            }
        }

        if let Ok(mut lock) = REPO.write()
            && let Some(repo) = lock.as_mut()
        {
            repo.set_chat_mute(target.index(), expire);
            let _ = repo.add_record(ModerationRecord {
                id: host_time() as u64,
                target_auth: target.auth_id(),
                target_ip: target.ip(),
                target_name: target.name().unwrap_or_default(),
                issuer_auth: "MODERATOR".to_string(),
                action_type: ActionType::MuteVoice,
                reason: why.clone(),
                created_at: host_time() as u64,
                expires_at: (host_time() as u64) + (mins as u64 * 60),
                active: true,
            });
        }

        let name = target
            .name()
            .unwrap_or_else(|| format!("Player #{}", target.index()));
        chat_broadcast!(&format!(
            "[Moderation] Игроку {name} заблокирован микрофон на {mins} мин. Причина: {why}"
        ));
        target.print_center(format!(
            "[MUTE] Вам заблокирован микрофон на {mins} мин ({why})"
        ));
        log_info!(
            "[Moderation] Muted player '{}' (#{}) for {}m",
            name,
            target.index(),
            mins
        );
    }

    /// Kick: immediately disconnects a player from the server.
    #[command(
        name = "grs_kick",
        aliases = ["kick", "/kick"],
        capability = "moderation:action:kick",
        description = "Disconnects a player from the server via engine console command",
        usage = "grs_kick <#userid|name> [reason]"
    )]
    fn cmd_kick(target: Player, reason: Option<String>) {
        if !target.is_valid() {
            return;
        }

        let why = reason.unwrap_or_else(|| "Kicked by moderator".to_string());
        let name = target
            .name()
            .unwrap_or_else(|| format!("Player #{}", target.index()));
        let uid = target.user_id();

        chat_broadcast!(&format!(
            "[Moderation] Игрок {name} отключен модератором (Причина: {why})"
        ));
        log_info!(
            "[Moderation] Kicking player '{}' (userid #{}) with reason '{}'",
            name,
            uid,
            why
        );

        // Execute engine server command to kick client cleanly
        server_command(format!("kick #{}\n", uid));
    }

    /// Ban: bans player by SteamID/IP and disconnects client.
    #[command(
        name = "grs_ban",
        aliases = ["ban", "/ban"],
        capability = "moderation:action:ban",
        description = "Bans player by SteamID and IP and disconnects client",
        usage = "grs_ban <#userid|name> <duration_mins> [reason]"
    )]
    fn cmd_ban(target: Player, duration_mins: i32, reason: Option<String>) {
        if !target.is_valid() {
            return;
        }

        let mins = duration_mins.max(1);
        let why = reason.unwrap_or_else(|| "Banned by administrator".to_string());
        let name = target
            .name()
            .unwrap_or_else(|| format!("Player #{}", target.index()));
        let auth_id = target.auth_id();
        let ip = target.ip();
        let uid = target.user_id();

        if let Ok(mut lock) = REPO.write()
            && let Some(repo) = lock.as_mut()
        {
            let _ = repo.add_record(ModerationRecord {
                id: host_time() as u64,
                target_auth: auth_id.clone(),
                target_ip: ip.clone(),
                target_name: name.clone(),
                issuer_auth: "MODERATOR".to_string(),
                action_type: ActionType::Ban,
                reason: why.clone(),
                created_at: host_time() as u64,
                expires_at: (host_time() as u64) + (mins as u64 * 60),
                active: true,
            });
        }

        chat_broadcast!(&format!(
            "[Moderation] Игрок {name} забанен на {mins} мин. Причина: {why}"
        ));
        log_info!(
            "[Moderation] Banned '{}' (Auth: {}, IP: {}) for {}m (reason: {})",
            name,
            auth_id,
            ip,
            mins,
            why
        );

        // Disconnect banned client
        server_command(format!("kick #{}\n", uid));
    }

    /// Unban: removes an active ban by SteamID or IP string.
    #[command(
        name = "grs_unban",
        aliases = ["unban", "/unban"],
        capability = "moderation:action:ban",
        description = "Revokes an active ban by SteamID or IP address",
        usage = "grs_unban <auth_or_ip>"
    )]
    fn cmd_unban(auth_or_ip: String) {
        if let Ok(mut lock) = REPO.write()
            && let Some(repo) = lock.as_mut()
        {
            match repo.revoke_by_target(&auth_or_ip) {
                Ok(count) => {
                    log_info!(
                        "[Moderation] Revoked {} sanctions for '{}'",
                        count,
                        auth_or_ip
                    );
                }
                Err(e) => {
                    log_err!("[Moderation] Failed to unban '{}': {}", auth_or_ip, e);
                }
            }
        }
    }

    /// Inspect: displays connection and player diagnostics.
    #[command(
        name = "grs_inspect",
        aliases = ["inspect", "/inspect"],
        capability = "moderation:inspect",
        description = "Inspects target player diagnostics, coordinates, health, and team",
        usage = "grs_inspect <#userid|name>"
    )]
    fn cmd_inspect(target: Player) {
        if !target.is_valid() {
            log_warn!("[Moderation] Invalid inspect target");
            return;
        }

        let name = target.name().unwrap_or_default();
        let hp = target.health().current;
        let ap = target.armorvalue();
        let team = target.team();
        let origin = target.origin();
        let vel = target.velocity();
        let speed = (vel.x * vel.x + vel.y * vel.y).sqrt();

        log_info!(
            "================ [PLAYER INSPECTION: #{}] ================",
            target.index()
        );
        log_info!(
            "Name: '{}' | Team: {:?} | Lang: '{}'",
            name,
            team,
            target.lang()
        );
        log_info!(
            "Health: {:.0} HP | Armor: {:.0} AP | Speed: {:.1} units/s",
            hp,
            ap,
            speed
        );
        log_info!(
            "Origin: ({:.1}, {:.1}, {:.1}) | Velocity: ({:.1}, {:.1}, {:.1})",
            origin.x,
            origin.y,
            origin.z,
            vel.x,
            vel.y,
            vel.z
        );

        log_info!(
            "SteamID: '{}' | IP: '{}' | UserID: #{}",
            target.auth_id(),
            target.ip(),
            target.user_id()
        );
        log_info!("===========================================================");
    }

    /// Opens the interactive moderator control panel.
    #[command(
        name = "grs_modmenu",
        aliases = ["modmenu", "/modmenu"],
        capability = "moderation:inspect",
        description = "Opens the interactive moderator control panel menu",
        usage = "grs_modmenu"
    )]
    fn cmd_modmenu(player: Player) {
        if !player.is_valid() {
            return;
        }
        let menu = build_moderator_main_menu();
        player.open_menu(&menu);
    }

    // --- Interactive Menu Action Handlers ---

    #[menu_action(id = 1001)]
    fn on_menu_slap(player: &mut Player) {
        let menu = build_target_selection_menu("Slap (Толчок)");
        player.open_menu(&menu);
    }

    #[menu_action(id = 1002)]
    fn on_menu_slay(player: &mut Player) {
        let menu = build_target_selection_menu("Slay (Уничтожение)");
        player.open_menu(&menu);
    }

    #[menu_action(id = 1003)]
    fn on_menu_freeze(player: &mut Player) {
        let menu = build_target_selection_menu("Freeze (Заморозка)");
        player.open_menu(&menu);
    }

    #[menu_action(id = 1004)]
    fn on_menu_gag(player: &mut Player) {
        let menu = build_target_selection_menu("Gag (Блок чата)");
        player.open_menu(&menu);
    }

    #[menu_action(id = 1005)]
    fn on_menu_mute(player: &mut Player) {
        player.print_chat(
            "[Moderation STUB] Voice Mute недоступен: отсутствует SetClientListening в host WIT.",
        );
    }

    #[menu_action(id = 1006)]
    fn on_menu_kick(player: &mut Player) {
        player.print_chat(
            "[Moderation STUB] Kick недоступен: отсутствует host-disconnect-client в host WIT.",
        );
    }

    #[menu_action(id = 1007)]
    fn on_menu_ban(player: &mut Player) {
        player.print_chat(
            "[Moderation STUB] Ban недоступен: отсутствует получение SteamID/IP в host WIT.",
        );
    }

    #[menu_action(id = 1008)]
    fn on_menu_inspect(player: &mut Player) {
        let menu = build_target_selection_menu("Inspect (Инспекция)");
        player.open_menu(&menu);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moderation_config_derives_toml() {
        let cfg = ModerationConfig::default();
        let toml_str = cfg.to_toml();
        assert!(toml_str.contains("default_ban_mins = 60"));
        assert!(toml_str.contains("notify_chat = 1"));

        let cvars_str = cfg.to_cvars();
        assert!(cvars_str.contains("grs_mod_default_ban_mins \"60\""));
    }

    #[test]
    fn test_moderation_record_expiry() {
        let record = ModerationRecord {
            id: 1,
            target_auth: "STEAM_0:1:12345".to_string(),
            target_ip: "127.0.0.1".to_string(),
            target_name: "TestCheater".to_string(),
            issuer_auth: "MOD_ROOT".to_string(),
            action_type: ActionType::Gag,
            reason: "Spam".to_string(),
            created_at: 1000,
            expires_at: 2000,
            active: true,
        };

        assert!(record.is_currently_active(1500));
        assert!(!record.is_currently_active(2500));
    }

    #[test]
    fn test_repository_in_memory_freeze_and_gag() {
        let mut repo = ModerationRepository::default();
        repo.set_player_freeze(1, 100.0);
        assert!(repo.is_player_frozen(1, 50.0));
        assert!(!repo.is_player_frozen(1, 150.0));

        repo.set_chat_mute(2, 200.0);
        assert!(repo.is_chat_muted(2, 100.0));
        assert!(!repo.is_chat_muted(2, 250.0));
    }
}
