//! Spatial orientation and kinematic value objects (`Origin`, `Velocity`, `Angles`).

use crate::types::Vector3;
use std::ops::{Deref, DerefMut};

/// Entity or player 3D world origin coordinates (`Vector3`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Origin(pub Vector3);

impl Origin {
    /// Creates a new origin coordinate wrapper.
    #[inline]
    pub const fn new(pos: Vector3) -> Self {
        Self(pos)
    }

    /// Returns the underlying vector.
    #[inline]
    pub const fn vec(&self) -> Vector3 {
        self.0
    }
}

impl From<Vector3> for Origin {
    #[inline]
    fn from(v: Vector3) -> Self {
        Self(v)
    }
}

impl From<Origin> for Vector3 {
    #[inline]
    fn from(o: Origin) -> Self {
        o.0
    }
}

impl Deref for Origin {
    type Target = Vector3;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Origin {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Entity or player velocity vector (`Vector3`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Velocity(pub Vector3);

impl Velocity {
    /// Creates a new velocity vector wrapper.
    #[inline]
    pub const fn new(vel: Vector3) -> Self {
        Self(vel)
    }

    /// Returns the underlying vector.
    #[inline]
    pub const fn vec(&self) -> Vector3 {
        self.0
    }
}

impl From<Vector3> for Velocity {
    #[inline]
    fn from(v: Vector3) -> Self {
        Self(v)
    }
}

impl From<Velocity> for Vector3 {
    #[inline]
    fn from(vel: Velocity) -> Self {
        vel.0
    }
}

impl Deref for Velocity {
    type Target = Vector3;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Velocity {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Entity or player view angles (pitch, yaw, roll) (`Vector3`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Angles(pub Vector3);

impl Angles {
    /// Creates a new view angles wrapper.
    #[inline]
    pub const fn new(angles: Vector3) -> Self {
        Self(angles)
    }

    /// Returns the underlying vector.
    #[inline]
    pub const fn vec(&self) -> Vector3 {
        self.0
    }
}

impl From<Vector3> for Angles {
    #[inline]
    fn from(v: Vector3) -> Self {
        Self(v)
    }
}

impl From<Angles> for Vector3 {
    #[inline]
    fn from(a: Angles) -> Self {
        a.0
    }
}

impl Deref for Angles {
    type Target = Vector3;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Angles {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// --- Property System Integrations ---

use crate::Entity;
use crate::client::Player;
use crate::property::{PropGet, PropSet};

impl PropGet<Entity> for Origin {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let v = crate::bindings::goldsrc::engine::api::host_entity_origin(target.index);
            Self(Vector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(target.inner.origin().unwrap_or([0.0, 0.0, 0.0]).into())
        }
    }
}

impl PropSet<Entity> for Origin {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_set_origin(
                target.index,
                crate::bindings::goldsrc::engine::api::Vector3 {
                    x: self.0.x,
                    y: self.0.y,
                    z: self.0.z,
                },
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_origin(self.0.into());
        }
    }
}

impl PropGet<Player> for Origin {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropGet::<Entity>::get_from(target)
    }
}

impl PropSet<Player> for Origin {
    #[inline(always)]
    fn set_on(self, target: &mut Player) {
        PropSet::<Entity>::set_on(self, target);
    }
}

impl PropGet<Entity> for Velocity {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let v = crate::bindings::goldsrc::engine::api::host_entity_velocity(target.index);
            Self(Vector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(target.inner.velocity().unwrap_or([0.0, 0.0, 0.0]).into())
        }
    }
}

impl PropSet<Entity> for Velocity {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_set_velocity(
                target.index,
                crate::bindings::goldsrc::engine::api::Vector3 {
                    x: self.0.x,
                    y: self.0.y,
                    z: self.0.z,
                },
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_velocity(self.0.into());
        }
    }
}

impl PropGet<Player> for Velocity {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropGet::<Entity>::get_from(target)
    }
}

impl PropSet<Player> for Velocity {
    #[inline(always)]
    fn set_on(self, target: &mut Player) {
        PropSet::<Entity>::set_on(self, target);
    }
}

impl PropGet<Entity> for Angles {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let v = crate::bindings::goldsrc::engine::api::host_entity_angles(target.index);
            Self(Vector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(target.inner.angles().unwrap_or([0.0, 0.0, 0.0]).into())
        }
    }
}

impl PropSet<Entity> for Angles {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_set_angles(
                target.index,
                crate::bindings::goldsrc::engine::api::Vector3 {
                    x: self.0.x,
                    y: self.0.y,
                    z: self.0.z,
                },
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_angles(self.0.into());
        }
    }
}

impl PropGet<Player> for Angles {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropGet::<Entity>::get_from(target)
    }
}

impl PropSet<Player> for Angles {
    #[inline(always)]
    fn set_on(self, target: &mut Player) {
        PropSet::<Entity>::set_on(self, target);
    }
}
