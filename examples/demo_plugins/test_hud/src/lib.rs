use goldsrc::prelude::*;

pub struct TestHud;

#[plugin(
    name = "test_hud",
    version = "0.13.0",
    bundle = "test_suite",
    author = "GoldSrc.rs Team",
    description = "HUD, DHUD, and Screen Effects verification suite",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl TestHud {
    #[on_load]
    fn init() {
        log_info!("[Test HUD] Initialized HUD and visual effects verification plugin (v0.13.0).");
    }

    /// Tests classic 4-channel HUD screen message.
    #[command(
        name = "test_hud",
        description = "Displays a classic HUD message on screen",
        usage = "test_hud [player_index]"
    )]
    fn handle_test_hud(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let player = Player::new(idx);
        if !player.is_valid() {
            log_warn!("[Test HUD] Player {} is not valid!", idx);
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
        log_info!("[Test HUD] Sent classic HUD message to player #{}", idx);
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
            log_warn!("[Test HUD] Player {} is not valid!", idx);
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
        log_info!("[Test HUD] Sent DHUD message to player #{}", idx);
    }
}
