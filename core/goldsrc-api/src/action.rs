//! Universal Entity and Player Action System and Value Objects.
//!
//! Encapsulates all side-effectual operations, state transitions, audio emissions,
//! menu interactions, and engine commands into strongly typed Value Objects.
//! Ensures Command-Query Separation (CQS) where queries (`get::<P>()`) are pure
//! and side-effects are routed through `player.act(Action)`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::client::{Player, PrintTarget};
use crate::hud::HudMessage;
use crate::menu::Menu;

/// Trait for executable actions applied to a `Target` (e.g. `Player`).
pub trait PlayerAction {
    /// Type of the result or token returned upon action completion.
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

        /// Creates a colored chat message action.
        pub fn colored_chat(msg: impl Into<String>) -> Self {
            Self {
                target: PrintTarget::ColoredChat,
                message: msg.into(),
            }
        }
    }

    impl PlayerAction for Print {
        type Output = ();

        #[inline(always)]
        fn execute(self, player: &Player) -> Self::Output {
            #[cfg(target_arch = "wasm32")]
            {
                use crate::bindings::goldsrc::engine::api as host;
                match self.target {
                    PrintTarget::Console => host::host_print_console(player.index, &self.message),
                    PrintTarget::Center => host::host_print_center(player.index, &self.message),
                    PrintTarget::Notify => host::host_print_notify(player.index, &self.message),
                    PrintTarget::Chat | PrintTarget::ColoredChat => {
                        host::host_print_chat(player.index, &self.message)
                    }
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Ok(lock) = crate::client::player::NATIVE_PRINT_HOOK.read()
                    && let Some(hook) = *lock
                {
                    hook(player.index, self.target, &self.message);
                }
            }
        }
    }

    /// Emit an audio sound effect targeting the player.
    #[derive(Debug, Clone, PartialEq)]
    pub struct PlaySound {
        pub sound_path: String,
        pub channel: i32,
        pub volume: f32,
        pub attenuation: f32,
        pub flags: i32,
        pub pitch: i32,
    }

    impl PlaySound {
        /// Creates a new sound action with default volume (1.0) and pitch (100).
        pub fn new(path: impl Into<String>) -> Self {
            Self {
                sound_path: path.into(),
                channel: 0,
                volume: 1.0,
                attenuation: 1.0,
                flags: 0,
                pitch: 100,
            }
        }

        /// Sets audio channel (e.g. `CHAN_AUTO = 0`, `CHAN_VOICE = 2`).
        pub fn channel(mut self, ch: i32) -> Self {
            self.channel = ch;
            self
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
            #[cfg(target_arch = "wasm32")]
            {
                crate::bindings::goldsrc::engine::api::host_emit_sound(
                    player.index,
                    self.channel,
                    &self.sound_path,
                    self.volume,
                    self.attenuation,
                    self.flags,
                    self.pitch,
                );
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (player, self);
            }
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
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Ok(lock) = crate::client::player::OPEN_MENU_HOOK.read()
                    && let Some(hook) = *lock
                {
                    hook(player.index, self.menu);
                    return;
                }
            }
            let total_players = crate::auth::Auth::total_players();
            let ctx = crate::menu::MenuContext {
                player_index: player.index,
                round_number: 1,
                round_time_elapsed: 0.0,
                is_alive: player.get::<crate::property::prop::Health>() > 0.0,
                players_count: if total_players > 0 {
                    total_players as u32
                } else {
                    1
                },
            };
            if let Some(rendered) = self.menu.render_page(&ctx, 0) {
                match rendered.renderer {
                    crate::menu::MenuRendererKind::Text => {
                        player.act(ShowRawMenu {
                            keys_mask: rendered.keys_mask as i32,
                            timeout: rendered.timeout,
                            text: &rendered.text,
                        });
                    }
                    crate::menu::MenuRendererKind::Dhud {
                        position,
                        color,
                        effect,
                    } => {
                        let hud_msg = crate::hud::HudMessage {
                            text: rendered.text.clone(),
                            kind: crate::hud::HudKind::Dhud,
                            color,
                            color2: color,
                            position,
                            effect,
                        };
                        player.act(SendHud::new(&hud_msg));
                        player.act(ShowRawMenu {
                            keys_mask: rendered.keys_mask as i32,
                            timeout: rendered.timeout,
                            text: "",
                        });
                    }
                }
            }
        }
    }

    /// Displays a raw `ShowMenu` dialog to the player.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ShowRawMenu<'a> {
        pub keys_mask: i32,
        pub timeout: i32,
        pub text: &'a str,
    }

    impl<'a> PlayerAction for ShowRawMenu<'a> {
        type Output = ();

        #[inline(always)]
        fn execute(self, player: &Player) -> Self::Output {
            #[cfg(target_arch = "wasm32")]
            crate::bindings::goldsrc::engine::api::host_show_menu(
                player.index,
                self.keys_mask,
                self.timeout,
                self.text,
            );
            #[cfg(not(target_arch = "wasm32"))]
            let _ = (player, self.keys_mask, self.timeout, self.text);
        }
    }

    /// Closes any currently displayed menu on the player's screen.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct CloseMenu;

    impl PlayerAction for CloseMenu {
        type Output = ();

        #[inline(always)]
        fn execute(self, player: &Player) -> Self::Output {
            player.act(ShowRawMenu {
                keys_mask: 0,
                timeout: 0,
                text: "",
            });
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
            let (effect_val, fade_in, fade_out, hold_time) = match self.message.effect {
                crate::hud::HudEffect::FadeInOut {
                    fade_in,
                    fade_out,
                    hold_time,
                } => (0, fade_in, fade_out, hold_time),
                crate::hud::HudEffect::Flicker {
                    fx_time: _,
                    hold_time,
                } => (1, 0.0, 0.0, hold_time),
                crate::hud::HudEffect::Typewriter {
                    char_time: _,
                    fade_out,
                    hold_time,
                } => (2, 0.05, fade_out, hold_time),
            };

            match self.message.kind {
                crate::hud::HudKind::Classic { channel } => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        crate::bindings::goldsrc::engine::api::host_send_hud_message(
                            player.index,
                            channel as i32,
                            self.message.position.x,
                            self.message.position.y,
                            self.message.color.r as i32,
                            self.message.color.g as i32,
                            self.message.color.b as i32,
                            self.message.color.a as i32,
                            effect_val,
                            fade_in,
                            fade_out,
                            hold_time,
                            &self.message.text,
                        );
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let _ = (channel, effect_val, fade_in, fade_out, hold_time);
                    }
                }
                crate::hud::HudKind::Dhud => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        crate::bindings::goldsrc::engine::api::host_send_dhud_message(
                            player.index,
                            self.message.position.x,
                            self.message.position.y,
                            self.message.color.r as i32,
                            self.message.color.g as i32,
                            self.message.color.b as i32,
                            self.message.color.a as i32,
                            effect_val,
                            fade_in,
                            fade_out,
                            hold_time,
                            &self.message.text,
                        );
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let _ = (player, effect_val, fade_in, fade_out, hold_time);
                    }
                }
            }
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
            #[cfg(target_arch = "wasm32")]
            {
                use crate::bindings::goldsrc::engine::api as host;
                let ent = host::host_create_named_entity(&self.classname)?;
                let o = host::host_entity_origin(player.index);
                host::host_entity_set_origin(
                    ent,
                    crate::bindings::goldsrc::engine::api::Vector3 {
                        x: o.x,
                        y: o.y,
                        z: o.z,
                    },
                );
                host::host_dispatch_spawn(ent);
                host::host_dispatch_touch(ent, player.index);
                Some(ent)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (player, self);
                None
            }
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
