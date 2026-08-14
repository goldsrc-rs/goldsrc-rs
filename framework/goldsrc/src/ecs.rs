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
        (1..=32).contains(&self.0)
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
    storages: HashMap<TypeId, Box<dyn Any>>,
}

impl World {
    /// Creates an empty world.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts `component` for `entity` into the per-type storage.
    pub fn insert<T: 'static>(&mut self, entity: EntityId, component: T) {
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

        world.remove::<VipData>(player);
        assert_eq!(world.get::<VipData>(player), None);
    }
}
