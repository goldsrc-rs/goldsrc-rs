//! Flat Entity Component System (ECS) for GoldSrc WASM plugins.
//!
//! GoldSrc entities have a fixed index range:
//! - 0: World
//! - 1..=32: Players (Max clients)
//! - 33..=2048: Map entities & edicts

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Entity identifier mapping to GoldSrc edict indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub u16);

impl EntityId {
    /// World entity (index 0).
    pub const WORLD: EntityId = EntityId(0);

    /// Check if entity is a player (index 1 to 32).
    pub fn is_player(self) -> bool {
        (1..=goldsrc_api::consts::MAX_PLAYERS).contains(&self.0)
    }

    /// Check if entity is the world (index 0).
    pub fn is_world(self) -> bool {
        self.0 == 0
    }
}

/// Fast O(1) Component storage for GoldSrc entities.
pub struct ComponentStorage<T> {
    dense: Vec<Option<T>>,
}

impl<T> Default for ComponentStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ComponentStorage<T> {
    /// Creates an empty storage pre-sized for the player range.
    pub fn new() -> Self {
        Self {
            dense: Vec::with_capacity(33),
        }
    }

    /// Inserts `component` for `entity`, growing the backing storage as needed.
    pub fn insert(&mut self, entity: EntityId, component: T) {
        let idx = entity.0 as usize;
        if idx >= self.dense.len() {
            self.dense.resize_with(idx + 1, || None);
        }
        self.dense[idx] = Some(component);
    }

    /// Returns a shared reference to `entity`'s component, if present.
    pub fn get(&self, entity: EntityId) -> Option<&T> {
        let idx = entity.0 as usize;
        if idx < self.dense.len() {
            self.dense[idx].as_ref()
        } else {
            None
        }
    }

    /// Returns a mutable reference to `entity`'s component, if present.
    pub fn get_mut(&mut self, entity: EntityId) -> Option<&mut T> {
        let idx = entity.0 as usize;
        if idx < self.dense.len() {
            self.dense[idx].as_mut()
        } else {
            None
        }
    }

    /// Removes and returns `entity`'s component, if present.
    pub fn remove(&mut self, entity: EntityId) -> Option<T> {
        let idx = entity.0 as usize;
        if idx < self.dense.len() {
            self.dense[idx].take()
        } else {
            None
        }
    }
}

/// Flat World for WASM plugin ECS.
#[derive(Default)]
pub struct World {
    storages: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl World {
    /// Creates an empty world.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts `component` for `entity` into the per-type storage.
    pub fn insert<T: Send + Sync + 'static>(&mut self, entity: EntityId, component: T) {
        let type_id = TypeId::of::<T>();
        let storage = self
            .storages
            .entry(type_id)
            .or_insert_with(|| Box::new(ComponentStorage::<T>::new()));

        let storage = storage
            .downcast_mut::<ComponentStorage<T>>()
            .expect("TypeId mismatch in ECS storage");

        storage.insert(entity, component);
    }

    /// Returns a shared reference to `entity`'s component of type `T`, if present.
    pub fn get<T: 'static>(&self, entity: EntityId) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let storage = self.storages.get(&type_id)?;
        let storage = storage
            .downcast_ref::<ComponentStorage<T>>()
            .expect("TypeId mismatch in ECS storage");
        storage.get(entity)
    }

    /// Returns a mutable reference to `entity`'s component of type `T`, if present.
    pub fn get_mut<T: 'static>(&mut self, entity: EntityId) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let storage = self.storages.get_mut(&type_id)?;
        let storage = storage
            .downcast_mut::<ComponentStorage<T>>()
            .expect("TypeId mismatch in ECS storage");
        storage.get_mut(entity)
    }

    /// Removes and returns `entity`'s component of type `T`, if present.
    pub fn remove<T: 'static>(&mut self, entity: EntityId) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let storage = self.storages.get_mut(&type_id)?;
        let storage = storage
            .downcast_mut::<ComponentStorage<T>>()
            .expect("TypeId mismatch in ECS storage");
        storage.remove(entity)
    }

    /// Queries all entities having a component of type `T`.
    pub fn query<T: 'static>(&self) -> impl Iterator<Item = (EntityId, &T)> {
        let type_id = TypeId::of::<T>();
        let storage = self
            .storages
            .get(&type_id)
            .and_then(|s| s.downcast_ref::<ComponentStorage<T>>());

        let items: Vec<(EntityId, &T)> = match storage {
            Some(s) => s
                .dense
                .iter()
                .enumerate()
                .filter_map(|(idx, opt)| opt.as_ref().map(|comp| (EntityId(idx as u16), comp)))
                .collect(),
            None => Vec::new(),
        };

        items.into_iter()
    }
}

