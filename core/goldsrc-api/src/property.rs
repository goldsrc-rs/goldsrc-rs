//! Universal Entity and Player Property System (`PropertyGetter<Target>` & `PropertySetter<Target>`).
//!
//! Provides a symmetrical, type-safe CQS property querying (`get::<T>()`), mutation (`set(val)`),
//! and in-place updating (`modify::<T>(f)`) model with rich domain models (`Health`, `Armor`, `Origin`, `Velocity`, `Angles`).

use crate::Entity;
use crate::client::Player;
#[cfg(target_arch = "wasm32")]
use crate::types::Vector3;

pub use crate::auth::property::Capability;
pub use crate::client::property::{Lang, Name, PlayerLifeState, PlayerTeam};
pub use crate::types::spatial::{
    Angles, Angles as SpatialAngles, Origin, Origin as SpatialOrigin, Velocity,
    Velocity as SpatialVelocity,
};
pub use crate::types::vital::{Armor, Armor as VitalArmor, Health, Health as VitalHealth};

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

/// Idiomatic Rust alias for [`PropertyGetter`].
pub use PropertyGetter as PropGet;

/// Idiomatic Rust alias for [`PropertySetter`].
pub use PropertySetter as PropSet;

/// Idiomatic Rust alias for [`Property`].
pub use Property as Prop;

// --- Spatial Properties ---

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
