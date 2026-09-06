//! Universal Entity and Player Property System (`Property<Target>` & `MutProperty<Target>`).
//!
//! Provides a type-safe, extensible property querying (`get(prop)`) and mutation (`set(prop, val)`)
//! model with zero runtime cost (ZST markers) and compile-time protection for read-only attributes.

use crate::client::Player;
use crate::{Entity, Vector3};

pub use crate::auth::property::Capability;
pub use crate::client::property::{Lang, Name, PlayerLifeState, PlayerTeam};

/// Trait for read-only or read-write properties on a given `Target` entity or handle.
pub trait Property<Target> {
    /// Type of the value returned by this property.
    type Value;

    /// Reads the property value from the given target.
    fn get(&self, target: &Target) -> Self::Value;
}

/// Trait for mutable properties that can be modified on `Target`.
pub trait MutProperty<Target>: Property<Target> {
    /// Writes the new property value to the target.
    fn set(&self, target: &mut Target, val: Self::Value);
}

// --- Spatial Properties ---

/// Entity or player 3D world origin coordinates (`Vector3`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Origin;

impl Property<Entity> for Origin {
    type Value = Vector3;

    #[inline(always)]
    fn get(&self, target: &Entity) -> Self::Value {
        #[cfg(target_arch = "wasm32")]
        {
            let v = crate::bindings::goldsrc::engine::api::host_entity_origin(target.index);
            Vector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.origin().unwrap_or([0.0, 0.0, 0.0]).into()
        }
    }
}

impl MutProperty<Entity> for Origin {
    #[inline(always)]
    fn set(&self, target: &mut Entity, val: Self::Value) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_set_origin(
                target.index,
                crate::bindings::goldsrc::engine::api::Vector3 {
                    x: val.x,
                    y: val.y,
                    z: val.z,
                },
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_origin(val.into());
        }
    }
}

impl Property<Player> for Origin {
    type Value = Vector3;

    #[inline(always)]
    fn get(&self, target: &Player) -> Self::Value {
        Property::<Entity>::get(self, target)
    }
}

impl MutProperty<Player> for Origin {
    #[inline(always)]
    fn set(&self, target: &mut Player, val: Self::Value) {
        MutProperty::<Entity>::set(self, target, val);
    }
}

/// Entity or player velocity vector (`Vector3`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Velocity;

impl Property<Entity> for Velocity {
    type Value = Vector3;

    #[inline(always)]
    fn get(&self, target: &Entity) -> Self::Value {
        #[cfg(target_arch = "wasm32")]
        {
            let v = crate::bindings::goldsrc::engine::api::host_entity_velocity(target.index);
            Vector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.velocity().unwrap_or([0.0, 0.0, 0.0]).into()
        }
    }
}

impl MutProperty<Entity> for Velocity {
    #[inline(always)]
    fn set(&self, target: &mut Entity, val: Self::Value) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_set_velocity(
                target.index,
                crate::bindings::goldsrc::engine::api::Vector3 {
                    x: val.x,
                    y: val.y,
                    z: val.z,
                },
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_velocity(val.into());
        }
    }
}

impl Property<Player> for Velocity {
    type Value = Vector3;

    #[inline(always)]
    fn get(&self, target: &Player) -> Self::Value {
        Property::<Entity>::get(self, target)
    }
}

impl MutProperty<Player> for Velocity {
    #[inline(always)]
    fn set(&self, target: &mut Player, val: Self::Value) {
        MutProperty::<Entity>::set(self, target, val);
    }
}

/// Entity or player view angles (pitch, yaw, roll) (`Vector3`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Angles;

impl Property<Entity> for Angles {
    type Value = Vector3;

    #[inline(always)]
    fn get(&self, target: &Entity) -> Self::Value {
        #[cfg(target_arch = "wasm32")]
        {
            let v = crate::bindings::goldsrc::engine::api::host_entity_angles(target.index);
            Vector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.angles().unwrap_or([0.0, 0.0, 0.0]).into()
        }
    }
}

