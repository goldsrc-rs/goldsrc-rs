//! Declarative Administrator Menu orchestrator (`grs_adminmenu`).

use goldsrc::prelude::*;

pub const ACTION_ADM_RESTART_1: u32 = 3001;
pub const ACTION_ADM_RESTART_3: u32 = 3002;
pub const ACTION_ADM_PAUSE: u32 = 3003;
pub const ACTION_ADM_CFG_CW: u32 = 3004;
pub const ACTION_ADM_CFG_WARMUP: u32 = 3005;
pub const ACTION_ADM_MAP_CHANGE: u32 = 3006;
pub const ACTION_ADM_STAFF_LIST: u32 = 3007;

/// Builds the root administrator control menu using localized dictionary.
pub fn build_admin_main_menu_localized(lang: &str) -> Menu {
    let title = tr!("administration", lang, "menus.title");
    let item_slay = tr!("administration", lang, "menus.slay");
    let item_slap = tr!("administration", lang, "menus.slap");
    let item_teleport = tr!("administration", lang, "menus.teleport");
    let item_team = tr!("administration", lang, "menus.team");
    let item_map = tr!("administration", lang, "menus.map");

    Menu::builder(title)
        .style(MenuStyle::brackets())
        .item(MenuItem::new(item_slay, ACTION_ADM_RESTART_1).keep_open())
        .item(MenuItem::new(item_slap, ACTION_ADM_RESTART_3).keep_open())
        .item(MenuItem::new(item_teleport, ACTION_ADM_PAUSE).keep_open())
        .item(MenuItem::new(item_team, ACTION_ADM_CFG_CW).keep_open())
        .item(MenuItem::new(item_map, ACTION_ADM_MAP_CHANGE).keep_open())
        .build()
}

/// Builds the root administrator control menu with default language.
pub fn build_admin_main_menu() -> Menu {
    build_admin_main_menu_localized("common")
}
