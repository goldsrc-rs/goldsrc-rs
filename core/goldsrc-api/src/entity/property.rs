//! Entity domain properties: Classname, Buttons, Flags, MaxSpeed, Gravity, Model, Render, Collision.

use crate::client::Player;
use crate::entity::Entity;
use crate::entity::types::{MoveType, RenderFx, RenderMode, SolidType};
use crate::property::{PropGet, PropSet};

#[cfg(target_arch = "wasm32")]
use crate::bindings::goldsrc::engine::api as host_api;

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

/// Player input buttons bitflags (`pev->button`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Buttons(pub i32);

impl Buttons {
    pub const IN_ATTACK: i32 = 1 << 0;
    pub const IN_JUMP: i32 = 1 << 1;
    pub const IN_DUCK: i32 = 1 << 2;
    pub const IN_FORWARD: i32 = 1 << 3;
    pub const IN_BACK: i32 = 1 << 4;
    pub const IN_USE: i32 = 1 << 5;
    pub const IN_CANCEL: i32 = 1 << 6;
    pub const IN_LEFT: i32 = 1 << 7;
    pub const IN_RIGHT: i32 = 1 << 8;
    pub const IN_MOVELEFT: i32 = 1 << 9;
    pub const IN_MOVERIGHT: i32 = 1 << 10;
    pub const IN_ATTACK2: i32 = 1 << 11;
    pub const IN_RUN: i32 = 1 << 12;
    pub const IN_RELOAD: i32 = 1 << 13;
    pub const IN_ALT1: i32 = 1 << 14;
    pub const IN_SCORE: i32 = 1 << 15;

    #[inline]
    pub const fn new(raw: i32) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn raw(self) -> i32 {
        self.0
    }

    #[inline]
    pub const fn is_down(self, mask: i32) -> bool {
        (self.0 & mask) == mask
    }

    #[inline]
    pub const fn attack(self) -> bool {
        self.is_down(Self::IN_ATTACK)
    }

    #[inline]
    pub const fn jump(self) -> bool {
        self.is_down(Self::IN_JUMP)
    }

    #[inline]
    pub const fn duck(self) -> bool {
        self.is_down(Self::IN_DUCK)
    }

    #[inline]
    pub const fn forward(self) -> bool {
        self.is_down(Self::IN_FORWARD)
    }

    #[inline]
    pub const fn back(self) -> bool {
        self.is_down(Self::IN_BACK)
    }

    #[inline]
    pub const fn use_key(self) -> bool {
        self.is_down(Self::IN_USE)
    }

    #[inline]
    pub const fn reload(self) -> bool {
        self.is_down(Self::IN_RELOAD)
    }
}

impl PropGet<Entity> for Buttons {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = target;
            Self(0)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(target.inner.button().unwrap_or(0))
        }
    }
}

impl PropSet<Entity> for Buttons {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_button(self.0);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (self, target);
        }
    }
}

impl PropGet<Player> for Buttons {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropGet::<Entity>::get_from(target)
    }
}

impl PropSet<Player> for Buttons {
    #[inline(always)]
    fn set_on(self, target: &mut Player) {
        PropSet::<Entity>::set_on(self, target);
    }
}

/// Entity flags bitmask (`pev->flags`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags(pub i32);

impl Flags {
    pub const FL_FLY: i32 = 1 << 0;
    pub const FL_SWIM: i32 = 1 << 1;
    pub const FL_CONVEYOR: i32 = 1 << 2;
    pub const FL_CLIENT: i32 = 1 << 3;
    pub const FL_INWATER: i32 = 1 << 4;
    pub const FL_MONSTER: i32 = 1 << 5;
    pub const FL_GODMODE: i32 = 1 << 6;
    pub const FL_NOTARGET: i32 = 1 << 7;
    pub const FL_ITEM: i32 = 1 << 8;
    pub const FL_ONGROUND: i32 = 1 << 9;
    pub const FL_PARTIALGROUND: i32 = 1 << 10;
    pub const FL_WATERJUMP: i32 = 1 << 11;
    pub const FL_FROZEN: i32 = 1 << 12;
    pub const FL_FAKECLIENT: i32 = 1 << 13;
    pub const FL_DUCKING: i32 = 1 << 14;
    pub const FL_FLOAT: i32 = 1 << 15;
    pub const FL_PROXY: i32 = 1 << 20;

    #[inline]
    pub const fn new(raw: i32) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn raw(self) -> i32 {
        self.0
    }

    #[inline]
    pub const fn has(self, flag: i32) -> bool {
        (self.0 & flag) != 0
    }

    #[inline]
    pub const fn is_on_ground(self) -> bool {
        self.has(Self::FL_ONGROUND)
    }

    #[inline]
    pub const fn is_ducking(self) -> bool {
        self.has(Self::FL_DUCKING)
    }

