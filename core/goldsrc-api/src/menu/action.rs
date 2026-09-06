//! Interactive and raw menu display and closure actions.

use crate::action::Action;
use crate::client::Player;
use crate::hud::{HudKind, HudMessage, SendHud};
use crate::menu::{Menu, MenuContext, MenuRendererKind};
use crate::property::Health;

/// Displays an interactive declarative menu for the player.
#[derive(Debug, Clone)]
pub struct ShowMenu<'a> {
    /// Declarative menu configuration and pages.
    pub menu: &'a Menu,
}

impl<'a> ShowMenu<'a> {
    /// Creates a new declarative menu action.
    pub fn new(menu: &'a Menu) -> Self {
        Self { menu }
    }
}

impl<'a> Action<Player> for ShowMenu<'a> {
    type Output = ();

    #[inline(always)]
    fn execute(self, player: &Player) -> Self::Output {
        if !player.is_valid() {
            return;
        }

        let ctx = MenuContext {
            player_index: player.index,
            round_number: 0,
            round_time_elapsed: 0.0,
            is_alive: player.get::<Health>().is_alive(),
            players_count: 0,
        };

        if let Some(rendered) = self.menu.render_page(&ctx, 0) {
            match rendered.renderer {
                MenuRendererKind::Text => {
                    player.act(ShowRawMenu {
                        keys_mask: rendered.keys_mask as i32,
                        timeout: rendered.timeout,
                        text: &rendered.text,
                    });
                }
                MenuRendererKind::Dhud {
                    position,
                    color,
                    effect,
                } => {
                    let hud_msg = HudMessage {
                        text: rendered.text.clone(),
                        kind: HudKind::Dhud,
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

/// Closes any currently displayed menu on the player's client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CloseMenu;

impl Action<Player> for CloseMenu {
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

/// Displays a raw `ShowMenu` dialog to the player with keys mask and timeout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowRawMenu<'a> {
    /// Bitmask of selectable number keys (`(1 << 0) .. (1 << 9)`).
    pub keys_mask: i32,
    /// Timeout in seconds before automatically closing (-1 for infinite).
    pub timeout: i32,
    /// Menu content text.
    pub text: &'a str,
}

impl<'a> Action<Player> for ShowRawMenu<'a> {
    type Output = ();

    #[inline(always)]
    fn execute(self, player: &Player) -> Self::Output {
        if !player.is_valid() {
            return;
        }

        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_show_menu(
                player.index,
                self.keys_mask,
                self.timeout,
                self.text,
            );
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (player, self.keys_mask, self.timeout, self.text);
        }
    }
}
