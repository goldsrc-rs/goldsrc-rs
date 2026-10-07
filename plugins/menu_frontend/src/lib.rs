//! GoldSrc.rs Standard In-Game Menu Frontend (`goldsrc:menu_frontend`).
//!
//! Provides a universal, declarative menu orchestrator, multi-plugin section registry,
//! pagination, session tracking, debounce protection, and contract dispatch.

pub mod config;
pub mod error;
pub mod registry;
pub mod session;

use config::MenuFrontendConfig;
use error::MenuFrontendError;
use goldsrc::prelude::*;
use goldsrc_api::timer::host_time;
use registry::{MenuRegistry, RegisteredMenuSection};
use session::SessionManager;
use std::sync::RwLock;

static CONFIG: RwLock<Option<MenuFrontendConfig>> = RwLock::new(None);
static REGISTRY: RwLock<Option<MenuRegistry>> = RwLock::new(None);
static SESSIONS: RwLock<Option<SessionManager>> = RwLock::new(None);

pub struct MenuFrontend;

#[plugin(
    name = "menu_frontend",
    role = "ui",
    bundle = "infra",
    version = "0.19.0",
    author = "GoldSrc.rs Team",
    description = "Universal declarative menu renderer, session tracker, and cross-plugin registry",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl MenuFrontend {
    #[on_load]
    fn init() {
        log_info!("[Menu Frontend] Initializing universal menu orchestrator (v0.19.0)...");

        // 1. Initialize configuration
        if let Ok(mut lock) = CONFIG.write() {
            *lock = Some(MenuFrontendConfig::default());
        }

        // 2. Initialize section registry with standard plugins
        if let Ok(mut lock) = REGISTRY.write() {
            *lock = Some(MenuRegistry::new());
        }

        // 3. Initialize session manager
        if let Ok(mut lock) = SESSIONS.write() {
            *lock = Some(SessionManager::new());
        }

        log_info!("[Menu Frontend] Section registry and session orchestrator online.");
    }

    /// Renders the composite main menu for the target player.
    pub fn open_main_menu(player: &Player) {
        let title = if let Ok(lock) = CONFIG.read() {
            lock.as_ref()
                .map(|c| c.title.clone())
                .unwrap_or_else(|| "Главное Меню".to_string())
        } else {
            "Главное Меню".to_string()
        };

        let sections = if let Ok(lock) = REGISTRY.read() {
            if let Some(reg) = lock.as_ref() {
                reg.sections_for_player(player)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<RegisteredMenuSection>>()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let mut builder = Menu::builder(title).style(MenuStyle::brackets());

        if sections.is_empty() {
            builder = builder.item(MenuItem::new("Нет доступных разделов", 0));
        } else {
            for (idx, section) in sections.iter().enumerate() {
                let item_label = format!("{}. {}", idx + 1, section.title);
                builder = builder.item(MenuItem::new(item_label, section.id).keep_open());
            }
        }

        let menu = builder.build();
        player.open_menu(&menu);

        if let Ok(mut lock) = SESSIONS.write()
            && let Some(mgr) = lock.as_mut()
        {
            mgr.start_session(player.index(), 1, host_time());
        }
    }

    // --- Command Implementations ---

    /// Opens the unified server main menu.
    #[command(
        name = "grs_menu",
        aliases = ["menu", "/menu", "!menu", "mainmenu"],
        description = "Opens the unified server main menu with all accessible sections",
        usage = "grs_menu"
    )]
    fn cmd_menu(player: Player) {
        if !player.is_valid() {
            return;
        }
        Self::open_main_menu(&player);
    }

    /// Dynamically registers a custom sub-menu section.
    #[command(
        name = "grs_menu_add",
        capability = "admin:engine",
        description = "Registers a new section in the server main menu",
        usage = "grs_menu_add <id> <title> <command> [capability]"
    )]
    fn cmd_menu_add(id: u32, title: String, command: String, capability: Option<String>) {
        if let Ok(mut lock) = REGISTRY.write()
            && let Some(reg) = lock.as_mut()
        {
            reg.register(RegisteredMenuSection {
                id,
                title: title.clone(),
                command,
                capability,
            });
            log_info!(
                "[Menu Frontend] Registered new menu section #{} ('{}')",
                id,
                title
            );
        }
    }

    /// Dispatches messagemode text input prompt to client.
    #[command(
        name = "grs_menu_messagemode",
        description = "Prompts player for text input (STUB: missing host messagemode client command dispatch)",
        usage = "grs_menu_messagemode [prompt]"
    )]
    fn cmd_messagemode(player: Player, _prompt: Option<String>) {
        let err = MenuFrontendError::FeatureUnsupported {
            feature: "messagemode (Client Text Input Prompt)",
            missing_wit_binding: "host-client-command(player: s32, cmd: string)",
            reason: "WASM guest cannot send engine console command 'messagemode' to client to capture text responses",
        };
        log_err!("[Menu Frontend] {}", err);
        player.print_chat("[STUB Warning] Запрос текстового ввода через messagemode недоступен в текущей версии WIT.");
    }

    // --- Menu Action Handlers ---

    #[menu_action(id = 5001)]
    fn on_menu_admin(player: &mut Player) {
        player.print_notify("[Menu] Переход в панель управления сервером: grs_adminmenu");
    }

    #[menu_action(id = 5002)]
    fn on_menu_mod(player: &mut Player) {
        player.print_notify("[Menu] Переход в панель модерации: grs_modmenu");
    }

    #[menu_action(id = 5003)]
    fn on_menu_vip(player: &mut Player) {
        player.print_notify("[Menu] Переход в меню VIP привилегий: grs_privmenu");
    }

    #[menu_action(id = 5004)]
    fn on_menu_maps(player: &mut Player) {
        player.print_notify("[Menu] Переход в меню ротации карт: say /maps");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_frontend_config_derives_toml() {
        let cfg = MenuFrontendConfig::default();
        let toml_str = cfg.to_toml();
        assert!(toml_str.contains("page_size = 7"));
        assert!(toml_str.contains("debounce_ms = 150"));

        let cvars_str = cfg.to_cvars();
        assert!(cvars_str.contains("grs_menu_page_size"));
    }

    #[test]
    fn test_menu_registry_standard_sections() {
        let reg = MenuRegistry::new();
        assert!(reg.find_by_id(5001).is_some());
        assert!(reg.find_by_id(5002).is_some());
        assert!(reg.find_by_id(5003).is_some());
        assert!(reg.find_by_id(5004).is_some());
        assert!(reg.find_by_id(9999).is_none());
    }

    #[test]
    fn test_session_manager_debounce() {
        let mut mgr = SessionManager::new();
        mgr.start_session(1, 2, 10.0);
        // Pressing within 0.15s should be debounced
        assert!(!mgr.check_debounce(1, 10.05, 0.15));
        // Pressing after 0.20s should be allowed
        assert!(mgr.check_debounce(1, 10.25, 0.15));
    }
}
