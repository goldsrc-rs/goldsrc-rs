use goldsrc::prelude::*;

pub struct VipMenu;

#[plugin(
    name = "vip_menu",
    version = "0.12.0",
    author = "GoldSrc.rs Team",
    description = "Interactive VIP multi-page equipment menu and weapon dispensary",
    url = "https://github.com/goldsrc-rs/goldsrc-rs",
    dependencies = ["vip_core@>=0.10.0"]
)]
impl VipMenu {
    #[on_load]
    fn init() {
        log_info!("[VIP Menu] Interactive VIP Menu Plugin v0.12.0 loaded successfully.");
    }

    /// Opens interactive VIP menu (e.g. `vipmenu 1` or chat `/vip`).
    #[command(
        name = "vipmenu",
        aliases = ["vip", "/vip", "!vip"],
        capability = "vip.access",
        description = "Opens interactive VIP equipment menu",
        usage = "vipmenu <player_index>"
    )]
    fn handle_menu(player: Alive<Player>) {
        // Send a welcoming DHUD notice
        let notice = HudMessage::builder("\\y[VIP CLUB]\\w Добро пожаловать в VIP Меню!")
            .dhud()
            .rgb(0, 255, 200)
            .position(-1.0, 0.2)
            .timing(0.1, 0.2, 3.0)
            .build();
        player.send_hud(&notice);

        // Build declarative interactive menu
        let menu = Menu::builder("\\yVIP Оружейка")
            .text("\\dВыберите комплект снаряжения:")
            .item(("M4A1 Carbine + Deagle", 1))
            .item(("AK-47 Kalashnikov + Deagle", 2))
            .item(
                MenuItem::new("AWP Sniper Rifle + Deagle", 3)
                    .require(Condition::Capability("vip.gold".into()))
                    .on_deny_replace("\\d[AWP Sniper - Нужен VIP Gold]"),
            )
            .spacer()
            .text("\\y--- Поддержка ---")
            .item(("Комплект брони (100 AP + Шлем)", 4))
            .item(("Экстренное лечение (+50 HP)", 5))
            .build();

        player.open_menu(&menu);

        let name = player
            .name()
            .unwrap_or_else(|| format!("Player #{}", player.index()));
        log_info!(
            "[VIP Menu] Opened interactive VIP menu for '{}' (idx={})",
            name,
            player.index()
        );
    }

    /// Handles menu slot selections dispatched by the engine.
    #[event(name = "menu_select")]
    fn on_menu_select(_name: String, payload: Vec<u8>) {
        if payload.len() < 4 {
            return;
        }
        let action_id = u32::from_le_bytes(payload[0..4].try_into().unwrap_or_default());
        log_info!("[VIP Menu] Player executed menu action #{action_id}");
    }
}
