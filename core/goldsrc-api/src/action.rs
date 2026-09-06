//! Universal Entity and Player Action System (`PlayerAction` & Value Objects).
//!
//! Models all side-effects and engine operations performed on entities as explicit
//! command objects, enabling centralized interception, auditing, and lifecycle cancellation.

use crate::client::{Player, PrintTarget};
use crate::hud::HudMessage;
use crate::menu::Menu;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Trait implemented by any executable command or side-effect on a `Player`.
pub trait PlayerAction {
    /// Result produced by executing the action.
    type Output;

    /// Executes the action against the given player handle.
    fn execute(self, player: &Player) -> Self::Output;
}

/// Handle for observing or triggering cancellation of long-running or repeating actions.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Creates a new active (uncancelled) cancellation token.
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Triggers cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Checks whether cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// Resets the cancellation state back to active (false).
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::Relaxed);
    }
}

/// Standard player side-effects and engine commands.
#[allow(clippy::module_inception)]
pub mod action {
    use super::*;

    /// Output a message to the player's screen, console, or chat.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Print {
        pub target: PrintTarget,
        pub message: String,
    }

    impl Print {
        /// Creates a chat print action (`say` / `say_team` destination).
        pub fn chat(msg: impl Into<String>) -> Self {
            Self {
                target: PrintTarget::Chat,
                message: msg.into(),
            }
        }

        /// Creates a center screen notification action (`HUD_PRINTCENTER`).
        pub fn center(msg: impl Into<String>) -> Self {
            Self {
                target: PrintTarget::Center,
                message: msg.into(),
            }
        }

        /// Creates a developer console print action (`HUD_PRINTCONSOLE`).
        pub fn console(msg: impl Into<String>) -> Self {
            Self {
                target: PrintTarget::Console,
                message: msg.into(),
            }
        }

        /// Creates a top-left HUD notification action (`HUD_PRINTNOTIFY`).
        pub fn notify(msg: impl Into<String>) -> Self {
            Self {
                target: PrintTarget::Notify,
                message: msg.into(),
            }
        }
    }

    impl PlayerAction for Print {
        type Output = ();

        #[inline(always)]
        fn execute(self, player: &Player) -> Self::Output {
            player.print(self.target, &self.message);
        }
    }

    /// Emit an audio sound effect targeting the player.
    #[derive(Debug, Clone, PartialEq)]
    pub struct PlaySound {
        pub sound_path: String,
        pub volume: f32,
        pub pitch: i32,
    }

    impl PlaySound {
        /// Creates a new sound action with default volume (1.0) and pitch (100).
        pub fn new(path: impl Into<String>) -> Self {
            Self {
                sound_path: path.into(),
                volume: 1.0,
                pitch: 100,
            }
        }

        /// Sets audio volume in range `0.0..=1.0`.
        pub fn volume(mut self, vol: f32) -> Self {
            self.volume = vol.clamp(0.0, 1.0);
            self
        }

        /// Sets audio pitch (standard 100).
        pub fn pitch(mut self, p: i32) -> Self {
            self.pitch = p;
            self
        }
    }

    impl PlayerAction for PlaySound {
        type Output = ();

        #[inline(always)]
        fn execute(self, player: &Player) -> Self::Output {
            player.play_sound(&self.sound_path);
        }
    }

    /// Open an interactive declarative menu for the player.
    #[derive(Debug, Clone)]
    pub struct ShowMenu<'a> {
        pub menu: &'a Menu,
    }

    impl<'a> ShowMenu<'a> {
        /// Creates a new show menu action.
        pub fn new(menu: &'a Menu) -> Self {
            Self { menu }
        }
    }

    impl<'a> PlayerAction for ShowMenu<'a> {
        type Output = ();

        #[inline(always)]
        fn execute(self, player: &Player) -> Self::Output {
            player.open_menu(self.menu);
        }
    }

    /// Closes any currently displayed menu on the player's screen.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct CloseMenu;

    impl PlayerAction for CloseMenu {
        type Output = ();

        #[inline(always)]
        fn execute(self, player: &Player) -> Self::Output {
            player.close_menu();
        }
    }

    /// Sends a Director HUD (DHUD) or channel HUD message to the player.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SendHud<'a> {
        pub message: &'a HudMessage,
    }

    impl<'a> SendHud<'a> {
        /// Creates a new HUD transmission action.
        pub fn new(message: &'a HudMessage) -> Self {
            Self { message }
        }
    }

    impl<'a> PlayerAction for SendHud<'a> {
        type Output = ();

        #[inline(always)]
        fn execute(self, player: &Player) -> Self::Output {
            player.send_hud(self.message);
        }
    }

    /// Gives an item or weapon entity to the player by classname.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct GiveItem {
        pub classname: String,
    }

    impl GiveItem {
        /// Creates a give item action for the specified entity classname.
        pub fn new(classname: impl Into<String>) -> Self {
            Self {
                classname: classname.into(),
            }
        }
    }

    impl PlayerAction for GiveItem {
        type Output = Option<i32>;

        #[inline(always)]
        fn execute(self, player: &Player) -> Self::Output {
            player.give_item(&self.classname)
        }
    }

    /// Grants an authorization capability to the player dynamically.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct GrantCapability {
        pub capability: String,
    }

    impl GrantCapability {
        /// Creates a grant capability action.
        pub fn new(cap: impl Into<String>) -> Self {
            Self {
                capability: cap.into(),
            }
        }
    }

    impl PlayerAction for GrantCapability {
        type Output = bool;

        #[inline(always)]
        fn execute(self, player: &Player) -> Self::Output {
            crate::auth::Auth::grant_capability(player.index, &self.capability)
        }
    }

    /// Revokes an authorization capability from the player dynamically.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RevokeCapability {
        pub capability: String,
    }

    impl RevokeCapability {
        /// Creates a revoke capability action.
        pub fn new(cap: impl Into<String>) -> Self {
            Self {
                capability: cap.into(),
            }
        }
    }

    impl PlayerAction for RevokeCapability {
        type Output = bool;

        #[inline(always)]
        fn execute(self, player: &Player) -> Self::Output {
            crate::auth::Auth::revoke_capability(player.index, &self.capability)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let granted = player.act(action::GrantCapability::new("action.test.unique_jump"));
        assert!(granted);
        assert!(player.has_capability("action.test.unique_jump"));

        let revoked = player.act(action::RevokeCapability::new("action.test.unique_jump"));
        assert!(revoked);
        assert!(!player.has_capability("action.test.unique_jump"));
    }
}
