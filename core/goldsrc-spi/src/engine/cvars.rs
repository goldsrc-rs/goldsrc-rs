//! Engine console variables (cvar) operations.

use goldsrc_api::cvar::CvarEngine;

/// Console variable operations extending the base [`CvarEngine`] capability.
pub trait EngineCvars: CvarEngine + Send + Sync {}

impl<T: CvarEngine + Send + Sync> EngineCvars for T {}
