# goldsrc:chat_director

High-performance chat management, anti-flood protection, and rotating broadcast director plugin for GoldSrc.rs servers.

## Features

- **Sliding-Window Anti-Flood**: Blocks chat spam with configurable rate-limiting thresholds.
- **Dedicated Staff Channel**: Intercepts `say_team @` and `say @` messages and routes them to online staff.
- **Periodic Broadcast Rotator**: Dispatches timed informational messages and DHUD announcements.
- **Multilingual Support (i18n)**: English and Russian localization (`resources/lang/chat_director.toml`).
- **Zero Allocations on Clean Messages**: Lightweight inspection passes clean messages directly to engine.

## Configuration (`chat_director.toml` / CVARs)

| CVAR | Type | Default | Range | Description |
| :--- | :--- | :--- | :--- | :--- |
| `grs_chat_flood_interval` | `f32` | `0.75` | `0.1..=10.0` | Minimum interval in seconds between messages |
| `grs_chat_broadcast_interval` | `f32` | `60.0` | `5.0..=600.0` | Interval between rotating announcements |
| `grs_chat_enable_staff_channel` | `bool` | `true` | - | Enable say @ prefix routing to staff |
| `grs_chat_enable_dhud_banners` | `bool` | `true` | - | Render announcements in Director HUD |
