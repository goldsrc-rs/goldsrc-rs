# goldsrc:menu_frontend

Central navigation hub, dynamic menu pagination, and debounced menu driver for GoldSrc.rs servers.

## Features

- **Central Navigation Hub (`/menu`, `grs_menu`)**: Aggregates modules into a unified player menu.
- **Multilingual Localization (i18n)**: Full English and Russian localization (`resources/lang/menu_frontend.toml`).
- **Dynamic Pagination**: Automatically splits items into pages based on `page_size`.
- **Anti-Spam Debounce**: Protects against rapid key-spam with configurable cooldown.
- **Auto-Discovery Registry**: Plugins register sections dynamically.

## Console Commands & Chat Triggers

| Command / Trigger | Capability | Usage | Description |
| :--- | :--- | :--- | :--- |
| `grs_menu` | `menu:open` | `grs_menu` | Opens the main server navigation menu |
| `menu`, `/menu`, `!menu`, `/help` | `menu:open` | `say /menu` | Chat alias triggers for server menu |

## Configuration (`menu_frontend.toml` / CVARs)

| CVAR | Type | Default | Range | Description |
| :--- | :--- | :--- | :--- | :--- |
| `grs_menu_title` | `String` | `"Главное Меню Сервера"` | - | Header title displayed atop main menu |
| `grs_menu_page_size` | `i32` | `7` | `1..=8` | Selectable items per menu page |
| `grs_menu_debounce_ms` | `i32` | `150` | `50..=1000` | Minimum interval in ms between button presses |
| `grs_menu_timeout_secs` | `i32` | `30` | `5..=300` | Auto-close timeout for idle menus |
