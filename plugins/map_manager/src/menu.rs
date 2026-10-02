//! Interactive map voting and nomination menus.

use goldsrc::prelude::*;

pub const ACTION_VOTE_BASE: u32 = 6000; // 6000 + option_idx

/// Builds the interactive map voting ballot menu.
pub fn build_vote_menu(options: &[String]) -> Menu {
    let mut builder = Menu::builder("Голосование за следующую карту").style(MenuStyle::brackets());

    for (idx, map) in options.iter().enumerate() {
        let label = format!("{}. {}", idx + 1, map);
        builder = builder.item(MenuItem::new(label, ACTION_VOTE_BASE + idx as u32));
    }

    builder.build()
}

/// Builds the map nomination selection menu.
pub fn build_nomination_menu(maps: &[String]) -> Menu {
    let mut builder = Menu::builder("Номинация карт").style(MenuStyle::brackets());

    for (idx, map) in maps.iter().take(8).enumerate() {
        let label = format!("{}. {}", idx + 1, map);
        builder = builder.item(MenuItem::new(label, ACTION_VOTE_BASE + 100 + idx as u32));
    }

    builder.build()
}
