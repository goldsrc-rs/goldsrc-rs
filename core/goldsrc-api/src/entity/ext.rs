//! Extension trait providing spatial, physics, vital, and identity shortcuts on entities.

use crate::client::{Client, Player};
use crate::entity::Entity;
use crate::property::{Angles, Classname, Health, Origin, Velocity};
use crate::types::Vector3;

/// Extension trait providing spatial, physics, vital, and identity queries on entities.
pub trait EntityExt {
    /// Returns the entity's 3D world origin.
    fn origin(&self) -> Vector3;
    /// Sets the entity's 3D world origin.
    fn set_origin(&mut self, pos: Vector3);
    /// Returns the entity's velocity vector.
    fn velocity(&self) -> Vector3;
    /// Sets the entity's velocity vector.
    fn set_velocity(&mut self, vel: Vector3);
    /// Returns the entity's rotation angles (pitch, yaw, roll).
    fn angles(&self) -> Vector3;
    /// Sets the entity's rotation angles.
    fn set_angles(&mut self, angles: Vector3);
    /// Returns the entity's current health.
    fn health(&self) -> Health;
    /// Sets the entity's health.
    fn set_health(&mut self, health: impl Into<Health>);
    /// Returns the entity's class name, if set.
    fn classname(&self) -> Option<String>;
    /// Returns `true` if the entity is alive (`health > 0.0`).
    fn is_alive(&self) -> bool;
    /// Returns `true` if the entity slot is currently valid.
    fn is_valid(&self) -> bool;
}

impl EntityExt for Entity {
    #[inline(always)]
    fn origin(&self) -> Vector3 {
        self.get::<Origin>().0
    }

    #[inline(always)]
    fn set_origin(&mut self, pos: Vector3) {
        self.set(Origin(pos));
    }

    #[inline(always)]
    fn velocity(&self) -> Vector3 {
        self.get::<Velocity>().0
    }

    #[inline(always)]
    fn set_velocity(&mut self, vel: Vector3) {
        self.set(Velocity(vel));
    }

    #[inline(always)]
    fn angles(&self) -> Vector3 {
        self.get::<Angles>().0
    }

    #[inline(always)]
    fn set_angles(&mut self, angles: Vector3) {
        self.set(Angles(angles));
    }

    #[inline(always)]
    fn health(&self) -> Health {
        self.get::<Health>()
    }

    #[inline(always)]
    fn set_health(&mut self, health: impl Into<Health>) {
        self.set(health.into());
    }

    #[inline(always)]
    fn classname(&self) -> Option<String> {
        self.get::<Classname>().0
    }

    #[inline(always)]
    fn is_alive(&self) -> bool {
        self.health().is_alive()
    }

    #[inline(always)]
    fn is_valid(&self) -> bool {
        Entity::is_valid(self)
    }
}

impl EntityExt for Client {
    #[inline(always)]
    fn origin(&self) -> Vector3 {
        (**self).origin()
    }

    #[inline(always)]
    fn set_origin(&mut self, pos: Vector3) {
        (**self).set_origin(pos);
    }

    #[inline(always)]
    fn velocity(&self) -> Vector3 {
        (**self).velocity()
    }

    #[inline(always)]
    fn set_velocity(&mut self, vel: Vector3) {
        (**self).set_velocity(vel);
    }

    #[inline(always)]
    fn angles(&self) -> Vector3 {
        (**self).angles()
    }

    #[inline(always)]
    fn set_angles(&mut self, angles: Vector3) {
        (**self).set_angles(angles);
    }

    #[inline(always)]
    fn health(&self) -> Health {
        (**self).health()
    }

    #[inline(always)]
    fn set_health(&mut self, health: impl Into<Health>) {
        (**self).set_health(health);
    }

    #[inline(always)]
    fn classname(&self) -> Option<String> {
        (**self).classname()
    }

    #[inline(always)]
    fn is_alive(&self) -> bool {
        self.health().is_alive()
    }

    #[inline(always)]
    fn is_valid(&self) -> bool {
        crate::client::Client::is_valid(self)
    }
}

impl EntityExt for Player {
    #[inline(always)]
    fn origin(&self) -> Vector3 {
        self.get::<Origin>().0
    }

    #[inline(always)]
    fn set_origin(&mut self, pos: Vector3) {
        self.set(Origin(pos));
    }

    #[inline(always)]
    fn velocity(&self) -> Vector3 {
        self.get::<Velocity>().0
    }

    #[inline(always)]
    fn set_velocity(&mut self, vel: Vector3) {
        self.set(Velocity(vel));
    }

    #[inline(always)]
    fn angles(&self) -> Vector3 {
        self.get::<Angles>().0
    }

    #[inline(always)]
    fn set_angles(&mut self, angles: Vector3) {
        self.set(Angles(angles));
    }

    #[inline(always)]
    fn health(&self) -> Health {
        self.get::<Health>()
    }

    #[inline(always)]
    fn set_health(&mut self, health: impl Into<Health>) {
        self.set(health.into());
    }

    #[inline(always)]
    fn classname(&self) -> Option<String> {
        self.get::<Classname>().0
    }

    #[inline(always)]
    fn is_alive(&self) -> bool {
        self.health().is_alive()
    }

    #[inline(always)]
    fn is_valid(&self) -> bool {
        crate::client::Player::is_valid(self)
    }
}
