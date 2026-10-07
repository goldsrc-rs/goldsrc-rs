# goldsrc:map_manager

Intelligent map rotation, timelimit tracking, player nominations, and automated voting feature plugin for GoldSrc.rs servers.

## Features

- **Deterministic Timelimit Calculation**: Tracks elapsed round/map duration and queries `mp_timelimit`.
- **End-of-Map Automated Voting**: Triggers countdown poll `grs_map_vote_trigger_mins` prior to timelimit expiry.
- **Recent Maps Anti-Repetition**: Blocks recently played maps from nominations.
- **Multilingual Support (i18n)**: English and Russian dictionaries (`resources/lang/map_manager.toml`).
- **Engine Level Change (`changelevel`)**: Safely schedules engine map switch via `grs_nextmap`.

## Console Commands & Chat Triggers

| Command / Trigger | Capability | Usage | Description |
| :--- | :--- | :--- | :--- |
| `grs_timeleft`, `timeleft`, `/timeleft` | `map:query` | `say timeleft` | Displays formatted mm:ss remaining |
| `grs_currentmap`, `currentmap` | `map:query` | `say currentmap` | Displays currently running map |
| `grs_nextmap`, `nextmap` | `map:query` | `say nextmap` | Displays designated next map in rotation |
| `grs_maps`, `maps`, `/maps` | `map:query` | `say /maps` | Opens map nomination menu |
| `grs_mapvote_start` | `map:vote` | `grs_mapvote_start` | Forces immediate manual map vote |

## Configuration (`map_manager.toml` / CVARs)

| CVAR | Type | Default | Range | Description |
| :--- | :--- | :--- | :--- | :--- |
| `grs_map_vote_trigger_mins` | `f32` | `2.5` | `0.5..=30.0` | Minutes before map end to trigger vote |
| `grs_map_vote_duration_secs` | `i32` | `15` | `5..=120` | Duration in seconds for voting poll |
| `grs_map_block_recent_count` | `i32` | `3` | `0..=20` | Recently played maps excluded from rotation |
| `grs_map_default_pool` | `String` | `"de_dust2,..."` | - | Default map rotation pool |
