//! Engine console variables (cvar) operations.

pub use crate::cvar::{CvarEngine, CvarFlags};

/// Console variable operations extending the base [`CvarEngine`] capability.
pub trait EngineCvars: CvarEngine + Send + Sync {}

impl<T: CvarEngine + Send + Sync> EngineCvars for T {}
