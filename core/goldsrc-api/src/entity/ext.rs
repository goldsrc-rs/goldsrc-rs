//! Extension trait providing spatial, physics, vital, and identity shortcuts on entities.

use crate::entity::Entity;
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
    fn health(&self) -> f32;
    /// Sets the entity's health.
    fn set_health(&mut self, health: f32);
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
        self.get(crate::property::Origin)
    }

    #[inline(always)]
    fn set_origin(&mut self, pos: Vector3) {
        self.set(crate::property::Origin, pos);
    }

    #[inline(always)]
    fn velocity(&self) -> Vector3 {
        self.get(crate::property::Velocity)
    }

    #[inline(always)]
    fn set_velocity(&mut self, vel: Vector3) {
        self.set(crate::property::Velocity, vel);
    }

    #[inline(always)]
    fn angles(&self) -> Vector3 {
        self.get(crate::property::Angles)
    }

    #[inline(always)]
    fn set_angles(&mut self, angles: Vector3) {
        self.set(crate::property::Angles, angles);
    }

    #[inline(always)]
    fn health(&self) -> f32 {
        self.get(crate::property::Health)
    }

    #[inline(always)]
    fn set_health(&mut self, health: f32) {
        self.set(crate::property::Health, health);
    }

    #[inline(always)]
    fn classname(&self) -> Option<String> {
        self.get(crate::property::Classname)
    }

    #[inline(always)]
    fn is_alive(&self) -> bool {
        self.health() > 0.0
    }

    #[inline(always)]
    fn is_valid(&self) -> bool {
        Entity::is_valid(self)
    }
}

impl EntityExt for crate::client::Player {
    #[inline(always)]
    fn origin(&self) -> Vector3 {
        self.get(crate::property::Origin)
    }

    #[inline(always)]
    fn set_origin(&mut self, pos: Vector3) {
        self.set(crate::property::Origin, pos);
    }

    #[inline(always)]
    fn velocity(&self) -> Vector3 {
        self.get(crate::property::Velocity)
    }

    #[inline(always)]
    fn set_velocity(&mut self, vel: Vector3) {
        self.set(crate::property::Velocity, vel);
    }

    #[inline(always)]
    fn angles(&self) -> Vector3 {
        self.get(crate::property::Angles)
    }

    #[inline(always)]
    fn set_angles(&mut self, angles: Vector3) {
        self.set(crate::property::Angles, angles);
    }

    #[inline(always)]
    fn health(&self) -> f32 {
        self.get(crate::property::Health)
    }

    #[inline(always)]
    fn set_health(&mut self, health: f32) {
        self.set(crate::property::Health, health);
    }

    #[inline(always)]
    fn classname(&self) -> Option<String> {
        self.get(crate::property::Classname)
    }

    #[inline(always)]
    fn is_alive(&self) -> bool {
        self.health() > 0.0
    }

    #[inline(always)]
    fn is_valid(&self) -> bool {
        crate::client::Player::is_valid(self)
    }
}
