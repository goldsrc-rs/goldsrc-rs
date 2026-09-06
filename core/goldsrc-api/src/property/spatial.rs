//! Spatial and physics-related entity properties (Origin, Velocity, Angles).

use crate::client::Player;
use crate::property::{MutProperty, Property};
use crate::{Entity, Vector3};

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
