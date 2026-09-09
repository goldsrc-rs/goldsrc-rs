//! Extension traits providing client and gameplay shortcuts (slots 1..=32: Player, Bot, HLTV).

use crate::action::{
    CheckCapability, CloseMenu, GiveItem, GrantCapability, PlaySound, Print, RevokeCapability,
    SendHud, ShowMenu, ShowRawMenu,
};
use crate::client::property::{Lang, Name};
use crate::client::{Client, ClientKind, LifeState, Player, PrintTarget, Team};
#[cfg(not(target_arch = "wasm32"))]
use crate::consts::{FL_FAKECLIENT, FL_PROXY};
use crate::entity::EntityExt;
use crate::hud::HudMessage;
use crate::menu::Menu;
use crate::property::Armor;

/// Extension trait providing client-specific queries and actions (slots 1..=32: Player, Bot, HLTV).
pub trait ClientExt: EntityExt {
    /// Returns the 1-based client slot index (1..=32).
    fn client_index(&self) -> i32;
    /// Returns the client display name, if set.
    fn name(&self) -> Option<String>;
    /// Returns the client's preferred language code.
    fn lang(&self) -> String;
    /// Returns the client kind (Player, Bot, HLTV).
    fn client_kind(&self) -> ClientKind;
    /// Returns `true` if this client is an AI bot (`FL_FAKECLIENT`).
    fn is_bot(&self) -> bool;
    /// Returns `true` if this client is an HLTV proxy (`FL_PROXY`).
    fn is_hltv(&self) -> bool;
    /// Returns the client's current game team.
    fn team(&self) -> Team;
    /// Prints a message to the specified target.
    fn print(&self, target: PrintTarget, msg: impl Into<String>);
    /// Prints a message to client's console.
    fn print_console(&self, msg: impl Into<String>);
    /// Prints a message to client's chat.
    fn print_chat(&self, msg: impl Into<String>);
    /// Prints a center notification message to client's screen.
    fn print_center(&self, msg: impl Into<String>);
    /// Prints a colorized chat message.
    fn print_color(&self, msg: impl Into<String>);
    /// Prints a top-left notification to client's screen.
    fn print_notify(&self, msg: impl Into<String>);
    /// Plays an audio sound effect for this client.
    fn play_sound(&self, sample: impl Into<String>);
}

/// Extension trait providing gameplay combatant operations (Human Player, Bot).
pub trait PlayerExt: ClientExt {
    /// Returns the player's armor value.
    fn armorvalue(&self) -> f32;
    /// Sets the player's armor value.
    fn set_armorvalue(&mut self, armor: f32);
    /// Returns the player's armor points (`Armor`).
    fn armor(&self) -> Armor;
    /// Sets the player's armor points.
    fn set_armor(&mut self, armor: impl Into<Armor>);
    /// Returns the player's current life state.
    fn life_state(&self) -> LifeState;
    /// Opens an interactive declarative menu for this player.
    fn open_menu(&self, menu: &Menu);
    /// Displays a menu for the player.
    fn show_menu(&self, menu: &Menu);
    /// Displays a raw `ShowMenu` dialog to the player.
    fn show_raw_menu(&self, keys_mask: i32, timeout: i32, text: &str);
    /// Closes any currently displayed menu on the player's client.
    fn close_menu(&self);
    /// Sends a HUD or DHUD message to the player.
    fn send_hud(&self, msg: &HudMessage);
    /// Spawns an item or weapon entity and delivers it to the player.
    fn give_item(&self, item: impl Into<String>) -> Option<i32>;
    /// Checks if the player has the specified capability.
    fn has_capability(&self, name: &str) -> bool;
    /// Grants a capability to the player dynamically.
    fn grant_capability(&self, name: impl Into<String>) -> bool;
    /// Revokes a capability from the player dynamically.
    fn revoke_capability(&self, name: impl Into<String>) -> bool;
}

impl ClientExt for Client {
    #[inline(always)]
    fn client_index(&self) -> i32 {
        self.index
    }

    #[inline(always)]
    fn name(&self) -> Option<String> {
        self.get::<Name>().0
    }

    #[inline(always)]
    fn lang(&self) -> String {
        self.get::<Lang>().0
    }

    #[inline(always)]
    fn client_kind(&self) -> ClientKind {
        if self.is_hltv() {
            ClientKind::Hltv
        } else if self.is_bot() {
            ClientKind::Bot
        } else {
            ClientKind::Player
        }
    }

