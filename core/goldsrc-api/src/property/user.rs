//! Identity, localization, and authorization properties (Name, Lang, Classname, Capability).

use crate::Entity;
use crate::client::Player;
use crate::property::{MutProperty, Property};

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

impl Property<Entity> for Classname {
    type Value = Option<String>;

    #[inline(always)]
    fn get(&self, target: &Entity) -> Self::Value {
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

impl Property<Player> for Classname {
    type Value = Option<String>;

    #[inline(always)]
    fn get(&self, target: &Player) -> Self::Value {
        Property::<Entity>::get(self, target)
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
