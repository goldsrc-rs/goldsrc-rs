use goldsrc::{EntityId, Vector3, World, command, event, log_info, log_warn, on_load, plugin};

#[derive(Debug, PartialEq)]
struct StatsComponent {
    kills: u32,
    deaths: u32,
}

#[plugin(
    name = "test_suite",
    version = "1.0.0",
    author = "Oleg",
    systems = ["TestSystem"]
)]
pub struct TestSuite;

#[on_load]
fn init() {
    log_info!("[Test Suite] Plugin loaded with zero unsafe code!");

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
pub fn handle_event(name: &str, data: &str) {
    log_warn!(
        "[Test Suite] Event Handler received: '{}' => {}",
        name,
        data
    );
}

#[command(name = "testcmd")]
pub fn handle_testcmd(cmd: &str, args: &str) {
    log_info!(
        "[Test Suite] Command Handler '{}' executed with args: '{}'",
        cmd,
        args
    );
}
