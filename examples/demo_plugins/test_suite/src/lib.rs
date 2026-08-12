use goldsrc::{log_info, log_warn, plugin, EntityId, Vector3, World};

#[derive(Debug, PartialEq)]
struct StatsComponent {
    kills: u32,
    deaths: u32,
}

pub struct TestSuite;

#[plugin(name = "test_suite", version = "1.0.0", author = "Oleg")]
impl TestSuite {
    #[on_load]
    fn init() {
        log_info!("[Test Suite] Plugin loaded with Component Model bindings!");

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
                "[Test Suite] Player 1 Stats: Kills={}, Deaths={}",
                stats.kills,
                stats.deaths
            );
        }

        let pos = Vector3 {
            x: 100.0,
            y: 200.0,
            z: -50.0,
        };
        log_info!(
            "[Test Suite] Vector3 Test: ({}, {}, {})",
            pos.x,
            pos.y,
            pos.z
        );
    }

    #[event]
    fn handle_event(name: String, data: Vec<u8>) {
        if let Ok(str_data) = String::from_utf8(data) {
            log_warn!(
                "[Test Suite] Event Handler received: '{}' => {}",
                name,
                str_data
            );
        }
    }

    #[command(name = "testcmd")]
    fn handle_testcmd(cmd: String, args: String) {
        log_info!(
            "[Test Suite] Command Handler '{}' executed with args: '{}'",
            cmd,
            args
        );
    }
}
