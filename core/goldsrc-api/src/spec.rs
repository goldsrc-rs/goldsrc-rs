//! Compile-Time Specifications, Zero-Sized Typestates, and Refinement Guards.
//!
//! Provides type-level preconditions and compound specifications ([`All`], [`Any`], [`NoneOf`])
//! evaluated at function boundaries, guaranteeing entity state validity at compile time via [`Refined`].

use std::fmt;
use std::marker::PhantomData;

use crate::Entity;
use crate::client::{ClientExt, LifeState, Player, PlayerExt};
use crate::command::FromArg;
use crate::entity::EntityExt;

/// Standard error returned when a [`Spec`] check fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecError {
    /// Target is not alive.
    NotAlive,
    /// Target is not dead.
    NotDead,
    /// Client slot is not connected or valid.
    NotConnected,
    /// Client is not a bot.
    NotBot,
    /// Client is not a human player.
    NotHuman,
    /// Client is not a spectator.
    NotSpectator,
    /// Entity is not valid or not spawned in the engine.
    InvalidEntity,
    /// Entity is not solid.
    NotSolid,
    /// None of the alternatives in an [`Any`] specification were satisfied.
    AnyFailed(Vec<SpecError>),
    /// Custom or dynamic specification condition failed.
    ConditionFailed(String),
}

impl fmt::Display for SpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAlive => write!(f, "target is not alive"),
            Self::NotDead => write!(f, "target is not dead"),
            Self::NotConnected => write!(f, "client is not connected"),
            Self::NotBot => write!(f, "client is not a bot"),
            Self::NotHuman => write!(f, "client is not a human player"),
            Self::NotSpectator => write!(f, "client is not a spectator"),
            Self::InvalidEntity => write!(f, "entity handle is invalid or unspawned"),
            Self::NotSolid => write!(f, "entity is not solid"),
            Self::AnyFailed(errs) => {
                write!(
                    f,
                    "all alternatives failed in Any specification: {:?}",
                    errs
                )
            }
            Self::ConditionFailed(msg) => write!(f, "specification condition failed: {}", msg),
        }
    }
}

impl std::error::Error for SpecError {}

/// Trait for validating whether `Target` satisfies a given precondition or invariant.
pub trait Spec<Target> {
    /// Error returned if the specification check fails.
    type Error: fmt::Display;

    /// Validates whether `target` satisfies this specification.
    fn check(target: &Target) -> Result<(), Self::Error>;
}

// --- Logical Combinators ---

/// Logical conjunction (AND): requires that all nested specifications are satisfied.
pub struct All<T>(pub PhantomData<T>);

/// Logical disjunction (OR): requires that at least one nested specification is satisfied.
pub struct Any<T>(pub PhantomData<T>);

/// Logical negation (NOT): requires that the nested specification fails.
pub struct Not<T>(pub PhantomData<T>);

/// Logical exclusion (NOR): requires that none of the nested specifications are satisfied.
pub struct NoneOf<T>(pub PhantomData<T>);

impl<Target, T: Spec<Target>> Spec<Target> for Not<T> {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Target) -> Result<(), Self::Error> {
        match T::check(target) {
            Ok(()) => Err(SpecError::ConditionFailed(
                "negated specification succeeded unexpectedly".into(),
            )),
            Err(_) => Ok(()),
        }
    }
}

impl<Target, T> Spec<Target> for NoneOf<T>
where
    Not<T>: Spec<Target, Error = SpecError>,
{
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Target) -> Result<(), Self::Error> {
        <Not<T> as Spec<Target>>::check(target)
    }
}

// Any for single items
impl<Target, T: Spec<Target>> Spec<Target> for All<T> {
    type Error = T::Error;

    #[inline(always)]
    fn check(target: &Target) -> Result<(), Self::Error> {
        T::check(target)
    }
}

// --- Declarative Macro for Tuple Variadics ---

macro_rules! impl_tuple_specs {
    ($($T:ident),+) => {
        // Direct tuple implementation: (A, B, ...)
        impl<Target, $($T),+> Spec<Target> for ($($T,)+)
        where
            $($T: Spec<Target, Error = SpecError>),+,
        {
            type Error = SpecError;

            #[inline(always)]
            fn check(target: &Target) -> Result<(), Self::Error> {
                $(
                    $T::check(target)?;
                )+
                Ok(())
            }
        }

        // Any<(A, B, ...)>
        impl<Target, $($T),+> Spec<Target> for Any<($($T,)+)>
        where
            $($T: Spec<Target, Error = SpecError>),+,
        {
            type Error = SpecError;

            #[inline(always)]
            fn check(target: &Target) -> Result<(), Self::Error> {
                let mut errors = Vec::new();
                $(
                    match $T::check(target) {
                        Ok(()) => return Ok(()),
                        Err(e) => errors.push(e),
                    }
                )+
                Err(SpecError::AnyFailed(errors))
            }
        }
    };
}