/// Semantic intent phase of a system within a stage or event lifecycle.
///
/// Ensures deterministic inter-plugin execution without priority guessing wars:
/// `Validate` -> `Modify` -> `Execute` -> `React` -> `Monitor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum SystemPhase {
    /// 1. Early sanity checks, anti-cheat, authorization preconditions.
    Validate = 0,
    /// 2. Data/parameter mutations, multipliers, VIP buffs, discounts.
    Modify = 10,
    /// 3. Core primary execution step (default).
    #[default]
    Execute = 20,
    /// 4. Post-execution side effects, rewards, sounds, animations.
    React = 30,
    /// 5. Read-only logging, analytics, damage informers, telemetry.
    Monitor = 40,
}

impl SystemPhase {
    /// List of all system phases in deterministic execution order.
    pub const ALL: &'static [SystemPhase] = &[
        SystemPhase::Validate,
        SystemPhase::Modify,
        SystemPhase::Execute,
        SystemPhase::React,
        SystemPhase::Monitor,
    ];

    /// Returns the static string representation of this phase.
    pub const fn as_str(&self) -> &'static str {
        match self {
            SystemPhase::Validate => "validate",
            SystemPhase::Modify => "modify",
            SystemPhase::Execute => "execute",
            SystemPhase::React => "react",
            SystemPhase::Monitor => "monitor",
        }
    }
}

impl std::fmt::Display for SystemPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for SystemPhase {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "validate" => Ok(SystemPhase::Validate),
            "modify" => Ok(SystemPhase::Modify),
            "execute" => Ok(SystemPhase::Execute),
            "react" => Ok(SystemPhase::React),
            "monitor" => Ok(SystemPhase::Monitor),
            _ => Err(format!(
                "Unknown system phase: '{s}'. Expected: validate, modify, execute, react, monitor"
            )),
        }
    }
}

/// Execution stage for ECS systems in the GoldSrc lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Stage {
    /// Startup stage during plugin initialization (`on_load`).
    Startup = 0,
    /// Triggered when the map / server activates (`on_server_activate`).
    ServerActivate = 10,
    /// Triggered every server frame (`on_server_frame`).
    Frame = 20,
    /// Triggered during player post-think physics step (`player_post_think`).
    PostThink = 30,
    /// Triggered when a player connects to the server (`client_connect`).
    PlayerConnect = 40,
    /// Triggered when a player disconnects (`client_disconnect`).
    PlayerDisconnect = 50,
    /// Triggered when an entity spawns (`entity_spawn`).
    EntitySpawn = 60,
    /// Triggered when an entity takes damage (`entity_take_damage`).
    TakeDamage = 70,
    /// Triggered when an entity is killed (`entity_killed`).
    EntityKilled = 80,
    /// Triggered when a new round starts (`round_start`).
    RoundStart = 90,
    /// Triggered when a round ends (`round_end`).
    RoundEnd = 100,
    /// Triggered when the freeze period ends and players can move (`round_freeze_end`).
    RoundFreezeEnd = 110,
}

impl Stage {
    /// List of all lifecycle stages in order of declaration.
    pub const ALL: &'static [Stage] = &[
        Stage::Startup,
        Stage::ServerActivate,
        Stage::Frame,
        Stage::PostThink,
        Stage::PlayerConnect,
        Stage::PlayerDisconnect,
        Stage::EntitySpawn,
        Stage::TakeDamage,
        Stage::EntityKilled,
        Stage::RoundStart,
        Stage::RoundEnd,
        Stage::RoundFreezeEnd,
    ];

    /// Returns the static string representation of this stage.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Stage::Startup => "startup",
            Stage::ServerActivate => "server_activate",
            Stage::Frame => "frame",
            Stage::PostThink => "post_think",
            Stage::PlayerConnect => "player_connect",
            Stage::PlayerDisconnect => "player_disconnect",
            Stage::EntitySpawn => "entity_spawn",
            Stage::TakeDamage => "take_damage",
            Stage::EntityKilled => "entity_killed",
            Stage::RoundStart => "round_start",
            Stage::RoundEnd => "round_end",
            Stage::RoundFreezeEnd => "round_freeze_end",
        }
    }
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Stage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "startup" => Ok(Stage::Startup),
            "server_activate" => Ok(Stage::ServerActivate),
            "frame" => Ok(Stage::Frame),
            "post_think" => Ok(Stage::PostThink),
            "player_connect" => Ok(Stage::PlayerConnect),
            "player_disconnect" => Ok(Stage::PlayerDisconnect),
            "entity_spawn" => Ok(Stage::EntitySpawn),
            "take_damage" => Ok(Stage::TakeDamage),
            "entity_killed" => Ok(Stage::EntityKilled),
            "round_start" => Ok(Stage::RoundStart),
            "round_end" => Ok(Stage::RoundEnd),
            "round_freeze_end" => Ok(Stage::RoundFreezeEnd),
            _ => Err(format!(
                "Unknown stage: '{s}'. Expected: startup, server_activate, frame, post_think, player_connect, player_disconnect, entity_spawn, take_damage, entity_killed, round_start, round_end, round_freeze_end"
            )),
        }
    }
}