    #[inline]
    pub const fn is_client(self) -> bool {
        self.has(Self::FL_CLIENT)
    }

    #[inline]
    pub const fn is_bot(self) -> bool {
        self.has(Self::FL_FAKECLIENT)
    }
}

impl PropGet<Entity> for Flags {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = target;
            Self(0)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(target.inner.flags().unwrap_or(0))
        }
    }
}

impl PropSet<Entity> for Flags {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_flags(self.0);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (self, target);
        }
    }
}

impl PropGet<Player> for Flags {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropGet::<Entity>::get_from(target)
    }
}

impl PropSet<Player> for Flags {
    #[inline(always)]
    fn set_on(self, target: &mut Player) {
        PropSet::<Entity>::set_on(self, target);
    }
}

/// Entity maximum speed (`pev->maxspeed`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MaxSpeed(pub f32);

impl PropGet<Entity> for MaxSpeed {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = target;
            Self(250.0)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(target.inner.maxspeed().unwrap_or(250.0))
        }
    }
}

impl PropSet<Entity> for MaxSpeed {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_maxspeed(self.0);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (self, target);
        }
    }
}

impl PropGet<Player> for MaxSpeed {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropGet::<Entity>::get_from(target)
    }
}

impl PropSet<Player> for MaxSpeed {
    #[inline(always)]
    fn set_on(self, target: &mut Player) {
        PropSet::<Entity>::set_on(self, target);
    }
}

/// Entity gravity scale multiplier (`pev->gravity`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Gravity(pub f32);

impl PropGet<Entity> for Gravity {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = target;
            Self(1.0)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(target.inner.gravity().unwrap_or(1.0))
        }
    }
}

impl PropSet<Entity> for Gravity {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_gravity(self.0);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (self, target);
        }
    }
}

impl PropGet<Player> for Gravity {
    #[inline(always)]
    fn get_from(target: &Player) -> Self {
        PropGet::<Entity>::get_from(target)
    }
}

impl PropSet<Player> for Gravity {
    #[inline(always)]
    fn set_on(self, target: &mut Player) {
        PropSet::<Entity>::set_on(self, target);
    }
}

/// Entity rendering mode property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RenderModeProp(pub RenderMode);

impl PropGet<Entity> for RenderModeProp {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = target;
            Self(RenderMode::Normal)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(RenderMode::from_raw(target.inner.rendermode().unwrap_or(0)))
        }
    }
}

impl PropSet<Entity> for RenderModeProp {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_rendermode(self.0.as_raw());
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (self, target);
        }
    }
}

/// Entity rendering amount / opacity (`pev->renderamt`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RenderAmt(pub f32);

impl PropGet<Entity> for RenderAmt {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = target;
            Self(255.0)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(target.inner.renderamt().unwrap_or(255.0))
        }
    }
}

impl PropSet<Entity> for RenderAmt {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_renderamt(self.0);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (self, target);
        }
    }
}

/// Entity rendering color (`pev->rendercolor` [r, g, b]).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RenderColor(pub [f32; 3]);

impl PropGet<Entity> for RenderColor {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = target;
            Self([255.0, 255.0, 255.0])
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(target.inner.rendercolor().unwrap_or([255.0, 255.0, 255.0]))
        }
    }
}

impl PropSet<Entity> for RenderColor {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_rendercolor(self.0);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (self, target);
        }
    }
}

/// Entity rendering special effect property (`pev->renderfx`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RenderFxProp(pub RenderFx);

impl PropGet<Entity> for RenderFxProp {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = target;
            Self(RenderFx::None)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(RenderFx::from_raw(target.inner.renderfx().unwrap_or(0)))
        }
    }
}

impl PropSet<Entity> for RenderFxProp {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_renderfx(self.0.as_raw());
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (self, target);
        }
    }
}

/// Entity collision solidity property (`pev->solid`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SolidProp(pub SolidType);

impl PropGet<Entity> for SolidProp {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = target;
            Self(SolidType::Not)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(SolidType::from_raw(target.inner.solid().unwrap_or(0)))
        }
    }
}

impl PropSet<Entity> for SolidProp {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_solid(self.0.as_raw());
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (self, target);
        }
    }
}

/// Entity physics movement type property (`pev->movetype`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MoveTypeProp(pub MoveType);

impl PropGet<Entity> for MoveTypeProp {
    #[inline(always)]
    fn get_from(target: &Entity) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = target;
            Self(MoveType::None)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(MoveType::from_raw(target.inner.movetype().unwrap_or(0)))
        }
    }
}

impl PropSet<Entity> for MoveTypeProp {
    #[inline(always)]
    fn set_on(self, target: &mut Entity) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            target.inner.set_movetype(self.0.as_raw());
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (self, target);
        }
    }
}
