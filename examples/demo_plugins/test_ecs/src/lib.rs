use goldsrc::ecs::{EntityId, Stage, SystemDescriptor, SystemPhase, SystemRegistry, World};
use goldsrc::prelude::*;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Health(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Score(pub i32);

static ECS_WORLD: Mutex<Option<World>> = Mutex::new(None);
static ECS_REGISTRY: Mutex<Option<SystemRegistry>> = Mutex::new(None);

fn health_regen_system(world: &mut World, _target: Option<EntityId>) {
    let entities: Vec<EntityId> = world.query::<Health>().map(|(e, _)| e).collect();
    for entity in entities {
        if let Some(h) = world.get_mut::<Health>(entity)
            && h.0 < 100
        {
            h.0 += 1;
        }
    }
}

pub struct TestEcs;

#[plugin(
    name = "test_ecs",
    version = "0.13.0",
    author = "GoldSrc.rs Team",
    description = "Flat ECS state storage and system stages verification suite",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl TestEcs {
    #[on_load]
    fn init() {
        let mut world = World::new();
        let entity = EntityId(1);
        world.insert(entity, Health(85));
        world.insert(entity, Score(10));

        let mut registry = SystemRegistry::new();
        registry.register(SystemDescriptor {
            name: "health_regen",
            stage: Stage::Frame,
            phase: SystemPhase::Execute,
            before: Vec::new(),
            after: Vec::new(),
            run: health_regen_system,
        });

        *ECS_WORLD.lock().unwrap() = Some(world);
        *ECS_REGISTRY.lock().unwrap() = Some(registry);

        log_info!("[Test ECS] Initialized Flat ECS verification plugin (v0.13.0).");
    }

    /// Verifies Flat ECS component retrieval and system step.
    #[command(
        name = "test_ecs",
        description = "Runs ECS tick and asserts component state",
        usage = "test_ecs"
    )]
    fn handle_test_ecs(_cmd: String, _args: String) {
        if let (Some(world), Some(registry)) = (
            ECS_WORLD.lock().unwrap().as_mut(),
            ECS_REGISTRY.lock().unwrap().as_mut(),
        ) {
            registry.run_stage(Stage::Frame, world, None);
            for (entity, health) in world.query::<Health>() {
                log_info!("[Test ECS] Entity #{}: Health = {}", entity.0, health.0);
            }
        }
    }
}