/// A registered system runner function.
pub type SystemFn = fn(world: &mut World, target: Option<EntityId>);

/// A registered system descriptor.
#[derive(Clone)]
pub struct SystemDescriptor {
    pub name: &'static str,
    pub stage: Stage,
    pub phase: SystemPhase,
    pub before: Vec<&'static str>,
    pub after: Vec<&'static str>,
    pub run: SystemFn,
}

/// Global/Plugin-local registry of ECS systems ordered by stage, phase and DAG dependencies.
#[derive(Default)]
pub struct SystemRegistry {
    systems: Vec<SystemDescriptor>,
}

impl SystemRegistry {
    /// Creates an empty system registry.
    pub const fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Registers a new system descriptor and rebuilds execution order.
    pub fn register(&mut self, system: SystemDescriptor) {
        self.systems.push(system);
        self.sort_systems();
    }

    /// Sorts systems deterministically using (Stage, Phase) buckets + intra-phase DAG resolution.
    fn sort_systems(&mut self) {
        self.systems.sort_by_key(|s| (s.stage, s.phase));

        // Group by stage and phase to resolve intra-phase DAG dependencies (before/after)
        let mut resolved = Vec::with_capacity(self.systems.len());
        let mut i = 0;
        while i < self.systems.len() {
            let stage = self.systems[i].stage;
            let phase = self.systems[i].phase;
            let mut j = i;
            while j < self.systems.len()
                && self.systems[j].stage == stage
                && self.systems[j].phase == phase
            {
                j += 1;
            }

            let mut bucket = self.systems[i..j].to_vec();
            bucket = Self::topological_sort_bucket(bucket);
            resolved.extend(bucket);
            i = j;
        }

        self.systems = resolved;
    }

    /// Topologically sorts systems within the same (Stage, Phase) bucket.
    fn topological_sort_bucket(bucket: Vec<SystemDescriptor>) -> Vec<SystemDescriptor> {
        let n = bucket.len();
        if n <= 1 {
            return bucket;
        }

        let mut name_to_idx = HashMap::new();
        for (idx, sys) in bucket.iter().enumerate() {
            name_to_idx.insert(sys.name, idx);
        }

        let mut in_degree = vec![0; n];
        let mut adj = vec![Vec::new(); n];

        for (u, sys) in bucket.iter().enumerate() {
            // 'after = "foo"' means foo -> u (foo runs before u)
            for after_name in &sys.after {
                if let Some(&v) = name_to_idx.get(after_name) {
                    adj[v].push(u);
                    in_degree[u] += 1;
                }
            }
            // 'before = "bar"' means u -> bar (u runs before bar)
            for before_name in &sys.before {
                if let Some(&v) = name_to_idx.get(before_name) {
                    adj[u].push(v);
                    in_degree[v] += 1;
                }
            }
        }

        let mut queue = std::collections::VecDeque::new();
        for (idx, &deg) in in_degree.iter().enumerate() {
            if deg == 0 {
                queue.push_back(idx);
            }
        }

        let mut sorted = Vec::with_capacity(n);
        while let Some(u) = queue.pop_front() {
            sorted.push(bucket[u].clone());
            for &v in &adj[u] {
                in_degree[v] -= 1;
                if in_degree[v] == 0 {
                    queue.push_back(v);
                }
            }
        }

        // If cycle or unvisited remainder, append remaining deterministically
        if sorted.len() < n {
            for (idx, sys) in bucket.into_iter().enumerate() {
                if in_degree[idx] > 0 {
                    sorted.push(sys);
                }
            }
        }

        sorted
    }

