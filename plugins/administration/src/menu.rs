//! Declarative Administrator Menu orchestrator (`grs_adminmenu`).

use goldsrc::prelude::*;

pub const ACTION_ADM_RESTART_1: u32 = 3001;
pub const ACTION_ADM_RESTART_3: u32 = 3002;
pub const ACTION_ADM_PAUSE: u32 = 3003;
pub const ACTION_ADM_CFG_CW: u32 = 3004;
pub const ACTION_ADM_CFG_WARMUP: u32 = 3005;
pub const ACTION_ADM_MAP_CHANGE: u32 = 3006;
pub const ACTION_ADM_STAFF_LIST: u32 = 3007;

/// Builds the root administrator control menu.
pub fn build_admin_main_menu() -> Menu {
    Menu::builder("Администрация: Управление Сервером")
        .style(MenuStyle::brackets())
        .item(MenuItem::new("1. Рестарт раунда (1 сек)", ACTION_ADM_RESTART_1).keep_open())
        .item(MenuItem::new("2. Рестарт раунда (3 сек)", ACTION_ADM_RESTART_3).keep_open())
        .item(MenuItem::new("3. Техническая пауза матча [STUB]", ACTION_ADM_PAUSE).keep_open())
        .item(MenuItem::new("4. Загрузить Clanwar конфиг [STUB]", ACTION_ADM_CFG_CW).keep_open())
        .item(MenuItem::new("5. Загрузить Warmup конфиг [STUB]", ACTION_ADM_CFG_WARMUP).keep_open())
        .item(MenuItem::new("6. Смена карты [STUB]", ACTION_ADM_MAP_CHANGE).keep_open())
        .item(MenuItem::new("7. Список персонала сервера", ACTION_ADM_STAFF_LIST).keep_open())
        .build()
}
