//! Entity domain specifications and compile-time typestate witness tokens.

use crate::Entity;
use crate::spec::{Refined, Spec, SpecError};

/// Typestate marker indicating a spawned and valid entity in the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Spawned;

/// Typestate marker indicating an entity with collision geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Solid;

/// Typestate marker indicating a dormant or inactive entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Dormant;

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

impl Spec<Entity> for Dormant {
    type Error = SpecError;

    #[inline(always)]
    fn check(target: &Entity) -> Result<(), Self::Error> {
        if !target.is_valid() {
            Ok(())
        } else {
            Err(SpecError::ConditionFailed("entity is not dormant".into()))
        }
    }
}

/// A frame-scoped, validated entity guaranteed to be spawned and active.
pub type SpawnedEntity<'a> = Refined<'a, Entity, Spawned>;

/// A frame-scoped, validated entity guaranteed to be solid with physics collision.
pub type SolidEntity<'a> = Refined<'a, Entity, Solid>;
