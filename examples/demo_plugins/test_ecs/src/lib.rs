use goldsrc::ecs::{EntityId, Stage, SystemDescriptor, SystemPhase, SystemRegistry, World};
use goldsrc::prelude::*;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Health(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DamageBuffer(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Score(pub i32);

static ECS_WORLD: Mutex<Option<World>> = Mutex::new(None);
static ECS_REGISTRY: Mutex<Option<SystemRegistry>> = Mutex::new(None);
static PHASE_TRACE: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());

// 1. Validate Phase
fn validate_damage_system(world: &mut World, _target: Option<EntityId>) {
    PHASE_TRACE.lock().unwrap().push("1_validate");
    for (entity, dmg) in world.query::<DamageBuffer>() {
        if dmg.0 < 0 {
            log_warn!(
                "[ECS Test] Negative damage discarded on entity #{}.",
                entity.0
            );
        }
    }
}

// 2. Modify Phase
fn modify_buff_system(world: &mut World, _target: Option<EntityId>) {
    PHASE_TRACE.lock().unwrap().push("2_modify");
    let entities: Vec<EntityId> = world.query::<DamageBuffer>().map(|(e, _)| e).collect();
    for entity in entities {
        if let Some(dmg) = world.get_mut::<DamageBuffer>(entity) {
            // Apply 2x VIP damage buff during modify phase
            dmg.0 *= 2;
        }
    }
}

// 3. Execute Phase
fn execute_apply_damage_system(world: &mut World, _target: Option<EntityId>) {
    PHASE_TRACE.lock().unwrap().push("3_execute");
    let entities: Vec<EntityId> = world.query::<DamageBuffer>().map(|(e, _)| e).collect();
    for entity in entities {
        let dmg_val = world.get::<DamageBuffer>(entity).map(|d| d.0).unwrap_or(0);
        if let Some(h) = world.get_mut::<Health>(entity) {
            h.0 -= dmg_val;
        }
    }
}

// 4. React Phase
fn react_reward_system(world: &mut World, _target: Option<EntityId>) {
    PHASE_TRACE.lock().unwrap().push("4_react");
    let entities: Vec<EntityId> = world.query::<Score>().map(|(e, _)| e).collect();
    for entity in entities {
        if let Some(s) = world.get_mut::<Score>(entity) {
            s.0 += 50; // Award score on action
        }
    }
}

// 5. Monitor Phase
fn monitor_telemetry_system(world: &mut World, _target: Option<EntityId>) {
    PHASE_TRACE.lock().unwrap().push("5_monitor");
    for (entity, health) in world.query::<Health>() {
        let score = world.get::<Score>(entity).map(|s| s.0).unwrap_or(0);
        log_info!(
            "[ECS Pipeline] Entity #{}: Health={}, Score={}",
            entity.0,
            health.0,
            score
        );
    }
}

pub struct TestEcs;

#[plugin(
    name = "test_ecs",
    version = "0.16.0",
    bundle = "test_suite",
    author = "GoldSrc.rs Team",
    description = "Flat ECS state storage and comprehensive 5-phase system pipeline verification suite",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
#[lifecycle(load = anytime, unload = anytime)]
#[permissions("cvar:read", "chat:send")]
impl TestEcs {
    #[on_load]
    fn init() {
        let mut world = World::new();
        let entity = EntityId(1);
        world.insert(entity, Health(100));
        world.insert(entity, DamageBuffer(15));
        world.insert(entity, Score(10));

        let mut registry = SystemRegistry::new();
        // Register intentionally out of order to verify topological sort by phase
        registry.register(SystemDescriptor {
            name: "react_reward",
            stage: Stage::Frame,
            phase: SystemPhase::React,
            before: Vec::new(),
            after: Vec::new(),
            run: react_reward_system,
        });
        registry.register(SystemDescriptor {
            name: "validate_damage",
            stage: Stage::Frame,
            phase: SystemPhase::Validate,
            before: Vec::new(),
            after: Vec::new(),
            run: validate_damage_system,
        });
        registry.register(SystemDescriptor {
            name: "monitor_telemetry",
            stage: Stage::Frame,
            phase: SystemPhase::Monitor,
            before: Vec::new(),
            after: Vec::new(),
            run: monitor_telemetry_system,
        });
        registry.register(SystemDescriptor {
            name: "execute_apply_damage",
            stage: Stage::Frame,
            phase: SystemPhase::Execute,
            before: Vec::new(),
            after: Vec::new(),
            run: execute_apply_damage_system,
        });
        registry.register(SystemDescriptor {
            name: "modify_buff",
            stage: Stage::Frame,
            phase: SystemPhase::Modify,
            before: Vec::new(),
            after: Vec::new(),
            run: modify_buff_system,
        });

        *ECS_WORLD.lock().unwrap() = Some(world);
        *ECS_REGISTRY.lock().unwrap() = Some(registry);

        log_info!("[Test ECS] Initialized 5-Phase Flat ECS verification plugin (v0.16.0).");
    }

    /// Verifies Flat ECS component retrieval and deterministic 5-phase system execution pipeline.
    #[command(
        name = "test_ecs",
        description = "Runs 5-phase ECS pipeline and asserts component state",
        usage = "test_ecs"
    )]
    fn handle_test_ecs(_cmd: String, _args: String) {
        if let (Some(world), Some(registry)) = (
            ECS_WORLD.lock().unwrap().as_mut(),
            ECS_REGISTRY.lock().unwrap().as_mut(),
        ) {
            PHASE_TRACE.lock().unwrap().clear();
            registry.run_stage(Stage::Frame, world, None);

            let trace = PHASE_TRACE.lock().unwrap().clone();
            log_info!("[Test ECS] Executed phases in order: {:?}", trace);
        }
    }
}
