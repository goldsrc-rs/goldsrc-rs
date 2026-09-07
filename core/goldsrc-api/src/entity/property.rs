//! Entity identity and domain properties (`Classname`).

#[allow(unused_imports)]
use crate::bindings::goldsrc::engine::api as host_api;
use crate::client::Player;
use crate::entity::Entity;
use crate::property::PropGet;

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

impl PropGet<Entity> for Classname {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self(host_api::host_entity_classname(target.index))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(target.inner.classname())
        }
    }
}

impl PropGet<Player> for Classname {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropGet::<Entity>::get_from(target)
    }
}
