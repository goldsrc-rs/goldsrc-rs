//! Declarative VIP and Privileges Menu orchestrator (`grs_privmenu` / `/vip`).

use goldsrc::prelude::*;

pub const ACTION_VIP_ARMOR: u32 = 4001;
pub const ACTION_VIP_GRENADES: u32 = 4002;
pub const ACTION_VIP_M4A1: u32 = 4003;
pub const ACTION_VIP_AK47: u32 = 4004;
pub const ACTION_VIP_AWP: u32 = 4005;
pub const ACTION_VIP_DEAGLE: u32 = 4006;
pub const ACTION_VIP_TOGGLE_REGEN: u32 = 4007;

/// Builds the interactive VIP privileges menu.
pub fn build_vip_menu(round_number: u32) -> Menu {
    let mut builder = Menu::builder("VIP Меню Привилегий")
        .style(MenuStyle::brackets())
        .item(
            MenuItem::new("1. Комплект брони (+100 AP + Каска)", ACTION_VIP_ARMOR)
                .require_spec::<Alive>()
                .keep_open(),
        )
        .item(
            MenuItem::new(
                "2. Набор гранат (HE + 2 Flash + Smoke)",
                ACTION_VIP_GRENADES,
            )
            .require_spec::<Alive>()
            .keep_open(),
        )
        .item(
            MenuItem::new("3. Штурмовой комплект (M4A1 + Deagle)", ACTION_VIP_M4A1)
                .require_spec::<Alive>()
                .keep_open(),
        )
        .item(
            MenuItem::new("4. Штурмовой комплект (AK-47 + Deagle)", ACTION_VIP_AK47)
                .require_spec::<Alive>()
                .keep_open(),
        );

    // AWP sniper rifle restricted to round >= 3
    if round_number >= 3 {
        builder = builder.item(
            MenuItem::new("5. Снайперский комплект (AWP + Deagle)", ACTION_VIP_AWP)
                .require_spec::<Alive>()
                .keep_open(),
        );
    } else {
        builder = builder.item(
            MenuItem::new(
                format!("5. AWP [Заблокировано до 3 раунда (текущий: {round_number})]"),
                ACTION_VIP_AWP,
            )
            .keep_open(),
        );
    }

    builder
        .item(
            MenuItem::new("6. Пистолет Desert Eagle", ACTION_VIP_DEAGLE)
                .require_spec::<Alive>()
                .keep_open(),
        )
        .item(
            MenuItem::new(
                "7. Переключить авто-регенерацию HP",
                ACTION_VIP_TOGGLE_REGEN,
            )
            .keep_open(),
        )
        .build()
}
