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

    #[event]
    fn handle_event(name: String, data: Vec<u8>) {
        // Filter out high-frequency per-frame events to avoid console spam
        if name == "player_post_think"
            || name == "player_pre_think"
            || name == "entity_touch"
            || name == "on_frame"
        {
            return;
        }

        if data.is_empty() {
            log_info!(
                "[Test Suite] Event Handler received: '{}' (no payload)",
                name
            );
        } else if data.len() == 4 {
            let idx = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            log_info!(
                "[Test Suite] Event Handler received: '{}' => player #{}",
                name,
                idx
            );
        } else if let Ok(str_data) = String::from_utf8(data.clone()) {
            log_info!(
                "[Test Suite] Event Handler received: '{}' => '{}'",
                name,
                str_data
            );
        } else {
            log_info!(
                "[Test Suite] Event Handler received: '{}' => (raw bytes: {:?})",
                name,
                data
            );
        }
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

    /// Tests playing a sound on player 1.
    #[command(
        name = "test_sound",
        description = "Emits a test audio sample on player 1",
        usage = "test_sound [sound_path]"
    )]
    fn handle_test_sound(_cmd: String, args: String) {
        let sample = if args.trim().is_empty() {
            "events/tutor_msg.wav"
        } else {
            args.trim()
        };
        engine::emit_sound(1, 0, sample, 1.0, 0.8, 0, 100);
        log_info!("[Test Suite] Emitted sound '{}' on entity #1", sample);
    }
}