impl_tuple_specs!(A, B);
impl_tuple_specs!(A, B, C);
impl_tuple_specs!(A, B, C, D);
impl_tuple_specs!(A, B, C, D, E);
impl_tuple_specs!(A, B, C, D, E, F);
impl_tuple_specs!(A, B, C, D, E, F, G);
impl_tuple_specs!(A, B, C, D, E, F, G, H);

// --- Refined Witness Guard ---

/// A state-guarded witness token proving that `Target` satisfies specification `S`.
///
/// Cannot be constructed without executing [`Spec::check`]. Dereferences transparently
/// to the underlying `Target`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Refined<Target, S> {
    pub inner: Target,
    _marker: PhantomData<S>,
}

impl<Target, S: Spec<Target>> Refined<Target, S> {
    /// Attempts to construct a refined witness guard by verifying [`Spec::check`].
    #[inline(always)]
    pub fn try_new(target: Target) -> Result<Self, S::Error> {
        S::check(&target)?;
        Ok(Self {
            inner: target,
            _marker: PhantomData,
        })
    }

    /// Consumes this witness token, returning the underlying target.
    #[inline(always)]
    pub fn into_inner(self) -> Target {
        self.inner
    }

    /// Borrows the underlying target immutably.
    #[inline(always)]
    pub fn target(&self) -> &Target {
        &self.inner
    }

    /// Borrows the underlying target mutably.
    #[inline(always)]
    pub fn target_mut(&mut self) -> &mut Target {
        &mut self.inner
    }
}

impl<Target, S> std::ops::Deref for Refined<Target, S> {
    type Target = Target;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Target, S> std::ops::DerefMut for Refined<Target, S> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Automatic type-safe argument parsing for refined command arguments.
impl<Target: FromArg, S: Spec<Target>> FromArg for Refined<Target, S> {
    fn from_arg(token: &str) -> Result<Self, String> {
        let target = Target::from_arg(token)?;
        Refined::<Target, S>::try_new(target).map_err(|e| e.to_string())
    }
}

/// Ergonomic extension trait for refining entities into state-guarded [`Refined`] wrappers.
pub trait RefineExt: Sized {
    /// Validates `S` against `self`, returning a [`Refined`] witness guard upon success.
    #[inline(always)]
    fn refine<S: Spec<Self>>(self) -> Result<Refined<Self, S>, S::Error> {
        Refined::try_new(self)
    }

    /// Validates `S` against `&self`, returning an immutable [`Refined<&Self, S>`] guard.
    #[inline(always)]
    fn refine_ref<S: Spec<Self>>(&self) -> Result<Refined<&Self, S>, S::Error> {
        S::check(self)?;
        Ok(Refined {
            inner: self,
            _marker: PhantomData,
        })
    }

    /// Validates `S` against `&mut self`, returning a mutable [`Refined<&mut Self, S>`] guard.
    #[inline(always)]
    fn refine_mut<S: Spec<Self>>(&mut self) -> Result<Refined<&mut Self, S>, S::Error> {
        S::check(self)?;
        Ok(Refined {
            inner: self,
            _marker: PhantomData,
        })
    }
}

impl<T> RefineExt for T {}

// --- Domain Zero-Sized Types (ZST) Markers ---

pub mod markers {
    /// Typestate marker or container indicating a living player character (`health > 0`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Alive<T = ()>(pub T);

    /// Typestate marker or container indicating a dead player character.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Dead<T = ()>(pub T);

    /// Typestate marker or container indicating an AI bot client (`FL_FAKECLIENT`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Bot<T = ()>(pub T);

    /// Typestate marker or container indicating a spectator client.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Spectator<T = ()>(pub T);

    /// Typestate marker indicating a connected player client slot (1..=32).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Connected;

    /// Typestate marker indicating a human player (non-bot, non-HLTV).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Human;

    /// Typestate marker indicating a spawned and valid entity.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Spawned;

    /// Typestate marker indicating a dormant or inactive entity.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Dormant;

    /// Typestate marker indicating an entity with collision geometry.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Solid;
}

pub use markers::{Alive, Bot, Connected, Dead, Dormant, Human, Solid, Spawned, Spectator};

// --- Spec Implementations for Domain Markers ---

impl Spec<Entity> for Spawned {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Entity) -> Result<(), Self::Error> {
        if target.is_valid() {
            Ok(())
        } else {
            Err(SpecError::InvalidEntity)
        }
    }
}

