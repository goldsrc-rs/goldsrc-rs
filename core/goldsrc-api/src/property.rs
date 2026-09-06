//! Universal Entity and Player Property System (`PropertyGetter<Target>` & `PropertySetter<Target>`).
//!
//! Provides a symmetrical, type-safe CQS property querying (`get::<T>()`), mutation (`set(val)`),
//! and in-place updating (`modify::<T>(f)`) model with rich domain models (`Health`, `Armor`, `Origin`, `Velocity`, `Angles`).

use crate::client::Player;
use crate::{Entity, Vector3};

pub use crate::auth::property::Capability;
pub use crate::client::property::{Lang, Name, PlayerLifeState, PlayerTeam};

/// Trait for querying a property/component of type `Self` from `Target`.
pub trait PropertyGetter<Target> {
    /// Reads and constructs the property value from the given target.
    fn get_from(target: &Target) -> Self;
}

/// Trait for setting/mutating a property of type `Self` on `Target`.
pub trait PropertySetter<Target> {
    /// Applies this property value to the target.
    fn set_on(self, target: &mut Target);
}

/// Blanket trait for properties that support both reading and writing on `Target`.
pub trait Property<Target>: PropertyGetter<Target> + PropertySetter<Target> {}

impl<T, Target> Property<Target> for T where T: PropertyGetter<Target> + PropertySetter<Target> {}

// --- Spatial Properties ---

/// Entity or player 3D world origin coordinates (`Vector3`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Origin(pub Vector3);

impl Origin {
    /// Creates a new origin coordinate wrapper.
    #[inline]
    pub const fn new(pos: Vector3) -> Self {
        Self(pos)
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

impl PropertyGetter<Entity> for Origin {
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

impl PropertySetter<Entity> for Origin {
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

impl PropertyGetter<Player> for Origin {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropertyGetter::<Entity>::get_from(target)
    }
}

impl PropertySetter<Player> for Origin {
    #[inline(always)]
    fn set_on(self, target: &mut Player) {
        PropertySetter::<Entity>::set_on(self, target);
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

impl PropertyGetter<Entity> for Velocity {
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

impl PropertySetter<Entity> for Velocity {
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

impl PropertyGetter<Player> for Velocity {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropertyGetter::<Entity>::get_from(target)
    }
}

impl PropertySetter<Player> for Velocity {
    #[inline(always)]
    fn set_on(self, target: &mut Player) {
        PropertySetter::<Entity>::set_on(self, target);
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

impl PropertyGetter<Entity> for Angles {
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

impl PropertySetter<Entity> for Angles {
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

impl PropertyGetter<Player> for Angles {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropertyGetter::<Entity>::get_from(target)
    }
}

impl PropertySetter<Player> for Angles {
    #[inline(always)]
    fn set_on(self, target: &mut Player) {
        PropertySetter::<Entity>::set_on(self, target);
    }
}

// --- Vital State Properties ---

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

impl PropertyGetter<Entity> for Health {
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

impl PropertySetter<Entity> for Health {
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

impl PropertyGetter<Player> for Health {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropertyGetter::<Entity>::get_from(target)
    }
}

impl PropertySetter<Player> for Health {
    #[inline(always)]
    fn set_on(self, target: &mut Player) {
        PropertySetter::<Entity>::set_on(self, target);
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

impl PropertyGetter<Player> for Armor {
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

impl PropertySetter<Player> for Armor {
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

// --- Identity Properties ---

/// Entity class name (`Option<String>` / Read-Only).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Classname(pub Option<String>);

impl Classname {
    /// Creates a new classname wrapper.
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self(Some(name.into()))
    }

    /// Returns the classname as a string slice, if present.
    #[inline]
    pub fn as_deref(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

impl From<Option<String>> for Classname {
    #[inline]
    fn from(opt: Option<String>) -> Self {
        Self(opt)
    }
}

impl From<Classname> for Option<String> {
    #[inline]
    fn from(c: Classname) -> Self {
        c.0
    }
}

impl PropertyGetter<Entity> for Classname {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self(crate::bindings::goldsrc::engine::api::host_entity_classname(target.index))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(target.inner.classname())
        }
    }
}

impl PropertyGetter<Player> for Classname {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropertyGetter::<Entity>::get_from(target)
    }
}

/// Standard engine properties namespace (`crate::property::prop::*`).
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
    fn test_rich_health_logic() {
        let mut hp = Health::full(100.0);
        assert!(hp.is_alive());
        assert!(!hp.is_dead());
        assert_eq!(hp.percentage(), 100.0);

        hp.damage(85.0);
        assert!((hp.current() - 15.0).abs() < 1e-4);
        assert!(hp.is_critical());
        assert!((hp.percentage() - 15.0).abs() < 1e-4);

        hp.heal(10.0);
        assert!((hp.current() - 25.0).abs() < 1e-4);
        assert!(!hp.is_critical());

        hp.damage(50.0);
        assert_eq!(hp.current(), 0.0);
        assert!(hp.is_dead());
    }

    #[test]
    fn test_rich_armor_logic() {
        let mut armor = Armor::new(100.0);
        assert_eq!(armor.value(), 100.0);
        assert!(!armor.is_broken());

        armor.reduce(60.0);
        assert_eq!(armor.value(), 40.0);

        armor.reduce(50.0);
        assert_eq!(armor.value(), 0.0);
        assert!(armor.is_broken());
    }

    #[test]
    fn test_cqs_get_and_set() {
        let mut player = Player::new(1);
        let _health: Health = player.get();
        let _armor: Armor = player.get();
        let _origin: Origin = player.get();
        let _velocity: Velocity = player.get();
        let _angles: Angles = player.get();
        let _team: Team = player.get();
        let _life_state: LifeState = player.get();

        player.set(Health::full(100.0));
        player.set(Armor::new(100.0));
        player.set(Origin(Vector3::new(1.0, 2.0, 3.0)));

        player.modify::<Health>(|hp| hp.damage(10.0));
        player.modify::<Armor>(|ar| ar.reduce(10.0));
    }
}
