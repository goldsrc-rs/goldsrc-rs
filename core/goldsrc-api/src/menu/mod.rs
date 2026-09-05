//! Declarative, multi-page Menu System for GoldSrc.rs.

pub mod action_registry;
pub mod builder;
pub mod session;
pub mod types;

pub use action_registry::{
    MenuActionHandler, MenuActionRegistry, clear_menu_actions, dispatch_menu_action,
    register_menu_action_id, register_menu_action_name,
};
pub use builder::{MenuBuilder, MenuPageBuilder};
pub use session::{
    PlayerMenuSession, clear_all_menus, close_menu, close_menu as close_player_menu,
    handle_menu_slot, on_round_start, open_menu, refresh_all_menus, refresh_player_menu,
};
pub use types::{
    AntiSpamAction, Condition, DenyAction, DenyPolicy, ExitBehavior, Feedback, ItemKind, ItemTitle,
    Menu, MenuContext, MenuItem, MenuRendererKind, MenuStyle, RenderedMenuPage, SlotAction,
    VisualDeny,
};

/// Standard maximum number of menu slots per page (1..=10).
pub const MAX_MENU_SLOTS: u8 = 10;

/// Default items per page in paginated menus (slots 1..=7).
pub const DEFAULT_ITEMS_PER_PAGE: usize = 7;

/// Menu slot index for navigating to the previous page (Slot 8).
pub const MENU_SLOT_BACK: u8 = 8;

/// Menu slot index for navigating to the next page (Slot 9).
pub const MENU_SLOT_NEXT: u8 = 9;

/// Menu slot index for exiting or closing the menu (Slot 0 -> 10).
pub const MENU_SLOT_EXIT: u8 = 10;

/// Bitmask representing all 10 menu keys active (`1..=10`).
pub const MENU_KEY_ALL: u16 = 0x3FF;

/// Safe payload chunk size for `ShowMenu` multipart network messages.
pub const MAX_SHOW_MENU_CHUNK_SIZE: usize = 150;
