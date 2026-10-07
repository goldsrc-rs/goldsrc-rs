//! Declarative Moderator Menu orchestrator (`grs_modmenu`).

use goldsrc::prelude::*;

pub const ACTION_MENU_SLAP: u32 = 1001;
pub const ACTION_MENU_SLAY: u32 = 1002;
pub const ACTION_MENU_FREEZE: u32 = 1003;
pub const ACTION_MENU_GAG: u32 = 1004;
pub const ACTION_MENU_MUTE: u32 = 1005;
pub const ACTION_MENU_KICK: u32 = 1006;
pub const ACTION_MENU_BAN: u32 = 1007;
pub const ACTION_MENU_INSPECT: u32 = 1008;

pub const ACTION_TARGET_BASE: u32 = 2000; // 2000 + target_index (1..32)

/// Builds the root moderator control panel.
pub fn build_moderator_main_menu() -> Menu {
    Menu::builder("Модерация: Выбор Действия")
        .style(MenuStyle::brackets())
        .item(MenuItem::new("1. Slap (Толкнуть игрока)", ACTION_MENU_SLAP).keep_open())
        .item(MenuItem::new("2. Slay (Уничтожить игрока)", ACTION_MENU_SLAY).keep_open())
        .item(MenuItem::new("3. Freeze (Заморозить/Разморозить)", ACTION_MENU_FREEZE).keep_open())
        .item(MenuItem::new("4. Gag (Заблокировать текстовый чат)", ACTION_MENU_GAG).keep_open())
        .item(
            MenuItem::new("5. Mute (Заблокировать микрофон) [STUB]", ACTION_MENU_MUTE).keep_open(),
        )
        .item(MenuItem::new("6. Kick (Отключить от сервера) [STUB]", ACTION_MENU_KICK).keep_open())
        .item(MenuItem::new("7. Ban (Заблокировать доступ) [STUB]", ACTION_MENU_BAN).keep_open())
        .item(MenuItem::new("8. Inspect (Инспекция игрока)", ACTION_MENU_INSPECT).keep_open())
        .build()
}

/// Builds an interactive target selection menu for the specified action ID.
pub fn build_target_selection_menu(action_title: &str) -> Menu {
    let mut builder = Menu::builder(format!("Модерация: Выбор цели ({action_title})"));
    builder = builder.style(MenuStyle::brackets());

    for i in 1..=32 {
        let p = Player::new(i);
        if p.is_valid() {
            let name = p.name().unwrap_or_else(|| format!("Player #{i}"));
            let item_label = format!("{name} (#{i})");
            builder = builder.item(MenuItem::new(item_label, ACTION_TARGET_BASE + i as u32));
        }
    }

    builder.build()
}
