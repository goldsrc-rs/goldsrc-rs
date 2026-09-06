//! Universal Entity and Player Property System (`Property<Target>` & `MutProperty<Target>`).
//!
//! Provides a type-safe, extensible property querying (`get(prop)`) and mutation (`set(prop, val)`)
//! model with zero runtime cost (ZST markers) and compile-time protection for read-only attributes.

pub mod spatial;
pub mod state;
pub mod user;

pub use spatial::{Angles, Origin, Velocity};
pub use state::{Armor, Health, PlayerLifeState, PlayerTeam};
pub use user::{Capability, Classname, Lang, Name};

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

/// Standard engine properties namespace for backward compatibility (`crate::property::prop::*`).
pub mod prop {
    pub use super::spatial::*;
    pub use super::state::*;
    pub use super::user::*;
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
