//! Dynamic player authorization capability property (`Capability`).

use crate::client::Player;
use crate::property::{MutProperty, Property};

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