impl MutProperty<Entity> for Angles {
    #[inline(always)]
    fn set(&self, target: &mut Entity, val: Self::Value) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_set_angles(
                target.index,
                crate::bindings::goldsrc::engine::api::Vector3 {
                    x: val.x,
                    y: val.y,
                    z: val.z,
                },
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_angles(val.into());
        }
    }
}

impl Property<Player> for Angles {
    type Value = Vector3;

    #[inline(always)]
    fn get(&self, target: &Player) -> Self::Value {
        Property::<Entity>::get(self, target)
    }
}

impl MutProperty<Player> for Angles {
    #[inline(always)]
    fn set(&self, target: &mut Player, val: Self::Value) {
        MutProperty::<Entity>::set(self, target, val);
    }
}

// --- Vital State Properties ---

/// Player or entity health points (`f32`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Health;

impl Property<Entity> for Health {
    type Value = f32;

    #[inline(always)]
    fn get(&self, target: &Entity) -> Self::Value {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_health(target.index)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.health().unwrap_or(0.0)
        }
    }
}

impl MutProperty<Entity> for Health {
    #[inline(always)]
    fn set(&self, target: &mut Entity, val: Self::Value) {
        if !val.is_finite() {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_set_health(target.index, val);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_health(val);
        }
    }
}

impl Property<Player> for Health {
    type Value = f32;

    #[inline(always)]
    fn get(&self, target: &Player) -> Self::Value {
        Property::<Entity>::get(self, target)
    }
}

impl MutProperty<Player> for Health {
    #[inline(always)]
    fn set(&self, target: &mut Player, val: Self::Value) {
        MutProperty::<Entity>::set(self, target, val);
    }
}

/// Player armor value (`f32`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Armor;

impl Property<Player> for Armor {
    type Value = f32;

    #[inline(always)]
    fn get(&self, target: &Player) -> Self::Value {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_player_armorvalue(target.index)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.armorvalue().unwrap_or(0.0)
        }
    }
}

impl MutProperty<Player> for Armor {
    #[inline(always)]
    fn set(&self, target: &mut Player, val: Self::Value) {
        if !val.is_finite() {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_player_set_armorvalue(target.index, val);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_armorvalue(val);
        }
    }
}

// --- Identity Properties ---

/// Entity class name (`Option<String>` / Read-Only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Classname;

impl Property<Entity> for Classname {
    type Value = Option<String>;

    #[inline(always)]
    fn get(&self, target: &Entity) -> Self::Value {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_classname(target.index)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.classname()
        }
    }
}

impl Property<Player> for Classname {
    type Value = Option<String>;

    #[inline(always)]
    fn get(&self, target: &Player) -> Self::Value {
        Property::<Entity>::get(self, target)
    }
}

/// Standard engine properties namespace for backward compatibility (`crate::property::prop::*`).
pub mod prop {
    pub use super::{
        Angles, Armor, Capability, Classname, Health, Lang, Name, Origin, PlayerLifeState,
        PlayerTeam, Velocity,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vector3;
    use crate::client::{LifeState, Player, Team};

    #[test]
    fn test_property_zst_markers() {
        let player = Player::new(1);
        let _health: f32 = player.get(Health);
        let _armor: f32 = player.get(Armor);
        let _origin: Vector3 = player.get(Origin);
        let _velocity: Vector3 = player.get(Velocity);
        let _angles: Vector3 = player.get(Angles);
        let _team: Team = player.get(PlayerTeam);
        let _life_state: LifeState = player.get(PlayerLifeState);
    }

    #[test]
    fn test_parameterized_property_capability() {
        crate::auth::Auth::register_capability("prop.test.unique_ban", "ban permission");
        let mut player = Player::new(78);
        crate::auth::Auth::remove_player(78);
        assert!(!player.get(Capability("prop.test.unique_ban")));

        player.set(Capability("prop.test.unique_ban"), true);
        assert!(player.get(Capability("prop.test.unique_ban")));

        player.set(Capability("prop.test.unique_ban"), false);
        assert!(!player.get(Capability("prop.test.unique_ban")));
    }
}
