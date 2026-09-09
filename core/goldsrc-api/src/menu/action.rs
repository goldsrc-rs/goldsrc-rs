//! Interactive and raw menu display and closure actions.

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use crate::action::Action;
#[cfg(target_arch = "wasm32")]
use crate::bindings::goldsrc::engine::api as host_api;
use crate::client::Player;
use crate::hud::{HudKind, HudMessage, SendHud};
use crate::menu::{Menu, MenuContext, MenuRendererKind, SlotAction, dispatch_menu_action};
use crate::property::Health;

static ACTIVE_PLAYER_MENUS: LazyLock<RwLock<HashMap<i32, (Menu, usize)>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Sets the active menu and page index for a given player.
pub fn set_active_player_menu(player_idx: i32, menu: Menu, page: usize) {
    if let Ok(mut lock) = ACTIVE_PLAYER_MENUS.write() {
        lock.insert(player_idx, (menu, page));
    }
}

/// Clears any active menu session recorded for the given player.
pub fn clear_active_player_menu(player_idx: i32) {
    if let Ok(mut lock) = ACTIVE_PLAYER_MENUS.write() {
        lock.remove(&player_idx);
    }
}

/// Returns a clone of the active menu and current page index for a given player.
pub fn get_active_player_menu(player_idx: i32) -> Option<(Menu, usize)> {
    ACTIVE_PLAYER_MENUS
        .read()
        .ok()
        .and_then(|lock| lock.get(&player_idx).cloned())
}

/// Renders and sends a specific page of a declarative menu to a player.
pub fn display_player_menu_page(player_idx: i32, menu: &Menu, page: usize) -> bool {
    let player = Player::new(player_idx);
    if !player.is_valid() {
        return false;
    }

    let is_alive = player.get::<Health>().is_alive();
    let ctx = MenuContext {
        player_index: player_idx,
        round_number: 0,
        round_time_elapsed: 0.0,
        is_alive,
        players_count: 0,
    };

    if let Some(rendered) = menu.render_page(&ctx, page) {
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
        set_active_player_menu(player_idx, menu.clone(), page);
        true
    } else {
        false
    }
}

/// Handles incoming slot selection (1..=10) for an active declarative player menu session.
///
/// Automatically flips pages for `NextPage` and `PrevPage`, closes on `Exit`,
/// and routes item clicks to registered action callbacks.
/// Returns `true` if handled by an active declarative menu, or `false` if no session was active.
pub fn handle_player_menu_select(player_idx: i32, slot: u8) -> bool {
    let Some((menu, current_page)) = get_active_player_menu(player_idx) else {
        return false;
    };

    let player = Player::new(player_idx);
    let is_alive = player.get::<Health>().is_alive();
    let ctx = MenuContext {
        player_index: player_idx,
        round_number: 0,
        round_time_elapsed: 0.0,
        is_alive,
        players_count: 0,
    };

    let Some(rendered) = menu.render_page(&ctx, current_page) else {
        clear_active_player_menu(player_idx);
        return false;
    };

    match rendered.slots.get(&slot) {
        Some(SlotAction::NextPage) => {
            let next_page = current_page + 1;
            display_player_menu_page(player_idx, &menu, next_page);
            true
        }
        Some(SlotAction::PrevPage) => {
            let prev_page = current_page.saturating_sub(1);
            display_player_menu_page(player_idx, &menu, prev_page);
            true
        }
        Some(SlotAction::Exit) => {
            clear_active_player_menu(player_idx);
            player.act(CloseMenu);
            true
        }
        Some(SlotAction::Execute {
            id,
            action_name,
            keep_open,
        }) => {
            let id = *id;
            let action_name = action_name.clone();
            let keep_open = *keep_open;
            if keep_open {
                display_player_menu_page(player_idx, &menu, current_page);
            } else {
                clear_active_player_menu(player_idx);
            }
            dispatch_menu_action(player, Some(id), Some(&action_name));
            true
        }
        Some(SlotAction::DenyFeedback(_)) | Some(SlotAction::Noop) => {
            display_player_menu_page(player_idx, &menu, current_page);
            true
        }
        None => false,
    }
}

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
        display_player_menu_page(player.index, self.menu, 0);
    }
}

/// Closes any currently displayed menu on the player's client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CloseMenu;

impl Action<Player> for CloseMenu {
    type Output = ();

    #[inline(always)]
    fn execute(self, player: &Player) -> Self::Output {
        clear_active_player_menu(player.index);
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
            host_api::host_show_menu(player.index, self.keys_mask, self.timeout, self.text);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (player, self.keys_mask, self.timeout, self.text);
        }
    }
}
