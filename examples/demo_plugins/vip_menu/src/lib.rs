use goldsrc::prelude::*;

pub struct VipMenu;

#[plugin(
    name = "vip_menu",
    version = "0.12.0",
    author = "GoldSrc.rs Team",
    description = "Interactive VIP multi-page equipment menu and weapon dispensary",
    url = "https://github.com/goldsrc-rs/goldsrc-rs",
    require = ["plugin:vip_core@>=0.10.0"]
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
        // Send a welcoming DHUD notice (HUD messages do not support menu \y/\w escape codes)
        let notice = HudMessage::builder("[VIP CLUB] Добро пожаловать в VIP Меню!")
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

    #[menu_action(id = 1)]
    fn on_select_m4a1(player: &mut Player) {
        player.give_item("weapon_m4a1");
        player.give_item("weapon_deagle");
        player.print_center("[VIP] Вы получили: M4A1 + Deagle");
        player.print_color("^4[VIP]^1 Вы взяли комплект: ^3M4A1 + Deagle");
    }

    #[menu_action(id = 2)]
    fn on_select_ak47(player: &mut Player) {
        player.give_item("weapon_ak47");
        player.give_item("weapon_deagle");
        player.print_center("[VIP] Вы получили: AK-47 + Deagle");
        player.print_color("^4[VIP]^1 Вы взяли комплект: ^3AK-47 + Deagle");
    }

    #[menu_action(id = 3)]
    fn on_select_awp(player: &mut Player) {
        player.give_item("weapon_awp");
        player.give_item("weapon_deagle");
        player.print_center("[VIP Gold] Вы получили: AWP + Deagle");
        player.print_color("^4[VIP Gold]^1 Вы взяли снайперский комплект: ^3AWP + Deagle");
    }

    #[menu_action(id = 4)]
    fn on_select_armor(player: &mut Player) {
        player.set_armorvalue(100.0);
        player.print_center("[VIP] Броня пополнена: 100 AP + Шлем");
        player.print_color("^4[VIP]^1 Вам выдана броня: ^3100 AP");
    }

    #[menu_action(id = 5)]
    fn on_select_medkit(player: &mut Player) {
        let current_hp = player.health();
        if current_hp > 0.0 {
            player.set_health((current_hp + 50.0).min(100.0));
            player.print_center("[VIP] Здоровье восстановлено (+50 HP)");
            player.print_color("^4[VIP]^1 Вы восстановили здоровье!");
        }
    }
}
