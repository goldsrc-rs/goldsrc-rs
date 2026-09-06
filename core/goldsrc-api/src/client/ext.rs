//! Extension traits providing client and gameplay shortcuts (slots 1..=32: Player, Bot, HLTV).

use crate::client::{ClientKind, LifeState, Player, PrintTarget, Team};
use crate::entity::EntityExt;

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
    /// Prints a message to client's console.
    fn print_console(&self, msg: impl Into<String>);
    /// Prints a top-left notification to client's screen.
    fn print_notify(&self, msg: impl Into<String>);
}

/// Extension trait providing gameplay combatant operations (Human Player, Bot).
pub trait PlayerExt: ClientExt {
    /// Returns the player's armor value.
    fn armorvalue(&self) -> f32;
    /// Sets the player's armor value.
    fn set_armorvalue(&mut self, armor: f32);
    /// Returns the player's armor points (`Armor`).
    fn armor(&self) -> crate::property::Armor;
    /// Sets the player's armor points.
    fn set_armor(&mut self, armor: impl Into<crate::property::Armor>);
    /// Returns the player's current game team.
    fn team(&self) -> Team;
    /// Returns the player's current life state.
    fn life_state(&self) -> LifeState;
    /// Prints a message to the specified target.
    fn print(&self, target: PrintTarget, msg: impl Into<String>);
    /// Prints a message to player's chat.
    fn print_chat(&self, msg: impl Into<String>);
    /// Prints a center notification message to player's screen.
    fn print_center(&self, msg: impl Into<String>);
    /// Prints a colorized chat message.
    fn print_color(&self, msg: impl Into<String>);
    /// Plays an audio sound effect for this player.
    fn play_sound(&self, sample: impl Into<String>);
    /// Opens an interactive declarative menu for this player.
    fn open_menu(&self, menu: &crate::menu::Menu);
    /// Displays a menu for the player.
    fn show_menu(&self, menu: &crate::menu::Menu);
    /// Displays a raw `ShowMenu` dialog to the player.
    fn show_raw_menu(&self, keys_mask: i32, timeout: i32, text: &str);
    /// Closes any currently displayed menu on the player's client.
    fn close_menu(&self);
    /// Sends a HUD or DHUD message to the player.
    fn send_hud(&self, msg: &crate::hud::HudMessage);
    /// Spawns an item or weapon entity and delivers it to the player.
    fn give_item(&self, item: impl Into<String>) -> Option<i32>;
    /// Checks if the player has the specified capability.
    fn has_capability(&self, name: &str) -> bool;
    /// Grants a capability to the player dynamically.
    fn grant_capability(&self, name: impl Into<String>) -> bool;
    /// Revokes a capability from the player dynamically.
    fn revoke_capability(&self, name: impl Into<String>) -> bool;
}

impl ClientExt for Player {
    #[inline(always)]
    fn client_index(&self) -> i32 {
        self.index
    }

    #[inline(always)]
    fn name(&self) -> Option<String> {
        self.get::<crate::client::property::Name>().0
    }

    #[inline(always)]
    fn lang(&self) -> String {
        self.get::<crate::client::property::Lang>().0
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
                return (flags & crate::consts::FL_FAKECLIENT) != 0;
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
                return (flags & crate::consts::FL_PROXY) != 0;
            }
            false
        }
        #[cfg(target_arch = "wasm32")]
        {
            false
        }
    }

    #[inline(always)]
    fn print_console(&self, msg: impl Into<String>) {
        self.act(crate::action::Print::console(msg));
    }

    #[inline(always)]
    fn print_notify(&self, msg: impl Into<String>) {
        self.act(crate::action::Print::notify(msg));
    }
}

impl PlayerExt for Player {
    #[inline(always)]
    fn armorvalue(&self) -> f32 {
        self.armor().value()
    }

    #[inline(always)]
    fn set_armorvalue(&mut self, armor: f32) {
        self.set(crate::property::Armor::new(armor));
    }

    #[inline(always)]
    fn armor(&self) -> crate::property::Armor {
        self.get::<crate::property::Armor>()
    }

    #[inline(always)]
    fn set_armor(&mut self, armor: impl Into<crate::property::Armor>) {
        self.set(armor.into());
    }

    #[inline(always)]
    fn team(&self) -> Team {
        self.get::<Team>()
    }

    #[inline(always)]
    fn life_state(&self) -> LifeState {
        self.get::<LifeState>()
    }

    #[inline(always)]
    fn print(&self, target: PrintTarget, msg: impl Into<String>) {
        self.act(crate::action::Print {
            target,
            message: msg.into(),
        });
    }

    #[inline(always)]
    fn print_chat(&self, msg: impl Into<String>) {
        self.act(crate::action::Print::chat(msg));
    }

    #[inline(always)]
    fn print_center(&self, msg: impl Into<String>) {
        self.act(crate::action::Print::center(msg));
    }

    #[inline(always)]
    fn print_color(&self, msg: impl Into<String>) {
        self.act(crate::action::Print::colored_chat(msg));
    }

    #[inline(always)]
    fn play_sound(&self, sample: impl Into<String>) {
        self.act(crate::action::PlaySound::new(sample));
    }

    #[inline(always)]
    fn open_menu(&self, menu: &crate::menu::Menu) {
        self.act(crate::action::ShowMenu::new(menu));
    }

    #[inline(always)]
    fn show_menu(&self, menu: &crate::menu::Menu) {
        self.open_menu(menu);
    }

    #[inline(always)]
    fn show_raw_menu(&self, keys_mask: i32, timeout: i32, text: &str) {
        self.act(crate::action::ShowRawMenu {
            keys_mask,
            timeout,
            text,
        });
    }

    #[inline(always)]
    fn close_menu(&self) {
        self.act(crate::action::CloseMenu);
    }

    #[inline(always)]
    fn send_hud(&self, msg: &crate::hud::HudMessage) {
        self.act(crate::action::SendHud::new(msg));
    }

    #[inline(always)]
    fn give_item(&self, item: impl Into<String>) -> Option<i32> {
        self.act(crate::action::GiveItem::new(item))
    }

    #[inline(always)]
    fn has_capability(&self, name: &str) -> bool {
        self.act(crate::property::Capability(name))
    }

    #[inline(always)]
    fn grant_capability(&self, name: impl Into<String>) -> bool {
        self.act(crate::action::GrantCapability::new(name))
    }

    #[inline(always)]
    fn revoke_capability(&self, name: impl Into<String>) -> bool {
        self.act(crate::action::RevokeCapability::new(name))
    }
}
