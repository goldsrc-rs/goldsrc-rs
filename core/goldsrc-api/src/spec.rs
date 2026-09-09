//! Compile-Time Specifications, Zero-Sized Typestates, and Refinement Guards.
//!
//! Provides type-level preconditions and compound specifications ([`All`], [`Any`], [`NoneOf`])
//! evaluated at function boundaries, guaranteeing entity state validity at compile time via [`Refined`].

use std::fmt;
use std::marker::PhantomData;

use crate::command::FromArg;

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
    /// Client is not an HLTV proxy.
    NotHltv,
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
            Self::NotHltv => write!(f, "client is not an HLTV proxy"),
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

/// A frame-scoped, state-guarded witness token proving that `Target` satisfies specification `S`.
///
/// Cannot be constructed without executing [`Spec::check`]. Dereferences transparently
/// to the underlying `Target`.
///
/// # Lifetime Contract
/// The `'a` lifetime binds this guard to the current tick or hook scope (e.g. from borrowing
/// the underlying entity), mathematically preventing stale witness handles from being stored across frames.
#[must_use = "Refined witness guards are frame-scoped and must be evaluated/consumed within the current tick or hook context"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Refined<'a, Target, S> {
    pub inner: Target,
    _marker: PhantomData<(&'a (), S)>,
}

impl<'a, Target, S: Spec<Target>> Refined<'a, Target, S> {
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

impl<'a, Target, S> std::ops::Deref for Refined<'a, Target, S> {
    type Target = Target;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, Target, S> std::ops::DerefMut for Refined<'a, Target, S> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Automatic type-safe argument parsing for refined command arguments.
impl<'a, Target: FromArg, S: Spec<Target>> FromArg for Refined<'a, Target, S> {
    fn from_arg(token: &str) -> Result<Self, String> {
        let target = Target::from_arg(token)?;
        Refined::<'a, Target, S>::try_new(target).map_err(|e| e.to_string())
    }
}

/// Ergonomic extension trait for refining entities into state-guarded [`Refined`] wrappers.
pub trait RefineExt: Sized {
    /// Validates `S` against `&'a self`, returning a frame-scoped [`Refined<'a, Self, S>`] guard.
    #[inline(always)]
    fn refine<'a, S: Spec<Self>>(&'a self) -> Result<Refined<'a, Self, S>, S::Error>
    where
        Self: Copy,
    {
        S::check(self)?;
        Ok(Refined {
            inner: *self,
            _marker: PhantomData,
        })
    }

    /// Validates `S` against `&'a self`, returning an immutable [`Refined<'a, &'a Self, S>`] guard.
    #[inline(always)]
    fn refine_ref<'a, S: Spec<Self>>(&'a self) -> Result<Refined<'a, &'a Self, S>, S::Error> {
        S::check(self)?;
        Ok(Refined {
            inner: self,
            _marker: PhantomData,
        })
    }

    /// Validates `S` against `&'a mut self`, returning a mutable [`Refined<'a, &'a mut Self, S>`] guard.
    #[inline(always)]
    fn refine_mut<'a, S: Spec<Self>>(
        &'a mut self,
    ) -> Result<Refined<'a, &'a mut Self, S>, S::Error> {
        S::check(self)?;
        Ok(Refined {
            inner: self,
            _marker: PhantomData,
        })
    }
}

impl<T> RefineExt for T {}

// --- Domain Zero-Sized Types (ZST) Markers & Re-exports ---

pub use crate::client::spec::{
    Alive, Bot, Connected, ConnectedClient, Dead, DeadPlayer, Hltv, Human, HumanClient,
    LivingHuman, LivingPlayer, SpectatingPlayer, Spectator,
};
pub use crate::entity::spec::{Dormant, Solid, SolidEntity, Spawned, SpawnedEntity};

pub mod markers {
    pub use crate::client::spec::{Alive, Bot, Connected, Dead, Hltv, Human, Spectator};
    pub use crate::entity::spec::{Dormant, Solid, Spawned};
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
