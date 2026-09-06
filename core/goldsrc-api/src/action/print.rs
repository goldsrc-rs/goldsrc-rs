//! Message printing actions (console, chat, center, notify, colored chat).

use crate::action::Action;
use crate::client::{Player, PrintTarget};

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
                crate::bindings::goldsrc::engine::api::host_print_console(
                    player.index,
                    &self.message,
                );
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok(lock) = crate::client::player::NATIVE_PRINT_HOOK.read()
                    && let Some(hook) = *lock
                {
                    hook(player.index, PrintTarget::Console, &self.message);
                }
            }
            PrintTarget::Center => {
                #[cfg(target_arch = "wasm32")]
                crate::bindings::goldsrc::engine::api::host_print_center(
                    player.index,
                    &self.message,
                );
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok(lock) = crate::client::player::NATIVE_PRINT_HOOK.read()
                    && let Some(hook) = *lock
                {
                    hook(player.index, PrintTarget::Center, &self.message);
                }
            }
            PrintTarget::Chat => {
                #[cfg(target_arch = "wasm32")]
                crate::bindings::goldsrc::engine::api::host_print_chat(player.index, &self.message);
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok(lock) = crate::client::player::NATIVE_PRINT_HOOK.read()
                    && let Some(hook) = *lock
                {
                    hook(player.index, PrintTarget::Chat, &self.message);
                }
            }
            PrintTarget::Notify => {
                #[cfg(target_arch = "wasm32")]
                crate::bindings::goldsrc::engine::api::host_print_notify(
                    player.index,
                    &self.message,
                );
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok(lock) = crate::client::player::NATIVE_PRINT_HOOK.read()
                    && let Some(hook) = *lock
                {
                    hook(player.index, PrintTarget::Notify, &self.message);
                }
            }
            PrintTarget::ColoredChat => {
                #[cfg(target_arch = "wasm32")]
                crate::bindings::goldsrc::engine::api::host_print_chat(player.index, &self.message);
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok(lock) = crate::client::player::NATIVE_PRINT_HOOK.read()
                    && let Some(hook) = *lock
                {
                    hook(player.index, PrintTarget::ColoredChat, &self.message);
                }
            }
        }
    }
}
