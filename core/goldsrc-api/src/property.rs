//! Universal Entity and Player Property System (`Property<Target>` & `MutProperty<Target>`).
//!
//! Provides a type-safe, extensible property querying (`get::<P>()`) and mutation (`set::<P>(val)`)
//! model with zero runtime cost (ZST markers) and compile-time protection for read-only attributes.

use crate::Vector3;
use crate::client::{LifeState, Player, Team};

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

/// Standard engine properties for players and entities.
pub mod prop {
    use super::*;

    /// Player or entity health points (`f32`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Health;

    impl Property<Player> for Health {
        type Value = f32;

        #[inline(always)]
        fn get(&self, target: &Player) -> Self::Value {
            #[cfg(target_arch = "wasm32")]
            {
                crate::bindings::goldsrc::engine::api::host_entity_health(target.index)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                target.inner.health().unwrap_or(0.0)
            }
        }
    }

    impl MutProperty<Player> for Health {
        #[inline(always)]
        fn set(&self, target: &mut Player, val: Self::Value) {
            if !val.is_finite() {
                return;
            }
            #[cfg(target_arch = "wasm32")]
            {
                crate::bindings::goldsrc::engine::api::host_entity_set_health(target.index, val);
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                target.inner.set_health(val);
            }
        }
    }

    /// Player armor value (`f32`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Armor;

    impl Property<Player> for Armor {
        type Value = f32;

        #[inline(always)]
        fn get(&self, target: &Player) -> Self::Value {
            #[cfg(target_arch = "wasm32")]
            {
                crate::bindings::goldsrc::engine::api::host_player_armorvalue(target.index)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                target.inner.armorvalue().unwrap_or(0.0)
            }
        }
    }

    impl MutProperty<Player> for Armor {
        #[inline(always)]
        fn set(&self, target: &mut Player, val: Self::Value) {
            if !val.is_finite() {
                return;
            }
            #[cfg(target_arch = "wasm32")]
            {
                crate::bindings::goldsrc::engine::api::host_player_set_armorvalue(
                    target.index,
                    val,
                );
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                target.inner.set_armorvalue(val);
            }
        }
    }

    /// Player 3D world origin coordinates (`Vector3`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Origin;

    impl Property<Player> for Origin {
        type Value = Vector3;

