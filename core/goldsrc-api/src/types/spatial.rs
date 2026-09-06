//! Spatial orientation and kinematic value objects (`Origin`, `Velocity`, `Angles`).

use crate::types::Vector3;

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

impl std::ops::Deref for Origin {
    type Target = Vector3;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Origin {
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

impl std::ops::Deref for Velocity {
    type Target = Vector3;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Velocity {
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

impl std::ops::Deref for Angles {
    type Target = Vector3;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Angles {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
