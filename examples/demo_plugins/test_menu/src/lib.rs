use goldsrc::prelude::*;

pub struct TestMenu;

#[plugin(
    name = "test_menu",
    version = "0.13.0",
    author = "GoldSrc.rs Team",
    description = "Declarative multi-page and DHUD menu verification suite",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl TestMenu {
    #[on_load]
    fn init() {
        log_info!("[Test Menu] Initialized declarative menu verification plugin (v0.13.0).");
    }

    /// Tests interactive multi-page menu rendered via ShowMenu.
    #[command(
        name = "test_menu",
        description = "Opens a test multi-page menu",
        usage = "test_menu [player_index]"
    )]
    fn handle_test_menu(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let player = Player::new(idx);
        if !player.is_valid() {
            log_warn!("[Test Menu] Player {} is not connected/valid!", idx);
            return;
        }

        let menu = Menu::builder("Главное Тестовое Меню")
            .style(MenuStyle::brackets())
            .item(MenuItem::new("Пополнить здоровье (+100 HP)", 101).keep_open())
            .item(MenuItem::new("Пополнить броню (+100 AP)", 102).keep_open())
            .item(("Выдать AWP", 103))
            .item(("Выдать Deagle", 104))
            .page(|page| {
                page.divider("--- Оружие и Снаряжение ---")
                    .item(("Выдать M4A1", 105))
                    .item(("Выдать AK47", 106))
            })
            .build();

        player.open_menu(&menu);
        log_info!("[Test Menu] Opened test menu for player #{}", idx);
    }

    /// Tests menu rendered via Director HUD (DHUD).
    #[command(
        name = "test_dhud_menu",
        description = "Opens a test menu rendered via DHUD",
        usage = "test_dhud_menu [player_index]"
    )]
    fn handle_test_dhud_menu(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let player = Player::new(idx);
        if !player.is_valid() {
            log_warn!("[Test Menu] Player {} is not valid!", idx);
            return;
        }

        let menu = Menu::builder("DHUD Рендер Меню")
            .style(MenuStyle::brackets())
            .renderer(MenuRendererKind::Dhud {
                position: HudCoord::new(0.05, 0.25),
                color: HudColor::new(0, 255, 255, 255),
                effect: HudEffect::FadeInOut {
                    fade_in: 0.1,
                    fade_out: 0.1,
                    hold_time: 5.0,
                },
            })
            .item(MenuItem::new("Пополнить здоровье (+100 HP)", 101).keep_open())
            .item(MenuItem::new("Пополнить броню (+100 AP)", 102).keep_open())
            .item(("Телепорт вверх (+100 Z)", 107))
            .build();

        player.open_menu(&menu);
        log_info!("[Test Menu] Opened DHUD-rendered menu for player #{}", idx);
    }

    #[menu_action(id = 101)]
    fn on_menu_heal(player: &mut Player) {
        let cur = player.health();
        player.set_health(cur + 100.0);
        player.print_center("[Test Menu] Здоровье пополнено (+100 HP)");
    }

    #[menu_action(id = 102)]
    fn on_menu_armor(player: &mut Player) {
        let cur = player.armorvalue();
        player.set_armorvalue(cur + 100.0);
        player.print_center("[Test Menu] Броня пополнена (+100 AP)");
    }

    #[menu_action(id = 103)]
    fn on_menu_give_awp(player: &mut Player) {
        player.give_item("weapon_awp");
        player.print_center("[Test Menu] Выдана AWP");
    }

    #[menu_action(id = 104)]
    fn on_menu_give_deagle(player: &mut Player) {
        player.give_item("weapon_deagle");
        player.print_center("[Test Menu] Выдан Deagle");
    }

    #[menu_action(id = 105)]
    fn on_menu_give_m4a1(player: &mut Player) {
        player.give_item("weapon_m4a1");
        player.print_center("[Test Menu] Выдана M4A1");
    }

    #[menu_action(id = 106)]
    fn on_menu_give_ak47(player: &mut Player) {
        player.give_item("weapon_ak47");
        player.print_center("[Test Menu] Выдан AK47");
    }

    #[menu_action(id = 107)]
    fn on_menu_teleport(player: &mut Player) {
        let mut pos = player.origin();
        pos.z += 100.0;
        player.set_origin(pos);
        player.print_center("[Test Menu] Телепортирован вверх");
    }
}