    #[inline(always)]
    fn is_bot(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(flags) = self.inner.flags() {
                return (flags & FL_FAKECLIENT) != 0;
            }
            false
        }
        #[cfg(target_arch = "wasm32")]
        {
            false
        }
    }

    #[inline(always)]
    fn is_hltv(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(flags) = self.inner.flags() {
                return (flags & FL_PROXY) != 0;
            }
            false
        }
        #[cfg(target_arch = "wasm32")]
        {
            false
        }
    }

    #[inline(always)]
    fn team(&self) -> Team {
        self.get::<Team>()
    }

    #[inline(always)]
    fn print(&self, target: PrintTarget, msg: impl Into<String>) {
        self.act(Print {
            target,
            message: msg.into(),
        });
    }

    #[inline(always)]
    fn print_console(&self, msg: impl Into<String>) {
        self.act(Print::console(msg));
    }

    #[inline(always)]
    fn print_chat(&self, msg: impl Into<String>) {
        self.act(Print::chat(msg));
    }

    #[inline(always)]
    fn print_center(&self, msg: impl Into<String>) {
        self.act(Print::center(msg));
    }

    #[inline(always)]
    fn print_color(&self, msg: impl Into<String>) {
        self.act(Print::colored_chat(msg));
    }

    #[inline(always)]
    fn print_notify(&self, msg: impl Into<String>) {
        self.act(Print::notify(msg));
    }

    #[inline(always)]
    fn play_sound(&self, sample: impl Into<String>) {
        self.act(PlaySound::new(sample));
    }
}

impl ClientExt for Player {
    #[inline(always)]
    fn client_index(&self) -> i32 {
        self.client().client_index()
    }

    #[inline(always)]
    fn name(&self) -> Option<String> {
        self.client().name()
    }

    #[inline(always)]
    fn lang(&self) -> String {
        self.client().lang()
    }

    #[inline(always)]
    fn client_kind(&self) -> ClientKind {
        self.client().client_kind()
    }

    #[inline(always)]
    fn is_bot(&self) -> bool {
        self.client().is_bot()
    }

    #[inline(always)]
    fn is_hltv(&self) -> bool {
        self.client().is_hltv()
    }

    #[inline(always)]
    fn team(&self) -> Team {
        self.client().team()
    }

    #[inline(always)]
    fn print(&self, target: PrintTarget, msg: impl Into<String>) {
        self.client().print(target, msg);
    }

    #[inline(always)]
    fn print_console(&self, msg: impl Into<String>) {
        self.client().print_console(msg);
    }

    #[inline(always)]
    fn print_chat(&self, msg: impl Into<String>) {
        self.client().print_chat(msg);
    }

    #[inline(always)]
    fn print_center(&self, msg: impl Into<String>) {
        self.client().print_center(msg);
    }

    #[inline(always)]
    fn print_color(&self, msg: impl Into<String>) {
        self.client().print_color(msg);
    }

    #[inline(always)]
    fn print_notify(&self, msg: impl Into<String>) {
        self.client().print_notify(msg);
    }

    #[inline(always)]
    fn play_sound(&self, sample: impl Into<String>) {
        self.client().play_sound(sample);
    }
}

impl PlayerExt for Player {
    #[inline(always)]
    fn armorvalue(&self) -> f32 {
        self.armor().value()
    }

    #[inline(always)]
    fn set_armorvalue(&mut self, armor: f32) {
        self.set(Armor::new(armor));
    }

    #[inline(always)]
    fn armor(&self) -> Armor {
        self.get::<Armor>()
    }

    #[inline(always)]
    fn set_armor(&mut self, armor: impl Into<Armor>) {
        self.set(armor.into());
    }

    #[inline(always)]
    fn life_state(&self) -> LifeState {
        self.get::<LifeState>()
    }

    #[inline(always)]
    fn open_menu(&self, menu: &Menu) {
        self.act(ShowMenu::new(menu));
    }

    #[inline(always)]
    fn show_menu(&self, menu: &Menu) {
        self.open_menu(menu);
    }

    #[inline(always)]
    fn show_raw_menu(&self, keys_mask: i32, timeout: i32, text: &str) {
        self.act(ShowRawMenu {
            keys_mask,
            timeout,
            text,
        });
    }

    #[inline(always)]
    fn close_menu(&self) {
        self.act(CloseMenu);
    }

    #[inline(always)]
    fn send_hud(&self, msg: &HudMessage) {
        self.act(SendHud::new(msg));
    }

    #[inline(always)]
    fn give_item(&self, item: impl Into<String>) -> Option<i32> {
        self.act(GiveItem::new(item))
    }

    #[inline(always)]
    fn has_capability(&self, name: &str) -> bool {
        self.act(CheckCapability(name))
    }

    #[inline(always)]
    fn grant_capability(&self, name: impl Into<String>) -> bool {
        self.act(GrantCapability::new(name))
    }

    #[inline(always)]
    fn revoke_capability(&self, name: impl Into<String>) -> bool {
        self.act(RevokeCapability::new(name))
    }
}
