//! Player identity and state properties (`Name`, `Lang`, `Team`, `LifeState`).

#[cfg(target_arch = "wasm32")]
use crate::bindings::goldsrc::engine::api as host_api;
use crate::client::{Client, LifeState, Player, Team};
use crate::property::{Health, PropGet};

/// Player display name (`Option<String>` / Read-Only).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Name(pub Option<String>);

impl Name {
    /// Creates a new `Name` wrapper.
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self(Some(name.into()))
    }

    /// Returns the name as a string slice, if present.
    #[inline]
    pub fn as_deref(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

impl From<Option<String>> for Name {
    #[inline]
    fn from(opt: Option<String>) -> Self {
        Self(opt)
    }
}

impl From<Name> for Option<String> {
    #[inline]
    fn from(n: Name) -> Self {
        n.0
    }
}

impl PropGet<Client> for Name {
    #[inline(always)]
    fn get_from(target: &Client) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self(host_api::host_player_name(target.index))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(lock) = crate::client::player::PLAYER_NAME_RESOLVER_HOOK.read()
                && let Some(resolver) = *lock
                && let Some(name) = resolver(target.index)
            {
                return Self(Some(name));
            }
            Self(target.inner.netname())
        }
    }
}

impl PropGet<Player> for Name {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        target.client().get::<Name>()
    }
}

/// Player language code (`String` / Read-Only).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Lang(pub String);

impl Lang {
    /// Creates a new `Lang` wrapper.
    #[inline]
    pub fn new(lang: impl Into<String>) -> Self {
        Self(lang.into())
    }

    /// Returns the language code as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Lang {
    #[inline]
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Lang {
    #[inline]
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::ops::Deref for Lang {
    type Target = str;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PropGet<Client> for Lang {
    #[inline(always)]
    fn get_from(target: &Client) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self(host_api::host_player_lang(target.index).unwrap_or_else(|| "en".to_string()))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(lock) = crate::client::player::PLAYER_LANG_RESOLVER_HOOK.read()
                && let Some(resolver) = *lock
                && let Some(lang) = resolver(target.index)
            {
                return Self(lang);
            }
            Self("en".to_string())
        }
    }
}

impl PropGet<Player> for Lang {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        target.client().get::<Lang>()
    }
}

impl PropGet<Client> for Team {
    #[inline(always)]
    fn get_from(target: &Client) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            crate::client::Team::from(host_api::host_player_team(target.index))
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

impl PropGet<Player> for Team {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        target.client().get::<Team>()
    }
}

impl PropGet<Player> for LifeState {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        if !target.is_valid() {
            return LifeState::Dead;
        }
        if target.get::<Health>().is_alive() {
            LifeState::Alive
        } else {
            LifeState::Dead
        }
    }
}