        #[inline(always)]
        fn get(&self, target: &Player) -> Self::Value {
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

    impl MutProperty<Player> for Origin {
        #[inline(always)]
        fn set(&self, target: &mut Player, val: Self::Value) {
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

    /// Player velocity vector (`Vector3`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Velocity;

    impl Property<Player> for Velocity {
        type Value = Vector3;

        #[inline(always)]
        fn get(&self, target: &Player) -> Self::Value {
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

    impl MutProperty<Player> for Velocity {
        #[inline(always)]
        fn set(&self, target: &mut Player, val: Self::Value) {
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

    /// Player view angles (pitch, yaw, roll) (`Vector3`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Angles;

    impl Property<Player> for Angles {
        type Value = Vector3;

        #[inline(always)]
        fn get(&self, target: &Player) -> Self::Value {
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

    impl MutProperty<Player> for Angles {
        #[inline(always)]
        fn set(&self, target: &mut Player, val: Self::Value) {
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

    /// Player current team (`Team` / Read-Only).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct PlayerTeam;

    impl Property<Player> for PlayerTeam {
        type Value = Team;

        #[inline(always)]
        fn get(&self, target: &Player) -> Self::Value {
            #[cfg(target_arch = "wasm32")]
            {
                crate::client::Team::from(crate::bindings::goldsrc::engine::api::host_player_team(
                    target.index,
                ))
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Ok(lock) = crate::client::player::PLAYER_TEAM_RESOLVER_HOOK.read()
                    && let Some(resolver) = *lock
                {
                    return resolver(target.index).into();
                }
                target.inner.team().unwrap_or(0).into()
            }
        }
    }

    /// Player life state (`LifeState` / Read-Only).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct PlayerLifeState;

    impl Property<Player> for PlayerLifeState {
        type Value = LifeState;

        #[inline(always)]
        fn get(&self, target: &Player) -> Self::Value {
            if !target.is_valid() {
                return LifeState::Dead;
            }
            if target.get::<Health>() > 0.0 {
                LifeState::Alive
            } else {
                LifeState::Dead
            }
        }
    }

    /// Player display name (`Option<String>` / Read-Only).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Name;

    impl Property<Player> for Name {
        type Value = Option<String>;

        #[inline(always)]
        fn get(&self, target: &Player) -> Self::Value {
            #[cfg(target_arch = "wasm32")]
            {
                crate::bindings::goldsrc::engine::api::host_player_name(target.index)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Ok(lock) = crate::client::player::PLAYER_NAME_RESOLVER_HOOK.read()
                    && let Some(resolver) = *lock
                    && let Some(name) = resolver(target.index)
                {
                    return Some(name);
                }
                target.inner.netname()
            }
        }
    }

    /// Player language code (`String` / Read-Only).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Lang;

    impl Property<Player> for Lang {
        type Value = String;

        #[inline(always)]
        fn get(&self, target: &Player) -> Self::Value {
            #[cfg(target_arch = "wasm32")]
            {
                crate::bindings::goldsrc::engine::api::host_player_lang(target.index)
                    .unwrap_or_else(|| "en".to_string())
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Ok(lock) = crate::client::player::PLAYER_LANG_RESOLVER_HOOK.read()
                    && let Some(resolver) = *lock
                    && let Some(lang) = resolver(target.index)
                {
                    return lang;
                }
                "en".to_string()
            }
        }
    }

    /// Entity class name (`Option<String>` / Read-Only).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Classname;

    impl Property<Player> for Classname {
        type Value = Option<String>;

        #[inline(always)]
        fn get(&self, target: &Player) -> Self::Value {
            #[cfg(target_arch = "wasm32")]
            {
                crate::bindings::goldsrc::engine::api::host_entity_classname(target.index)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                target.inner.classname()
            }
        }
    }

    /// Dynamic player authorization capability flag (`bool` / Read-Write).
    ///
    /// Evaluates or modifies capabilities via `Auth::has_capability`,
    /// `Auth::grant_capability` and `Auth::revoke_capability`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Capability<'a>(pub &'a str);

    impl<'a> Property<Player> for Capability<'a> {
        type Value = bool;

        #[inline(always)]
        fn get(&self, target: &Player) -> Self::Value {
            crate::auth::Auth::has_capability(target.index, self.0)
        }
    }

    impl<'a> MutProperty<Player> for Capability<'a> {
        #[inline(always)]
        fn set(&self, target: &mut Player, val: Self::Value) {
            if val {
                crate::auth::Auth::grant_capability(target.index, self.0);
            } else {
                crate::auth::Auth::revoke_capability(target.index, self.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_zst_markers() {
        let player = Player::new(1);
        let _health: f32 = player.get::<prop::Health>();
        let _armor: f32 = player.get::<prop::Armor>();
        let _origin: Vector3 = player.get::<prop::Origin>();
        let _velocity: Vector3 = player.get::<prop::Velocity>();
        let _angles: Vector3 = player.get::<prop::Angles>();
        let _team: Team = player.get::<prop::PlayerTeam>();
        let _life_state: LifeState = player.get::<prop::PlayerLifeState>();
    }

    #[test]
    fn test_parameterized_property_capability() {
        crate::auth::Auth::register_capability("prop.test.unique_ban", "ban permission");
        let mut player = Player::new(78);
        crate::auth::Auth::remove_player(78);
        assert!(!player.get_prop(prop::Capability("prop.test.unique_ban")));

        player.set_prop(prop::Capability("prop.test.unique_ban"), true);
        assert!(player.get_prop(prop::Capability("prop.test.unique_ban")));

        player.set_prop(prop::Capability("prop.test.unique_ban"), false);
        assert!(!player.get_prop(prop::Capability("prop.test.unique_ban")));
    }
}
