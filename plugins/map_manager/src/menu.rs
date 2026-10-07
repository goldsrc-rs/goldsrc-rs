//! Interactive map voting and nomination menus.

use goldsrc::prelude::*;

pub const ACTION_VOTE_BASE: u32 = 6000; // 6000 + option_idx

/// Builds the interactive map voting ballot menu localized for the specified language.
pub fn build_vote_menu_localized(options: &[String], lang: &str) -> Menu {
    let title = tr!("map_manager", lang, "menus.vote_title");
    let mut builder = Menu::builder(title).style(MenuStyle::brackets());

    for (idx, map) in options.iter().enumerate() {
        let label = format!("{}. {}", idx + 1, map);
        builder = builder.item(MenuItem::new(label, ACTION_VOTE_BASE + idx as u32));
    }

    builder.build()
}

/// Builds the interactive map voting ballot menu.
pub fn build_vote_menu(options: &[String]) -> Menu {
    build_vote_menu_localized(options, "common")
}

/// Builds the map nomination selection menu localized for the specified language.
pub fn build_nomination_menu_localized(maps: &[String], lang: &str) -> Menu {
    let title = tr!("map_manager", lang, "menus.nomination_title");
    let mut builder = Menu::builder(title).style(MenuStyle::brackets());

    for (idx, map) in maps.iter().take(8).enumerate() {
        let label = format!("{}. {}", idx + 1, map);
        builder = builder.item(MenuItem::new(label, ACTION_VOTE_BASE + 100 + idx as u32));
    }

    builder.build()
}

/// Builds the map nomination selection menu.
pub fn build_nomination_menu(maps: &[String]) -> Menu {
    build_nomination_menu_localized(maps, "common")
}
