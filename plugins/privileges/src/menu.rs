//! Declarative VIP and Privileges Menu orchestrator (`grs_privmenu` / `/vip`).

use goldsrc::prelude::*;

pub const ACTION_VIP_ARMOR: u32 = 4001;
pub const ACTION_VIP_GRENADES: u32 = 4002;
pub const ACTION_VIP_M4A1: u32 = 4003;
pub const ACTION_VIP_AK47: u32 = 4004;
pub const ACTION_VIP_AWP: u32 = 4005;
pub const ACTION_VIP_DEAGLE: u32 = 4006;
pub const ACTION_VIP_TOGGLE_REGEN: u32 = 4007;

/// Builds the interactive VIP privileges menu using player's preferred language.
pub fn build_vip_menu_localized(round_number: u32, lang: &str) -> Menu {
    let title = tr!("privileges", lang, "menus.title");
    let item_armor = tr!("privileges", lang, "menus.armor");
    let item_grenades = tr!("privileges", lang, "menus.grenades");
    let item_m4a1 = tr!("privileges", lang, "menus.m4a1");
    let item_ak47 = tr!("privileges", lang, "menus.ak47");
    let item_awp = tr!("privileges", lang, "menus.awp");
    let item_deagle = tr!("privileges", lang, "menus.deagle");
    let item_toggle = tr!("privileges", lang, "menus.toggle_regen");

    let mut builder = Menu::builder(title)
        .style(MenuStyle::brackets())
        .item(
            MenuItem::new(item_armor, ACTION_VIP_ARMOR)
                .require_spec::<Alive>()
                .keep_open(),
        )
        .item(
            MenuItem::new(item_grenades, ACTION_VIP_GRENADES)
                .require_spec::<Alive>()
                .keep_open(),
        )
        .item(
            MenuItem::new(item_m4a1, ACTION_VIP_M4A1)
                .require_spec::<Alive>()
                .keep_open(),
        )
        .item(
            MenuItem::new(item_ak47, ACTION_VIP_AK47)
                .require_spec::<Alive>()
                .keep_open(),
        );

    // AWP sniper rifle restricted to round >= 3
    if round_number >= 3 {
        builder = builder.item(
            MenuItem::new(item_awp, ACTION_VIP_AWP)
                .require_spec::<Alive>()
                .keep_open(),
        );
    } else {
        let restricted_label = tr!(
            "privileges",
            lang,
            "awp_restricted",
            round = 3,
            cur = round_number
        );
        builder = builder.item(MenuItem::new(restricted_label, ACTION_VIP_AWP).keep_open());
    }

    builder
        .item(
            MenuItem::new(item_deagle, ACTION_VIP_DEAGLE)
                .require_spec::<Alive>()
                .keep_open(),
        )
        .item(MenuItem::new(item_toggle, ACTION_VIP_TOGGLE_REGEN).keep_open())
        .build()
}

/// Builds the interactive VIP privileges menu with default language.
pub fn build_vip_menu(round_number: u32) -> Menu {
    build_vip_menu_localized(round_number, "common")
}
