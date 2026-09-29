//! Decoupled Input Drivers for the GoldSrc.rs MVC Menu System.
//!
//! Provides the [`MenuInputDriver`] trait and implementations for handling:
//! - Classic numbered slot selection (`1..=10`) via [`SlotInputDriver`].
//! - Real-time player movement / action keys (`IN_*`) via [`ButtonInputDriver`].
//! - Seamless simultaneous input handling via [`HybridInputDriver`].
//! - [`GhostSlotTrap`] utility to lock client weapon slots during custom HUD/DHUD menus.

use crate::entity::property::Buttons;

/// High-level navigation and interaction actions resulting from input evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuInputAction {
    /// Move cursor / selection to previous item (e.g. `IN_FORWARD` or up arrow).
    NavigateUp,
    /// Move cursor / selection to next item (e.g. `IN_BACK` or down arrow).
    NavigateDown,
    /// Decrement active slider / previous sub-page (e.g. `IN_MOVELEFT`).
    NavigateLeft,
    /// Increment active slider / next sub-page (e.g. `IN_MOVERIGHT`).
    NavigateRight,
    /// Activate / toggle selected item (e.g. `IN_USE` or `IN_JUMP`).
    Select,
    /// Navigate to previous page (slot 8).
    PrevPage,
    /// Navigate to next page (slot 9).
    NextPage,
    /// Close menu or exit (slot 10 / 0).
    Exit,
    /// Direct slot selection (1..=10).
    DirectSlot(u8),
}

/// Abstract input driver trait in the MVC Menu System.
pub trait MenuInputDriver: Send + Sync {
    /// Process incoming console / client slot input (1..=10).
    fn handle_slot(&self, slot: u8) -> Option<MenuInputAction>;

    /// Process player physical controller buttons (`IN_*`) with rising-edge state detection.
    fn handle_buttons(&self, current: Buttons, previous: Buttons) -> Option<MenuInputAction>;
}

/// Standard numeric slot input driver (keys 1..=10 / `menuselect`).
#[derive(Debug, Clone, Copy, Default)]
pub struct SlotInputDriver;

impl MenuInputDriver for SlotInputDriver {
    fn handle_slot(&self, slot: u8) -> Option<MenuInputAction> {
        match slot {
            1..=7 => Some(MenuInputAction::DirectSlot(slot)),
            8 => Some(MenuInputAction::PrevPage),
            9 => Some(MenuInputAction::NextPage),
            10 => Some(MenuInputAction::Exit),
            _ => None,
        }
    }

    fn handle_buttons(&self, _current: Buttons, _previous: Buttons) -> Option<MenuInputAction> {
        None
    }
}

/// Real-time player movement and action key input driver (`pev->button` / `IN_*`).
///
/// Maps GoldSrc player movement keys:
/// - `IN_FORWARD` (W / Up): Move cursor up
/// - `IN_BACK` (S / Down): Move cursor down
/// - `IN_MOVELEFT` (A / Left): Move left / decrement slider
/// - `IN_MOVERIGHT` (D / Right): Move right / increment slider
/// - `IN_USE` / `IN_JUMP` (E / Space): Select / toggle
#[derive(Debug, Clone, Copy, Default)]
pub struct ButtonInputDriver;

impl ButtonInputDriver {
    /// Evaluates rising-edge button transitions (button pressed down on this frame).
    pub fn just_pressed(current: Buttons, previous: Buttons, mask: i32) -> bool {
        current.is_down(mask) && !previous.is_down(mask)
    }
}

impl MenuInputDriver for ButtonInputDriver {
    fn handle_slot(&self, _slot: u8) -> Option<MenuInputAction> {
        None
    }

    fn handle_buttons(&self, current: Buttons, previous: Buttons) -> Option<MenuInputAction> {
        if Self::just_pressed(current, previous, Buttons::IN_FORWARD) {
            return Some(MenuInputAction::NavigateUp);
        }
        if Self::just_pressed(current, previous, Buttons::IN_BACK) {
            return Some(MenuInputAction::NavigateDown);
        }
        if Self::just_pressed(current, previous, Buttons::IN_MOVELEFT) {
            return Some(MenuInputAction::NavigateLeft);
        }
        if Self::just_pressed(current, previous, Buttons::IN_MOVERIGHT) {
            return Some(MenuInputAction::NavigateRight);
        }
        if Self::just_pressed(current, previous, Buttons::IN_USE)
            || Self::just_pressed(current, previous, Buttons::IN_JUMP)
        {
            return Some(MenuInputAction::Select);
        }
        None
    }
}

/// Hybrid input driver combining numeric slot keys and real-time movement buttons.
#[derive(Debug, Clone, Copy, Default)]
pub struct HybridInputDriver;

impl MenuInputDriver for HybridInputDriver {
    fn handle_slot(&self, slot: u8) -> Option<MenuInputAction> {
        SlotInputDriver.handle_slot(slot)
    }

    fn handle_buttons(&self, current: Buttons, previous: Buttons) -> Option<MenuInputAction> {
        ButtonInputDriver.handle_buttons(current, previous)
    }
}

/// Helper managing the Ghost Slot Trap protocol.
///
/// Sends an empty/invisible `ShowMenu` packet with keys bitmask `0x3FF` (all 10 slots active).
/// This forces the GoldSrc client engine to capture slot keys `1..=10` into the menu subsystem
/// rather than dispatching them to weapon selection or slot switching.
pub struct GhostSlotTrap;

impl GhostSlotTrap {
    /// Bitmask capturing all slots 1 through 10.
    pub const ALL_SLOTS_MASK: i32 = 0x3FF;

    /// Generates the raw parameters needed for a ghost trap packet.
    #[inline]
    pub const fn trap_params(timeout_seconds: i32) -> (i32, i32, &'static str) {
        (Self::ALL_SLOTS_MASK, timeout_seconds, " \n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_driver_mapping() {
        let driver = SlotInputDriver;
        assert_eq!(driver.handle_slot(1), Some(MenuInputAction::DirectSlot(1)));
        assert_eq!(driver.handle_slot(8), Some(MenuInputAction::PrevPage));
        assert_eq!(driver.handle_slot(9), Some(MenuInputAction::NextPage));
        assert_eq!(driver.handle_slot(10), Some(MenuInputAction::Exit));
        assert_eq!(driver.handle_slot(11), None);
    }

    #[test]
    fn test_button_driver_rising_edge() {
        let driver = ButtonInputDriver;
        let prev = Buttons::new(0);
        let cur_fwd = Buttons::new(Buttons::IN_FORWARD);
        assert_eq!(
            driver.handle_buttons(cur_fwd, prev),
            Some(MenuInputAction::NavigateUp)
        );

        // Holding key: no rising edge
        assert_eq!(driver.handle_buttons(cur_fwd, cur_fwd), None);

        let cur_jump = Buttons::new(Buttons::IN_JUMP);
        assert_eq!(
            driver.handle_buttons(cur_jump, prev),
            Some(MenuInputAction::Select)
        );
    }

    #[test]
    fn test_ghost_slot_trap_bitmask() {
        let (mask, timeout, text) = GhostSlotTrap::trap_params(15);
        assert_eq!(mask, 0x3FF);
        assert_eq!(timeout, 15);
        assert!(!text.is_empty());
    }
}
