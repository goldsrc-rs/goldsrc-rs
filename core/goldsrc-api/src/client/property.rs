//! Player identity and state properties (`Name`, `Lang`, `PlayerTeam`, `PlayerLifeState`).

use crate::client::{LifeState, Player, Team};
use crate::property::Property;

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
        if target.get(crate::property::Health) > 0.0 {
            LifeState::Alive
        } else {
            LifeState::Dead
        }
    }
}
