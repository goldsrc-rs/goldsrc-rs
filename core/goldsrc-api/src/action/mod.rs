//! Universal Entity and Player Action System and Value Objects.
//!
//! Encapsulates engine commands, client interactions, sound playback, HUD/Menu dispatch,
//! and item delivery into strongly-typed Command Pattern value objects executed via
//! [`Player::act`].

pub mod auth;
pub mod hud;
pub mod item;
pub mod menu;
pub mod print;
pub mod sound;
pub mod token;

pub use auth::{GrantCapability, RevokeCapability};
pub use hud::SendHud;
pub use item::GiveItem;
pub use menu::{CloseMenu, ShowMenu, ShowRawMenu};
pub use print::Print;
pub use sound::PlaySound;
pub use token::CancellationToken;

use crate::client::Player;

/// Generic trait for actions executable against a given `Target`.
pub trait Action<Target> {
    /// Type returned upon successful execution.
    type Output;

    /// Executes the action against the target entity or handle.
    fn execute(self, target: &Target) -> Self::Output;
}

/// Trait alias marker for actions executed directly against a [`Player`].
pub trait PlayerAction: Action<Player> {}
impl<T: Action<Player>> PlayerAction for T {}

/// Standard engine actions namespace for backward compatibility (`crate::action::action::*`).
#[allow(clippy::module_inception)]
pub mod action {
    pub use super::auth::*;
    pub use super::hud::*;
    pub use super::item::*;
    pub use super::menu::*;
    pub use super::print::*;
    pub use super::sound::*;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::PlayerExt;

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());

        let cloned = token.clone();
        cloned.cancel();
        assert!(token.is_cancelled());

        token.reset();
        assert!(!token.is_cancelled());
        assert!(!cloned.is_cancelled());
    }

    #[test]
    fn test_player_action_dispatch() {
        crate::auth::Auth::register_capability("action.test.unique_jump", "double jump");
        let player = Player::new(77);
        crate::auth::Auth::remove_player(77);
        assert!(!player.has_capability("action.test.unique_jump"));

        let granted = player.act(GrantCapability::new("action.test.unique_jump"));
        assert!(granted);
        assert!(player.has_capability("action.test.unique_jump"));

        let revoked = player.act(RevokeCapability::new("action.test.unique_jump"));
        assert!(revoked);
        assert!(!player.has_capability("action.test.unique_jump"));
    }
}
