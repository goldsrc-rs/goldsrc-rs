//! Fluent entity instantiation builder for GoldSrc entities with keyvalues.

use crate::entity::Entity;
use crate::entity::keys;
use crate::entity::types::{RenderFx, RenderMode, SolidType};

#[cfg(target_arch = "wasm32")]
use crate::bindings::goldsrc::engine::api as host_api;

/// Abstract entity spawning operations required for [`EntityBuilder`].
pub trait EntitySpawner: Send + Sync {
    /// Creates a new named entity (e.g. "env_sprite", "info_target").
    /// Returns the newly allocated entity index.
    fn create_named_entity(&self, classname: &str) -> Option<i32>;

    /// Sets an entity's world position.
    fn entity_set_origin(&self, index: i32, pos: [f32; 3]);

    /// Sets an entity's rotation angles.
    fn entity_set_angles(&self, index: i32, angles: [f32; 3]);

    /// Sets an entity key-value string property (`pfnKeyValue`).
    /// Returns `true` if handled.
    fn entity_key_value(&self, index: i32, key: &str, value: &str) -> bool;

    /// Dispatches spawn call on the entity (`pfnSpawn`).
    fn dispatch_spawn(&self, index: i32) -> i32;
}

/// Fluent builder for constructing and spawning GoldSrc entities with pre-spawn keyvalues.
#[derive(Debug, Clone)]
pub struct EntityBuilder {
    classname: String,
    origin: Option<[f32; 3]>,
    angles: Option<[f32; 3]>,
    key_values: Vec<(String, String)>,
    auto_spawn: bool,
}

impl EntityBuilder {
    /// Creates a new builder for the specified entity classname (e.g. "env_sprite", "info_target").
    pub fn new(classname: impl Into<String>) -> Self {
        Self {
            classname: classname.into(),
            origin: None,
            angles: None,
            key_values: Vec::new(),
            auto_spawn: true,
        }
    }

    /// Sets a raw key-value pair to be dispatched via `pfnKeyValue` before spawning.
    pub fn key(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.key_values.push((key.into(), val.into()));
        self
    }

    /// Sets the entity target name (`keys::TARGET_NAME`).
    pub fn target_name(self, name: impl Into<String>) -> Self {
        self.key(keys::TARGET_NAME, name)
    }

    /// Sets the target to trigger (`keys::TARGET`).
    pub fn target(self, target: impl Into<String>) -> Self {
        self.key(keys::TARGET, target)
    }

    /// Sets the entity model path or brush id (`keys::MODEL`).
    pub fn model(self, model: impl Into<String>) -> Self {
        self.key(keys::MODEL, model)
    }

    /// Sets the entity initial world origin.
    pub fn origin(mut self, origin: impl Into<[f32; 3]>) -> Self {
        let pos = origin.into();
        self.origin = Some(pos);
        self.key(keys::ORIGIN, format!("{} {} {}", pos[0], pos[1], pos[2]))
    }

    /// Sets the entity initial Euler angles.
    pub fn angles(mut self, angles: impl Into<[f32; 3]>) -> Self {
        let ang = angles.into();
        self.angles = Some(ang);
        self.key(keys::ANGLES, format!("{} {} {}", ang[0], ang[1], ang[2]))
    }

    /// Sets the entity rendering mode (`keys::RENDER_MODE`).
    pub fn render_mode(self, mode: RenderMode) -> Self {
        self.key(keys::RENDER_MODE, mode.as_raw().to_string())
    }

    /// Sets the entity rendering amount / alpha (`keys::RENDER_AMT`).
    pub fn render_amt(self, amt: f32) -> Self {
        self.key(keys::RENDER_AMT, amt.to_string())
    }

    /// Sets the entity rendering color (`keys::RENDER_COLOR`).
    pub fn render_color(self, r: f32, g: f32, b: f32) -> Self {
        self.key(keys::RENDER_COLOR, format!("{r} {g} {b}"))
    }

    /// Sets the entity rendering effect (`keys::RENDER_FX`).
    pub fn render_fx(self, fx: RenderFx) -> Self {
        self.key(keys::RENDER_FX, fx.as_raw().to_string())
    }

    /// Sets the entity collision solidity (`keys::SOLID`).
    pub fn solid(self, solid: SolidType) -> Self {
        self.key(keys::SOLID, solid.as_raw().to_string())
    }

    /// Sets whether `dispatch_spawn` should be automatically invoked on completion. Default is `true`.
    pub fn auto_spawn(mut self, spawn: bool) -> Self {
        self.auto_spawn = spawn;
        self
    }

    /// Builds and parameterizes the entity against an explicit [`EntitySpawner`] implementation.
    pub fn build_with<E: EntitySpawner + ?Sized>(&self, engine: &E) -> Option<Entity> {
        let index = engine.create_named_entity(&self.classname)?;
        if let Some(pos) = self.origin {
            engine.entity_set_origin(index, pos);
        }
        if let Some(ang) = self.angles {
            engine.entity_set_angles(index, ang);
        }

        // Engine Contract: pfnKeyValue MUST be called before pfnSpawn
        for (k, v) in &self.key_values {
            let _ = engine.entity_key_value(index, k, v);
        }

        if self.auto_spawn {
            let _ = engine.dispatch_spawn(index);
        }

        Some(Entity::new(index))
    }

    /// Builds and spawns the entity using the active environment (WASM host API or placeholder).
    pub fn spawn(self) -> Option<Entity> {
        #[cfg(target_arch = "wasm32")]
        {
            let index = host_api::host_create_named_entity(&self.classname)?;
            if let Some(pos) = self.origin {
                host_api::host_entity_set_origin(
                    index,
                    host_api::Vector3 {
                        x: pos[0],
                        y: pos[1],
                        z: pos[2],
                    },
                );
            }
            if let Some(ang) = self.angles {
                host_api::host_entity_set_angles(
                    index,
                    host_api::Vector3 {
                        x: ang[0],
                        y: ang[1],
                        z: ang[2],
                    },
                );
            }

            // Engine Contract: pfnKeyValue MUST be called before pfnSpawn
            for (k, v) in &self.key_values {
                host_api::host_entity_key_value(index, k, v);
            }

            if self.auto_spawn {
                host_api::host_dispatch_spawn(index);
            }

            Some(Entity::new(index))
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Some(Entity::new(100))
        }
    }
}