impl Spec<Entity> for Solid {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Entity) -> Result<(), Self::Error> {
        if target.is_valid() {
            Ok(())
        } else {
            Err(SpecError::NotSolid)
        }
    }
}

impl Spec<Player> for Connected {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Player) -> Result<(), Self::Error> {
        if target.is_valid() {
            Ok(())
        } else {
            Err(SpecError::NotConnected)
        }
    }
}

impl<T> Spec<Player> for Alive<T> {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Player) -> Result<(), Self::Error> {
        if target.is_valid() && target.is_alive() {
            Ok(())
        } else {
            Err(SpecError::NotAlive)
        }
    }
}

impl<T> Spec<Player> for Dead<T> {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Player) -> Result<(), Self::Error> {
        if target.is_valid() && !target.is_alive() {
            Ok(())
        } else {
            Err(SpecError::NotDead)
        }
    }
}

impl<T> Spec<Player> for Bot<T> {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Player) -> Result<(), Self::Error> {
        if target.is_valid() && target.is_bot() {
            Ok(())
        } else {
            Err(SpecError::NotBot)
        }
    }
}

impl Spec<Player> for Human {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Player) -> Result<(), Self::Error> {
        if target.is_valid() && !target.is_bot() && !target.is_hltv() {
            Ok(())
        } else {
            Err(SpecError::NotHuman)
        }
    }
}

impl<T> Spec<Player> for Spectator<T> {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Player) -> Result<(), Self::Error> {
        if target.is_valid()
            && (target.life_state() == LifeState::Dead || target.team().is_spectator())
        {
            Ok(())
        } else {
            Err(SpecError::NotSpectator)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct MockTarget {
        alive: bool,
        connected: bool,
        money: i32,
    }

    struct IsAlive;
    impl Spec<MockTarget> for IsAlive {
        type Error = SpecError;
        fn check(target: &MockTarget) -> Result<(), Self::Error> {
            if target.alive {
                Ok(())
            } else {
                Err(SpecError::NotAlive)
            }
        }
    }

    struct IsConnected;
    impl Spec<MockTarget> for IsConnected {
        type Error = SpecError;
        fn check(target: &MockTarget) -> Result<(), Self::Error> {
            if target.connected {
                Ok(())
            } else {
                Err(SpecError::NotConnected)
            }
        }
    }

    struct HasMoney;
    impl Spec<MockTarget> for HasMoney {
        type Error = SpecError;
        fn check(target: &MockTarget) -> Result<(), Self::Error> {
            if target.money >= 100 {
                Ok(())
            } else {
                Err(SpecError::ConditionFailed("insufficient money".into()))
            }
        }
    }

    #[test]
    fn test_single_spec_success_and_failure() {
        let mut target = MockTarget {
            alive: true,
            connected: true,
            money: 50,
        };

        assert!(target.refine_ref::<IsAlive>().is_ok());

        target.alive = false;
        assert_eq!(
            target.refine_ref::<IsAlive>().err(),
            Some(SpecError::NotAlive)
        );
    }

    #[test]
    fn test_tuple_all_spec() {
        let mut target = MockTarget {
            alive: true,
            connected: true,
            money: 200,
        };

        let refined = target.refine_mut::<(IsAlive, IsConnected, HasMoney)>();
        assert!(refined.is_ok());

        let mut refined = refined.unwrap();
        assert_eq!(refined.money, 200);
        refined.money -= 100;
        assert_eq!(refined.money, 100);

        // Fail one precondition
        target.connected = false;
        assert_eq!(
            target
                .refine_ref::<All<(IsAlive, IsConnected, HasMoney)>>()
                .err(),
            Some(SpecError::NotConnected)
        );
    }

    #[test]
    fn test_any_spec() {
        let mut target = MockTarget {
            alive: false,
            connected: true,
            money: 0,
        };

        // One succeeds (connected)
        assert!(target.refine_ref::<Any<(IsAlive, IsConnected)>>().is_ok());

        target.connected = false;
        // Both fail
        assert!(target.refine_ref::<Any<(IsAlive, IsConnected)>>().is_err());
    }

    #[test]
    fn test_not_and_none_of_spec() {
        let mut target = MockTarget {
            alive: false,
            connected: true,
            money: 0,
        };

        assert!(target.refine_ref::<Not<IsAlive>>().is_ok());
        assert!(target.refine_ref::<NoneOf<IsAlive>>().is_ok());

        target.alive = true;
        assert!(target.refine_ref::<Not<IsAlive>>().is_err());
        assert!(target.refine_ref::<NoneOf<IsAlive>>().is_err());
    }
}
