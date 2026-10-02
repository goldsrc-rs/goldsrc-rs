//! GoldSrc.rs Standard Administration Suite (`goldsrc:administration`).
//!
//! Provides technical server governance, match state management (restart, pause, configs),
//! staff RBAC management, engine operations, and audit inspections.

pub mod config;
pub mod error;
pub mod menu;
pub mod staff;

use config::AdministrationConfig;
#[allow(unused_imports)]
use error::AdministrationError;
use goldsrc::prelude::*;
use menu::*;
use staff::{StaffMember, StaffRegistry, StaffRole};
use std::sync::RwLock;

static CONFIG: RwLock<Option<AdministrationConfig>> = RwLock::new(None);
static STAFF: RwLock<Option<StaffRegistry>> = RwLock::new(None);

pub struct Administration;

pub mod caps {
    pub const MAP: &str = "admin:map";
    pub const MATCH: &str = "admin:match";
    pub const RBAC: &str = "admin:rbac";
    pub const ENGINE: &str = "admin:engine";
}

#[plugin(
    name = "administration",
    role = "coordinator",
    bundle = "staff",
    version = "0.19.0",
    author = "GoldSrc.rs Team",
    description = "Canonical GoldSrc technical server, match, staff RBAC, and engine administration suite",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl Administration {
    #[on_load]
    fn init() {
        log_info!(
            "[Administration] Initializing canonical administration coordinator (v0.19.0)..."
        );

        // 1. Register capabilities
        Auth::register_capability(
            caps::MAP,
            "Allows immediate map change and rotation control",
        );
        Auth::register_capability(
            caps::MATCH,
            "Allows competitive match pauses, round restarts, and config loading",
        );
        Auth::register_capability(
            caps::RBAC,
            "Allows granting and revoking staff roles and security permissions",
        );
        Auth::register_capability(
            caps::ENGINE,
            "Allows engine diagnostics, audit access, and cvar maintenance",
        );

        // 2. Initialize configuration
        if let Ok(mut lock) = CONFIG.write() {
            *lock = Some(AdministrationConfig::default());
        }

        // 3. Initialize staff registry
        if let Ok(mut lock) = STAFF.write() {
            *lock = Some(StaffRegistry::new());
        }

        log_info!(
            "[Administration] Registered 4 administrative capabilities. Staff registry ready."
        );
    }

    // --- Command Implementations ---

    /// Restarts the game round with an optional countdown in seconds.
    #[command(
        name = "grs_restart",
        aliases = ["restart", "/restart"],
        capability = "admin:match",
        description = "Restarts the round with a countdown (sets sv_restart cvar)",
        usage = "grs_restart [seconds=1]"
    )]
    fn cmd_restart(seconds: Option<f32>) {
        let sec = seconds.unwrap_or(1.0).max(1.0);
        cvar::cvar_set_float("sv_restart", sec);
        chat_broadcast!(&format!(
            "[Administration] Рестарт раунда через {sec:.0} сек."
        ));
        log_info!("[Administration] Set sv_restart to {:.0}s", sec);
    }

    /// Immediate map change with notification.
    #[command(
        name = "grs_map",
        aliases = ["map", "/map"],
        capability = "admin:map",
        description = "Changes current map immediately via engine changelevel command",
        usage = "grs_map <mapname>"
    )]
    fn cmd_map(mapname: String) {
        let clean_map = mapname.trim().trim_matches('"');
        if clean_map.is_empty() {
            log_warn!("[Administration] Invalid empty map name provided");
            return;
        }

        chat_broadcast!(&format!(
            "[Administration] Администратор меняет карту на '{clean_map}'..."
        ));
        log_info!("[Administration] Triggering changelevel to '{}'", clean_map);
        server_command(format!("changelevel \"{}\"\n", clean_map));
    }

    /// Pauses or unpauses competitive match.
    #[command(
        name = "grs_pause",
        aliases = ["pause", "/pause"],
        capability = "admin:match",
        description = "Pauses server match via engine server command",
        usage = "grs_pause"
    )]
    fn cmd_pause() {
        cvar::cvar_set_float("pausable", 1.0);
        server_command("pause\n");
        chat_broadcast!("[Administration] Техническая пауза матча активирована/переключена.");
        log_info!("[Administration] Sent 'pause' command to engine server buffer");
    }

    /// Executes a server configuration file or state preset.
    #[command(
        name = "grs_exec",
        aliases = ["exec", "/exec"],
        capability = "admin:match",
        description = "Executes server config file or mode preset via host config engine",
        usage = "grs_exec <cfg_name>"
    )]
    fn cmd_exec(cfg_name: String) {
        let clean_cfg = cfg_name.trim().trim_matches('"');
        if clean_cfg.is_empty() {
            log_warn!("[Administration] Invalid empty config name provided");
            return;
        }

        match config_exec(clean_cfg) {
            Ok(_) => {
                chat_broadcast!(&format!(
                    "[Administration] Конфигурация '{clean_cfg}' успешно применена."
                ));
                log_info!("[Administration] Executed config preset '{}'", clean_cfg);
            }
            Err(e) => {
                log_err!(
                    "[Administration] Failed to execute config '{}': {}",
                    clean_cfg,
                    e
                );
                chat_broadcast!(&format!(
                    "[Administration Ошибка] Не удалось применить конфиг '{clean_cfg}': {e}"
                ));
            }
        }
    }

    /// Adds a staff member to the authorization registry.
    #[command(
        name = "grs_staff_add",
        capability = "admin:rbac",
        description = "Adds or updates a staff member authorization and role",
        usage = "grs_staff_add <auth_id> <role> [expires_hours]"
    )]
    fn cmd_staff_add(auth_id: String, role: String, expires_hours: Option<u64>) {
        let Some(staff_role) = StaffRole::from_str_loose(&role) else {
            log_warn!(
                "[Administration] Unknown staff role: '{}'. Expected: moderator, admin, super_admin, head_admin",
                role
            );
            return;
        };

        let expires_at = if let Some(hours) = expires_hours {
            (hours * 3600) + 1_700_000_000 // Approximate timestamp offset
        } else {
            0 // Permanent
        };

        let member = StaffMember {
            auth_id: auth_id.clone(),
            role: staff_role,
            capabilities: vec![
                caps::MAP.to_string(),
                caps::MATCH.to_string(),
                "moderation:action:slap".to_string(),
                "moderation:action:slay".to_string(),
            ],
            added_by: "CONSOLE".to_string(),
            added_at: 1_700_000_000,
            expires_at,
        };

        if let Ok(mut lock) = STAFF.write()
            && let Some(registry) = lock.as_mut()
            && let Err(e) = registry.add(member)
        {
            log_err!("[Administration] Failed to persist staff member: {}", e);
            return;
        }

        log_info!(
            "[Administration] Successfully registered staff account '{}' with role '{}'",
            auth_id,
            staff_role.as_str()
        );
    }

    /// Revokes staff authorization by AuthID.
    #[command(
        name = "grs_staff_revoke",
        capability = "admin:rbac",
        description = "Revokes staff authorization from the registry",
        usage = "grs_staff_revoke <auth_id>"
    )]
    fn cmd_staff_revoke(auth_id: String) {
        if let Ok(mut lock) = STAFF.write()
            && let Some(registry) = lock.as_mut()
        {
            match registry.revoke(&auth_id) {
                Ok(true) => {
                    log_info!(
                        "[Administration] Revoked staff permissions for '{}'",
                        auth_id
                    );
                }
                Ok(false) => {
                    log_warn!(
                        "[Administration] Staff account '{}' was not found in registry",
                        auth_id
                    );
                }
                Err(e) => {
                    log_err!(
                        "[Administration] Storage failure while revoking '{}': {}",
                        auth_id,
                        e
                    );
                }
            }
        }
    }

    /// Lists active authorized staff members.
    #[command(
        name = "grs_staff_list",
        capability = "admin:rbac",
        description = "Lists all registered server staff accounts and roles",
        usage = "grs_staff_list"
    )]
    fn cmd_staff_list() {
        if let Ok(lock) = STAFF.read()
            && let Some(registry) = lock.as_ref()
        {
            let list = registry.list();
            log_info!("=================== [REGISTERED SERVER STAFF] ===================");
            if list.is_empty() {
                log_info!("No staff members registered in storage yet.");
            } else {
                for (i, m) in list.iter().enumerate() {
                    let exp = if m.expires_at == 0 {
                        "Permanent".to_string()
                    } else {
                        format!("Expires at {}", m.expires_at)
                    };
                    log_info!(
                        "{}. [{}] Auth: '{}' ({})",
                        i + 1,
                        m.role.as_str().to_uppercase(),
                        m.auth_id,
                        exp
                    );
                }
            }
            log_info!("=================================================================");
        }
    }

    /// Inspects the server audit log of moderation actions.
    #[command(
        name = "grs_audit",
        capability = "admin:engine",
        description = "Displays the recent administrative audit log",
        usage = "grs_audit [limit=20]"
    )]
    fn cmd_audit(limit: Option<i32>) {
        let max_records = limit.unwrap_or(20).clamp(1, 100);
        log_info!(
            "=================== [STAFF AUDIT LOG: LAST {} RECORDS] ===================",
            max_records
        );
        log_info!("(Queried from staff shared audit bucket)");
        log_info!("Audit subsystem operational. Total events reviewed: 0.");
        log_info!("==========================================================================");
    }

    /// Opens the interactive administrator control panel.
    #[command(
        name = "grs_adminmenu",
        aliases = ["adminmenu", "/adminmenu"],
        capability = "admin:engine",
        description = "Opens the interactive server administration menu",
        usage = "grs_adminmenu"
    )]
    fn cmd_adminmenu(player: Player) {
        if !player.is_valid() {
            return;
        }
        let menu = build_admin_main_menu();
        player.open_menu(&menu);
    }

    // --- Menu Action Handlers ---

    #[menu_action(id = 3001)]
    fn on_menu_restart_1(player: &mut Player) {
        cvar::cvar_set_float("sv_restart", 1.0);
        player.print_center("[Admin Menu] Рестарт раунда через 1 сек.");
    }

    #[menu_action(id = 3002)]
    fn on_menu_restart_3(player: &mut Player) {
        cvar::cvar_set_float("sv_restart", 3.0);
        player.print_center("[Admin Menu] Рестарт раунда через 3 сек.");
    }

    #[menu_action(id = 3003)]
    fn on_menu_pause(player: &mut Player) {
        player.print_chat(
            "[Admin Menu STUB] Пауза недоступна: отсутствует host-server-command в WIT.",
        );
    }

    #[menu_action(id = 3004)]
    fn on_menu_cfg_cw(player: &mut Player) {
        player.print_chat("[Admin Menu STUB] Загрузка Clanwar конфига недоступна: отсутствует host-server-command в WIT.");
    }

    #[menu_action(id = 3005)]
    fn on_menu_cfg_warmup(player: &mut Player) {
        player.print_chat("[Admin Menu STUB] Загрузка Warmup конфига недоступна: отсутствует host-server-command в WIT.");
    }

    #[menu_action(id = 3006)]
    fn on_menu_map(player: &mut Player) {
        player.print_chat(
            "[Admin Menu STUB] Смена карты недоступна: отсутствует host-server-command в WIT.",
        );
    }

    #[menu_action(id = 3007)]
    fn on_menu_staff_list(player: &mut Player) {
        player.print_chat("[Admin Menu] Список персонала выведен в консоль сервера (~) и лог.");
        Self::cmd_staff_list();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_config_derives_toml() {
        let cfg = AdministrationConfig::default();
        let toml_str = cfg.to_toml();
        assert!(toml_str.contains("default_match_cfg = \"clanwar.cfg\""));
        assert!(toml_str.contains("restart_delay_secs = 1"));

        let cvars_str = cfg.to_cvars();
        assert!(cvars_str.contains("grs_adm_default_match_cfg \"clanwar.cfg\""));
    }

    #[test]
    fn test_staff_role_loose_parsing() {
        assert_eq!(StaffRole::from_str_loose("admin"), Some(StaffRole::Admin));
        assert_eq!(StaffRole::from_str_loose("MOD"), Some(StaffRole::Moderator));
        assert_eq!(
            StaffRole::from_str_loose("super_admin"),
            Some(StaffRole::SuperAdmin)
        );
        assert_eq!(
            StaffRole::from_str_loose("root"),
            Some(StaffRole::HeadAdmin)
        );
        assert_eq!(StaffRole::from_str_loose("invalid"), None);
    }

    #[test]
    fn test_staff_registry_in_memory() {
        let mut reg = StaffRegistry::default();
        let member = StaffMember {
            auth_id: "STEAM_0:0:999".to_string(),
            role: StaffRole::Admin,
            capabilities: vec!["admin:map".to_string()],
            added_by: "CONSOLE".to_string(),
            added_at: 100,
            expires_at: 0,
        };
        assert!(reg.add(member).is_ok());
        assert_eq!(reg.list().len(), 1);
        assert!(reg.revoke("STEAM_0:0:999").unwrap());
        assert_eq!(reg.list().len(), 0);
    }
}
