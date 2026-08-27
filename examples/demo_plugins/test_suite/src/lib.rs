use goldsrc::prelude::*;

#[derive(Debug, PartialEq)]
struct StatsComponent {
    kills: u32,
    deaths: u32,
}

pub struct TestSuite;

#[plugin(
    name = "test_suite",
    version = "0.10.0",
    author = "GoldSrc.rs Team",
    description = "ECS, entity inspection, sound playback, and CVar test suite",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl TestSuite {
    #[on_load]
    fn init() {
        log_info!("[Test Suite] Initializing GoldSrc.rs test suite (v0.10.0)...");

        // 1. ECS World Verification
        let mut world = World::new();
        let p1 = EntityId(1);
        let p2 = EntityId(2);

        world.insert(
            p1,
            StatsComponent {
                kills: 10,
                deaths: 2,
            },
        );
        world.insert(
            p2,
            StatsComponent {
                kills: 5,
                deaths: 8,
            },
        );

        if let Some(stats) = world.get::<StatsComponent>(p1) {
            log_info!(
                "[Test Suite] ECS Player 1 Stats: Kills={}, Deaths={}",
                stats.kills,
                stats.deaths
            );
        }

        log_info!("[Test Suite] ECS initialized successfully.");
    }

    #[command(
        name = "testcmd",
        description = "Test echo command for logging arguments",
        usage = "testcmd [args...]"
    )]
    fn handle_testcmd(cmd: String, args: String) {
        log_info!(
            "[Test Suite] Command Handler '{}' executed with args: '{}'",
            cmd,
            args
        );
    }

    /// Tests player inspection: origin, angles, health, armor.
    #[command(
        name = "test_player",
        description = "Inspects player origin, angles, health, and armor",
        usage = "test_player [player_index]"
    )]
    fn handle_test_player(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let player = Player::new(idx);
        if !player.is_valid() {
            log_warn!("[Test Suite] Player {} is not connected/valid!", idx);
            return;
        }

        let name = player.name().unwrap_or_else(|| "Unknown".to_string());
        let pos = player.origin();
        let ang = player.angles();
        let hp = player.health();
        let armor = player.armorvalue();

        log_info!(
            "[Test Suite] Player #{}: '{}' | HP: {} | Armor: {} | Pos: ({:.1}, {:.1}, {:.1}) | Angles: ({:.1}, {:.1}, {:.1})",
            idx,
            name,
            hp,
            armor,
            pos.x,
            pos.y,
            pos.z,
            ang.x,
            ang.y,
            ang.z
        );
    }

    /// Tests setting player health, armor, and teleporting.
    #[command(
        name = "test_buff",
        description = "Buffs target player health (250 HP) and armor (100 AP)",
        usage = "test_buff [player_index]"
    )]
    fn handle_test_buff(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let mut player = Player::new(idx);
        if !player.is_valid() {
            log_warn!("[Test Suite] Player {} is not valid!", idx);
            return;
        }

        player.set_health(250.0);
        player.set_armorvalue(100.0);
        log_info!("[Test Suite] Buffed player #{}: HP=250, Armor=100", idx);
    }

    /// Tests CVar reading and writing.
    #[command(
        name = "test_cvar",
        description = "Reads or updates a server console variable (CVar)",
        usage = "test_cvar <cvar_name> [new_value]"
    )]
    fn handle_test_cvar(_cmd: String, args: String) {
        let mut parts = args.split_whitespace();
        let cvar_name = parts.next().unwrap_or("sv_gravity");
        let new_val = parts.next();

        let old_val = engine::cvar_get_float(cvar_name);
        log_info!(
            "[Test Suite] CVar '{}' current value: {:.1}",
            cvar_name,
            old_val
        );

        if let Some(val_str) = new_val
            && let Ok(v) = val_str.parse::<f32>()
        {
            engine::cvar_set_float(cvar_name, v);
            log_info!("[Test Suite] CVar '{}' updated to: {:.1}", cvar_name, v);
        }
    }

    /// Tests nested and paginated menus with page breaks, ExitBehavior, and actions.
    #[command(
        name = "test_menu",
        description = "Opens interactive multi-page and nested test menu",
        usage = "test_menu [player_index]"
    )]
    fn handle_test_menu(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let player = Player::new(idx);
        if !player.is_valid() {
            log_warn!("[Test Suite] Player {} is not valid!", idx);
            return;
        }

        // Build a multi-page test menu with explicit page builder
        let menu = Menu::builder("Test Suite: Multi-Page Menu")
            .style(MenuStyle::brackets())
            .exit_behavior(ExitBehavior::PopParent)
            .page(|p| {
                p.text("\\y[Страница 1: Базовые действия]")
                    .item(MenuItem::new("Пополнить здоровье (+100 HP)", 101).keep_open())
                    .item(MenuItem::new("Пополнить броню (+100 AP)", 102).keep_open())
                    .item(("Открыть подменю оружия ->", 103))
            })
            .page(|p| {
                p.text("\\y[Страница 2: Дополнительные тесты]")
                    .item(("Тестовый звук", 104))
                    .item(("Телепорт на +100 Z юнитов", 105))
            })
            .build();

        player.open_menu(&menu);
        log_info!(
            "[Test Suite] Opened multi-page test menu for player #{}",
            idx
        );
    }

    #[menu_action(id = 101)]
    fn on_menu_heal(player: &mut Player) {
        player.set_health(100.0);
        player.print_center("[Test Suite] Здоровье восстановлено (100 HP)");
        player.print_color("^4[Test Suite]^1 Здоровье установлено на ^3100 HP");
    }

    #[menu_action(id = 102)]
    fn on_menu_armor(player: &mut Player) {
        player.set_armorvalue(100.0);
        player.print_center("[Test Suite] Броня выдана (100 AP)");
        player.print_color("^4[Test Suite]^1 Броня установлена на ^3100 AP");
    }

    #[menu_action(id = 103)]
    fn on_open_nested_weapons(player: &mut Player) {
        let submenu = Menu::builder("Оружейное подменю")
            .style(MenuStyle::classic())
            .exit_behavior(ExitBehavior::PopParent)
            .item(("M4A1 Carbine", 106))
            .item(("AK-47 Kalashnikov", 107))
            .item(("AWP Magnum", 108))
            .item(("Desert Eagle", 109))
            .build();

        player.open_menu(&submenu);
        log_info!(
            "[Test Suite] Opened nested submenu for player #{}",
            player.index()
        );
    }

    #[menu_action(id = 104)]
    fn on_menu_sound(player: &mut Player) {
        engine::emit_sound(player.index(), 0, "events/tutor_msg.wav", 1.0, 0.8, 0, 100);
        player.print_center("[Test Suite] Проигран тестовый звук");
    }

    #[menu_action(id = 105)]
    fn on_menu_teleport(player: &mut Player) {
        let mut origin = player.origin();
        origin.z += 100.0; // Boost player upwards
        player.set_origin(origin);
        player.print_center("[Test Suite] Телепорт выполнен (+100 Z)");
        player.print_color("^4[Test Suite]^1 Вы телепортированы на ^3+100 Z^1 единиц!");
    }

    #[menu_action(id = 106)]
    fn on_menu_give_m4(player: &mut Player) {
        player.give_item("weapon_m4a1");
        player.print_center("[Test Suite] Выдана M4A1");
    }

    #[menu_action(id = 107)]
    fn on_menu_give_ak(player: &mut Player) {
        player.give_item("weapon_ak47");
        player.print_center("[Test Suite] Выдан AK-47");
    }

    #[menu_action(id = 108)]
    fn on_menu_give_awp(player: &mut Player) {
        player.give_item("weapon_awp");
        player.print_center("[Test Suite] Выдана AWP");
    }

    #[menu_action(id = 109)]
    fn on_menu_give_deagle(player: &mut Player) {
        player.give_item("weapon_deagle");
        player.print_center("[Test Suite] Выдан Deagle");
    }

    /// Tests classic HUD screen message.
    #[command(
        name = "test_hud",
        description = "Displays a classic HUD message on screen",
        usage = "test_hud [player_index]"
    )]
    fn handle_test_hud(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let player = Player::new(idx);
        if !player.is_valid() {
            log_warn!("[Test Suite] Player {} is not valid!", idx);
            return;
        }

        let hud = HudMessage::builder(
            "=== GoldSrc.rs Classic HUD ===\nПривет! Это проверка HUD сообщения.",
        )
        .classic(1)
        .position(-1.0, 0.25)
        .color(HudColor::new(0, 255, 128, 255))
        .timing(0.5, 0.5, 3.5)
        .build();

        player.send_hud(&hud);
        log_info!("[Test Suite] Sent classic HUD message to player #{}", idx);
    }

    /// Tests modern Director HUD (DHUD) screen message.
    #[command(
        name = "test_dhud",
        description = "Displays a DHUD message on screen",
        usage = "test_dhud [player_index]"
    )]
    fn handle_test_dhud(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let player = Player::new(idx);
        if !player.is_valid() {
            log_warn!("[Test Suite] Player {} is not valid!", idx);
            return;
        }

        let dhud = HudMessage::builder(
            "=== GoldSrc.rs DHUD ===\nТекст с эффектом Typewriter и кириллицей!",
        )
        .dhud()
        .position(-1.0, 0.35)
        .color(HudColor::new(255, 180, 0, 255))
        .color2(HudColor::new(255, 50, 0, 255))
        .effect(HudEffect::Typewriter {
            char_time: 0.05,
            fade_out: 0.5,
            hold_time: 4.0,
        })
        .build();

        player.send_hud(&dhud);
        log_info!("[Test Suite] Sent DHUD message to player #{}", idx);
    }

    /// Tests menu rendered as DHUD on screen.
    #[command(
        name = "test_dhud_menu",
        description = "Opens a test menu rendered via DHUD",
        usage = "test_dhud_menu [player_index]"
    )]
    fn handle_test_dhud_menu(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let player = Player::new(idx);
        if !player.is_valid() {
            log_warn!("[Test Suite] Player {} is not valid!", idx);
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
            .item(("Телепорт (+100 Z)", 105))
            .build();

        player.open_menu(&menu);
        log_info!("[Test Suite] Opened DHUD-rendered menu for player #{}", idx);
    }
}
