//! Vital state properties (Health, Armor, Team, LifeState).

use crate::Entity;
use crate::client::{LifeState, Player, Team};
use crate::property::{MutProperty, Property};

/// Player or entity health points (`f32`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Health;

impl Property<Entity> for Health {
    type Value = f32;

    #[inline(always)]
    fn get(&self, target: &Entity) -> Self::Value {
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

impl MutProperty<Entity> for Health {
    #[inline(always)]
    fn set(&self, target: &mut Entity, val: Self::Value) {
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

impl Property<Player> for Health {
    type Value = f32;

    #[inline(always)]
    fn get(&self, target: &Player) -> Self::Value {
        Property::<Entity>::get(self, target)
    }
}

impl MutProperty<Player> for Health {
    #[inline(always)]
    fn set(&self, target: &mut Player, val: Self::Value) {
        MutProperty::<Entity>::set(self, target, val);
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
            crate::bindings::goldsrc::engine::api::host_player_set_armorvalue(target.index, val);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_armorvalue(val);
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
        if target.get(Health) > 0.0 {
            LifeState::Alive
        } else {
            LifeState::Dead
        }
    }
}
