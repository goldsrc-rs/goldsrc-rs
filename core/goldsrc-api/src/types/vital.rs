//! Vital combat and resilience value objects (`Health`, `Armor`).

/// Player or entity health points with max health, percentage, and combat arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Health {
    /// Current health points.
    pub current: f32,
    /// Maximum health points cap.
    pub max: f32,
}

impl Health {
    /// Creates a new `Health` value with current and max values.
    #[inline]
    pub const fn new(current: f32, max: f32) -> Self {
        Self { current, max }
    }

    /// Creates a full `Health` instance where `current == max`.
    #[inline]
    pub const fn full(max: f32) -> Self {
        Self { current: max, max }
    }

    /// Creates a `Health` instance with given current value and default 100.0 max.
    #[inline]
    pub const fn current_only(current: f32) -> Self {
        Self {
            current,
            max: 100.0,
        }
    }

    /// Returns `true` if current health is greater than 0.
    #[inline]
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    /// Returns `true` if current health is 0 or less.
    #[inline]
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    /// Returns current health percentage in range `0.0..=100.0`.
    #[inline]
    pub fn percentage(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.current / self.max * 100.0).clamp(0.0, 100.0)
        }
    }

    /// Returns `true` if health is in critical state (`0 < current <= 20.0`).
    #[inline]
    pub fn is_critical(&self) -> bool {
        self.current > 0.0 && self.current <= 20.0
    }

    /// Heals by adding `amount` up to `max`.
    #[inline]
    pub fn heal(&mut self, amount: f32) {
        if amount > 0.0 {
            self.current = (self.current + amount).min(self.max);
        }
    }

    /// Damages by subtracting `amount` down to `0.0`.
    #[inline]
    pub fn damage(&mut self, amount: f32) {
        if amount > 0.0 {
            self.current = (self.current - amount).max(0.0);
        }
    }

    /// Returns the current health points.
    #[inline]
    pub const fn current(&self) -> f32 {
        self.current
    }

    /// Returns the maximum health points.
    #[inline]
    pub const fn max(&self) -> f32 {
        self.max
    }
}

impl Default for Health {
    #[inline]
    fn default() -> Self {
        Self::full(100.0)
    }
}

impl From<f32> for Health {
    #[inline]
    fn from(val: f32) -> Self {
        Self::current_only(val)
    }
}

impl From<Health> for f32 {
    #[inline]
    fn from(h: Health) -> Self {
        h.current
    }
}

impl std::ops::Deref for Health {
    type Target = f32;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.current
    }
}

impl PartialEq<f32> for Health {
    #[inline]
    fn eq(&self, other: &f32) -> bool {
        self.current == *other
    }
}

impl PartialOrd<f32> for Health {
    #[inline]
    fn partial_cmp(&self, other: &f32) -> Option<std::cmp::Ordering> {
        self.current.partial_cmp(other)
    }
}

impl PartialEq<Health> for f32 {
    #[inline]
    fn eq(&self, other: &Health) -> bool {
        *self == other.current
    }
}

impl PartialOrd<Health> for f32 {
    #[inline]
    fn partial_cmp(&self, other: &Health) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.current)
    }
}

impl std::ops::Add<f32> for Health {
    type Output = Self;
    #[inline]
    fn add(self, rhs: f32) -> Self::Output {
        Self::new(self.current + rhs, self.max)
    }
}

impl std::ops::Sub<f32> for Health {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: f32) -> Self::Output {
        Self::new(self.current - rhs, self.max)
    }
}

impl std::ops::AddAssign<f32> for Health {
    #[inline]
    fn add_assign(&mut self, rhs: f32) {
        self.heal(rhs);
    }
}

impl std::ops::SubAssign<f32> for Health {
    #[inline]
    fn sub_assign(&mut self, rhs: f32) {
        self.damage(rhs);
    }
}

/// Player armor points (`armorvalue`).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Armor(pub f32);

impl Armor {
    /// Creates a new `Armor` value with a non-negative floor.
    #[inline]
    pub const fn new(val: f32) -> Self {
        Self(if val < 0.0 { 0.0 } else { val })
    }

    /// Returns the armor point value.
    #[inline]
    pub const fn value(&self) -> f32 {
        self.0
    }

    /// Returns `true` if armor is depleted (`<= 0.0`).
    #[inline]
    pub fn is_broken(&self) -> bool {
        self.0 <= 0.0
    }

    /// Repairs or increases armor by `amount`.
    #[inline]
    pub fn add(&mut self, amount: f32) {
        if amount > 0.0 {
            self.0 += amount;
        }
    }

    /// Reduces armor by `amount` down to `0.0`.
    #[inline]
    pub fn reduce(&mut self, amount: f32) {
        if amount > 0.0 {
            self.0 = (self.0 - amount).max(0.0);
        }
    }
}

impl From<f32> for Armor {
    #[inline]
    fn from(val: f32) -> Self {
        Self::new(val)
    }
}

impl From<Armor> for f32 {
    #[inline]
    fn from(a: Armor) -> Self {
        a.0
    }
}

impl std::ops::Deref for Armor {
    type Target = f32;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<f32> for Armor {
    #[inline]
    fn eq(&self, other: &f32) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<f32> for Armor {
    #[inline]
    fn partial_cmp(&self, other: &f32) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<Armor> for f32 {
    #[inline]
    fn eq(&self, other: &Armor) -> bool {
        *self == other.0
    }
}

impl PartialOrd<Armor> for f32 {
    #[inline]
    fn partial_cmp(&self, other: &Armor) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl std::ops::Add<f32> for Armor {
    type Output = Self;
    #[inline]
    fn add(self, rhs: f32) -> Self::Output {
        Self::new(self.0 + rhs)
    }
}

impl std::ops::Sub<f32> for Armor {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: f32) -> Self::Output {
        Self::new(self.0 - rhs)
    }
}

// --- Property System Integrations ---

use crate::Entity;
use crate::client::Player;
use crate::property::{PropGet, PropSet};

impl PropGet<Entity> for Health {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let cur = crate::bindings::goldsrc::engine::api::host_entity_health(target.index);
            Self::new(cur, 100.0)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let cur = target.inner.health().unwrap_or(0.0);
            Self::new(cur, 100.0)
        }
    }
}

impl PropSet<Entity> for Health {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        if !self.current.is_finite() {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_entity_set_health(
                target.index,
                self.current,
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_health(self.current);
        }
    }
}

impl PropGet<Player> for Health {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropGet::<Entity>::get_from(target)
    }
}

impl PropSet<Player> for Health {
    #[inline(always)]
    fn set_on(self, target: &mut Player) {
        PropSet::<Entity>::set_on(self, target);
    }
}

impl PropGet<Player> for Armor {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::new(crate::bindings::goldsrc::engine::api::host_player_armorvalue(target.index))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::new(target.inner.armorvalue().unwrap_or(0.0))
        }
    }
}

impl PropSet<Player> for Armor {
    #[inline(always)]
    fn set_on(self, target: &mut Player) {
        if !self.0.is_finite() {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_player_set_armorvalue(target.index, self.0);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_armorvalue(self.0);
        }
    }
}
