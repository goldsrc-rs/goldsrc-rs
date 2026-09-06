//! Screen HUD and DHUD rendering actions.

use crate::action::Action;
use crate::client::Player;

/// Sends a screen HUD or DHUD message to the player.
#[derive(Debug, Clone, PartialEq)]
pub struct SendHud<'a> {
    /// HUD message definition.
    pub message: &'a crate::hud::HudMessage,
}

impl<'a> SendHud<'a> {
    /// Creates a new HUD action with the given message definition.
    pub fn new(message: &'a crate::hud::HudMessage) -> Self {
        Self { message }
    }
}

impl<'a> Action<Player> for SendHud<'a> {
    type Output = ();

    #[inline(always)]
    fn execute(self, player: &Player) -> Self::Output {
        if !player.is_valid() {
            return;
        }

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
                    let _ = (player, channel, effect_val, fade_in, fade_out, hold_time);
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
