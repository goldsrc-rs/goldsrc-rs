//! Universal Entity and Player Action System and Value Objects.
//!
//! Encapsulates engine commands, client interactions, sound playback, HUD/Menu dispatch,
//! and item delivery into strongly-typed Command Pattern value objects executed via
//! [`Player::act`].

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::Entity;
#[allow(unused_imports)]
use crate::bindings::goldsrc::engine::api as host_api;
#[cfg(not(target_arch = "wasm32"))]
use crate::client::player::NATIVE_PRINT_HOOK;
use crate::client::{Player, PrintTarget};

pub use crate::auth::action::{GrantCapability, RevokeCapability};
pub use crate::hud::action::SendHud;
pub use crate::menu::action::{CloseMenu, ShowMenu, ShowRawMenu};

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

/// Thread-safe cooperative cancellation token.
///
/// Can be passed alongside actions or tasks to allow premature cancellation
/// or rollback across threads or frame ticks.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Creates a new, active cancellation token (`is_cancelled == false`).
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Triggers cancellation on this token and all its clones.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Returns `true` if this token has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Resets the cancellation token to uncancelled state.
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }
}

/// Prints a message to the player's client via the specified target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Print {
    /// Where to render the message.
    pub target: PrintTarget,
    /// Body text to display.
    pub message: String,
}

impl Print {
    /// Creates a chat message action.
    pub fn chat(msg: impl Into<String>) -> Self {
        Self {
            target: PrintTarget::Chat,
            message: msg.into(),
        }
    }

    /// Creates a center screen notification action.
    pub fn center(msg: impl Into<String>) -> Self {
        Self {
            target: PrintTarget::Center,
            message: msg.into(),
        }
    }

    /// Creates a game console output action.
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

    /// Creates a colored chat message action.
    pub fn colored_chat(msg: impl Into<String>) -> Self {
        Self {
            target: PrintTarget::ColoredChat,
            message: msg.into(),
        }
    }
}

impl Action<Player> for Print {
    type Output = ();

    #[inline(always)]
    fn execute(self, player: &Player) -> Self::Output {
        if !player.is_valid() {
            return;
        }

        match self.target {
            PrintTarget::Console => {
                #[cfg(target_arch = "wasm32")]
                host_api::host_print_console(player.index, &self.message);
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok(lock) = NATIVE_PRINT_HOOK.read()
                    && let Some(hook) = *lock
                {
                    hook(player.index, PrintTarget::Console, &self.message);
                }
            }
            PrintTarget::Center => {
                #[cfg(target_arch = "wasm32")]
                host_api::host_print_center(player.index, &self.message);
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok(lock) = NATIVE_PRINT_HOOK.read()
                    && let Some(hook) = *lock
                {
                    hook(player.index, PrintTarget::Center, &self.message);
                }
            }
            PrintTarget::Chat => {
                #[cfg(target_arch = "wasm32")]
                host_api::host_print_chat(player.index, &self.message);
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok(lock) = NATIVE_PRINT_HOOK.read()
                    && let Some(hook) = *lock
                {
                    hook(player.index, PrintTarget::Chat, &self.message);
                }
            }
            PrintTarget::Notify => {
                #[cfg(target_arch = "wasm32")]
                host_api::host_print_notify(player.index, &self.message);
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok(lock) = NATIVE_PRINT_HOOK.read()
                    && let Some(hook) = *lock
                {
                    hook(player.index, PrintTarget::Notify, &self.message);
                }
            }
            PrintTarget::ColoredChat => {
                #[cfg(target_arch = "wasm32")]
                host_api::host_print_chat(player.index, &self.message);
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok(lock) = NATIVE_PRINT_HOOK.read()
                    && let Some(hook) = *lock
                {
                    hook(player.index, PrintTarget::ColoredChat, &self.message);
                }
            }
        }
    }
}

/// Plays an audio sample effect directly to the player's client or from an entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaySound {
    /// Relative sound filepath within the game directory (e.g. `"buttons/button10.wav"`).
    pub sample: String,
}

impl PlaySound {
    /// Creates a new sound playback action.
    pub fn new(sample: impl Into<String>) -> Self {
        Self {
            sample: sample.into(),
        }
    }
}

impl Action<Entity> for PlaySound {
    type Output = ();

    #[inline(always)]
    fn execute(self, entity: &Entity) -> Self::Output {
        if !entity.is_valid() {
            return;
        }

        #[cfg(target_arch = "wasm32")]
        {
            host_api::host_emit_sound(
                entity.index,
                0, // CHAN_AUTO
                &self.sample,
                1.0, // VOL_NORM
                1.0, // ATTN_NORM
                0,
                100, // PITCH_NORM
            );
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (entity, self.sample);
        }
    }
}

impl Action<Player> for PlaySound {
    type Output = ();

    #[inline(always)]
    fn execute(self, player: &Player) -> Self::Output {
        self.execute(&**player)
    }
}

/// Spawns an item or weapon entity by classname and delivers it to the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GiveItem {
    /// Entity classname to give (e.g. `"weapon_ak47"`, `"item_assaultsuit"`).
    pub item: String,
}

impl GiveItem {
    /// Creates a new item delivery action.
    pub fn new(item: impl Into<String>) -> Self {
        Self { item: item.into() }
    }
}

impl Action<Player> for GiveItem {
    type Output = Option<i32>;

    #[inline(always)]
    fn execute(self, player: &Player) -> Self::Output {
        if !player.is_valid() {
            return None;
        }

        #[cfg(target_arch = "wasm32")]
        {
            let ent = host_api::host_create_named_entity(&self.item)?;
            let o = host_api::host_entity_origin(player.index);
            host_api::host_entity_set_origin(
                ent,
                host_api::Vector3 {
                    x: o.x,
                    y: o.y,
                    z: o.z,
                },
            );
            host_api::host_dispatch_spawn(ent);
            host_api::host_dispatch_touch(ent, player.index);
            Some(ent)
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (player, self);
            None
        }
    }
}

/// Standard engine actions namespace for backward compatibility (`crate::action::action::*`).
#[allow(clippy::module_inception)]
pub mod action {
    pub use super::{
        CloseMenu, GiveItem, GrantCapability, PlaySound, Print, RevokeCapability, SendHud,
        ShowMenu, ShowRawMenu,
    };
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
        let _guard = crate::auth::AUTH_TEST_LOCK.lock().unwrap();
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