    /// Runs all systems registered for the specified `stage`.
    pub fn run_stage(&self, stage: Stage, world: &mut World, target: Option<EntityId>) {
        for sys in self.systems.iter().filter(|s| s.stage == stage) {
            (sys.run)(world, target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct VipData {
        level: u8,
    }

    #[test]
    fn test_flat_ecs() {
        let mut world = World::new();
        let player = EntityId(1);

        assert!(player.is_player());
        assert!(!player.is_world());

        world.insert(player, VipData { level: 3 });
        assert_eq!(world.get::<VipData>(player), Some(&VipData { level: 3 }));

        let queried: Vec<_> = world.query::<VipData>().collect();
        assert_eq!(queried.len(), 1);
        assert_eq!(queried[0], (player, &VipData { level: 3 }));

        world.remove::<VipData>(player);
        assert_eq!(world.get::<VipData>(player), None);
    }

    #[test]
    fn test_system_stages_and_order() {
        static EXEC_LOG: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());

        fn sys_validate(_world: &mut World, _target: Option<EntityId>) {
            EXEC_LOG.lock().unwrap().push("validate");
        }

        fn sys_modify(_world: &mut World, _target: Option<EntityId>) {
            EXEC_LOG.lock().unwrap().push("modify");
        }

        fn sys_exec_1(_world: &mut World, _target: Option<EntityId>) {
            EXEC_LOG.lock().unwrap().push("exec_1");
        }

        fn sys_exec_2(_world: &mut World, _target: Option<EntityId>) {
            EXEC_LOG.lock().unwrap().push("exec_2");
        }

        fn sys_monitor(_world: &mut World, _target: Option<EntityId>) {
            EXEC_LOG.lock().unwrap().push("monitor");
        }

        let mut registry = SystemRegistry::new();
        // Register in intentionally scrambled order
        registry.register(SystemDescriptor {
            name: "monitor_sys",
            stage: Stage::Frame,
            phase: SystemPhase::Monitor,
            before: vec![],
            after: vec![],
            run: sys_monitor,
        });
        registry.register(SystemDescriptor {
            name: "exec_2",
            stage: Stage::Frame,
            phase: SystemPhase::Execute,
            before: vec![],
            after: vec!["exec_1"], // DAG constraint: exec_2 must run after exec_1
            run: sys_exec_2,
        });
        registry.register(SystemDescriptor {
            name: "exec_1",
            stage: Stage::Frame,
            phase: SystemPhase::Execute,
            before: vec![],
            after: vec![],
            run: sys_exec_1,
        });
        registry.register(SystemDescriptor {
            name: "validate_sys",
            stage: Stage::Frame,
            phase: SystemPhase::Validate,
            before: vec![],
            after: vec![],
            run: sys_validate,
        });
        registry.register(SystemDescriptor {
            name: "modify_sys",
            stage: Stage::Frame,
            phase: SystemPhase::Modify,
            before: vec![],
            after: vec![],
            run: sys_modify,
        });

        let mut world = World::new();
        EXEC_LOG.lock().unwrap().clear();
        registry.run_stage(Stage::Frame, &mut world, None);

        assert_eq!(
            *EXEC_LOG.lock().unwrap(),
            vec!["validate", "modify", "exec_1", "exec_2", "monitor"]
        );
    }

    #[test]
    fn test_five_phase_all_stages_execution() {
        static PHASES_HIT: std::sync::Mutex<Vec<SystemPhase>> = std::sync::Mutex::new(Vec::new());

        fn p_val(_world: &mut World, _target: Option<EntityId>) {
            PHASES_HIT.lock().unwrap().push(SystemPhase::Validate);
        }
        fn p_mod(_world: &mut World, _target: Option<EntityId>) {
            PHASES_HIT.lock().unwrap().push(SystemPhase::Modify);
        }
        fn p_exe(_world: &mut World, _target: Option<EntityId>) {
            PHASES_HIT.lock().unwrap().push(SystemPhase::Execute);
        }
        fn p_rea(_world: &mut World, _target: Option<EntityId>) {
            PHASES_HIT.lock().unwrap().push(SystemPhase::React);
        }
        fn p_mon(_world: &mut World, _target: Option<EntityId>) {
            PHASES_HIT.lock().unwrap().push(SystemPhase::Monitor);
        }

        let mut registry = SystemRegistry::new();

        type PhaseRunner = (SystemPhase, fn(&mut World, Option<EntityId>));
        let funcs: [PhaseRunner; 5] = [
            (SystemPhase::Monitor, p_mon),
            (SystemPhase::React, p_rea),
            (SystemPhase::Execute, p_exe),
            (SystemPhase::Modify, p_mod),
            (SystemPhase::Validate, p_val),
        ];

        for (phase, run_fn) in funcs {
            registry.register(SystemDescriptor {
                name: "phase_test",
                stage: Stage::TakeDamage,
                phase,
                before: vec![],
                after: vec![],
                run: run_fn,
            });
        }

        let mut world = World::new();
        PHASES_HIT.lock().unwrap().clear();

        registry.run_stage(Stage::TakeDamage, &mut world, Some(EntityId(1)));

        let hits = PHASES_HIT.lock().unwrap().clone();
        assert_eq!(hits.len(), 5);
        assert_eq!(
            hits,
            vec![
                SystemPhase::Validate,
                SystemPhase::Modify,
                SystemPhase::Execute,
                SystemPhase::React,
                SystemPhase::Monitor,
            ]
        );
    }
}
