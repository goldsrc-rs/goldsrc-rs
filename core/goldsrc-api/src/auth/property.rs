//! Dynamic player authorization capability check (`Capability`).

use crate::action::Action;
use crate::client::Player;

/// Dynamic player authorization capability check action.
///
/// Returns `true` if the player has the specified capability flag.
///
/// Example:
/// ```ignore
/// let is_admin = player.act(Capability("admin.ban"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capability<'a>(pub &'a str);

impl<'a> Action<Player> for Capability<'a> {
    type Output = bool;

    #[inline(always)]
    fn execute(self, player: &Player) -> Self::Output {
        crate::auth::Auth::has_capability(player.index, self.0)
    }
}
