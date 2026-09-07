//! Universal Entity and Player Property System (`PropGet<Target>` & `PropSet<Target>`).
//!
//! Provides a symmetrical, type-safe CQS property querying (`get::<T>()`), mutation (`set(val)`),
//! and in-place updating (`modify::<T>(f)`) model with rich domain models (`Health`, `Armor`, `Origin`, `Velocity`, `Angles`).

pub use crate::auth::property::Capability;
pub use crate::client::property::{Lang, Name};
pub use crate::entity::Classname;
pub use crate::types::spatial::{Angles, Origin, Velocity};
pub use crate::types::vital::{Armor, Health};

/// Trait for querying a property/component of type `Self` from `Target`.
pub trait PropGet<Target> {
    /// Reads and constructs the property value from the given target.
    fn get_from(target: &Target) -> Self;
}

/// Trait for setting/mutating a property of type `Self` on `Target`.
pub trait PropSet<Target> {
    /// Applies this property value to the target.
    fn set_on(self, target: &mut Target);
}

/// Blanket trait for properties that support both reading and writing on `Target`.
pub trait Prop<Target>: PropGet<Target> + PropSet<Target> {}

impl<T, Target> Prop<Target> for T where T: PropGet<Target> + PropSet<Target> {}

/// Standard engine properties namespace (`crate::property::prop::*`).
pub mod prop {
    pub use super::{Angles, Armor, Capability, Classname, Health, Lang, Name, Origin, Velocity};
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
