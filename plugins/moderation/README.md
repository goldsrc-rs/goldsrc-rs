# goldsrc:moderation

Production-ready moderation, punishment, and player discipline coordinator plugin for GoldSrc.rs servers.

## Features

- **Player Discipline Suite**: Ban, Unban, Kick, Gag, Mute, Slap, Freeze, and Inspection.
- **Multilingual Localization (i18n)**: Out-of-the-box English and Russian dictionaries (`moderation.toml`).
- **Policy-Based Access Control (PBAC)**: Fine-grained capabilities for every action (`moderation:action:ban`, `moderation:action:kick`, etc.).
- **Debounced Moderator Menu (`grs_modmenu`)**: Interactive UI with target selection.
- **Audit Logging**: Persists disciplinary actions with timestamps and player network identities.

## Console Commands

| Command | Capability | Usage | Description |
| :--- | :--- | :--- | :--- |
| `grs_slap` | `moderation:action:slap` | `grs_slap <#userid\|name> [damage]` | Slaps player with vertical impulse and damage |
| `grs_slay` | `moderation:action:slay` | `grs_slay <#userid\|name>` | Instantly eliminates target player |
| `grs_freeze` | `moderation:action:freeze` | `grs_freeze <#userid\|name> [seconds]` | Freezes target player velocity for inspection |
| `grs_gag` | `moderation:action:mute:chat` | `grs_gag <#userid\|name> [mins] [reason]` | Blocks player text and radio chat |
| `grs_mute` | `moderation:action:mute:voice` | `grs_mute <#userid\|name> [mins] [reason]` | Mutes player voice communications |
| `grs_kick` | `moderation:action:kick` | `grs_kick <#userid\|name> [reason]` | Disconnects player from the game session |
| `grs_ban` | `moderation:action:ban` | `grs_ban <#userid\|name> [mins] [reason]` | Bans player by SteamID/IP |
| `grs_unban` | `moderation:action:unban` | `grs_unban <auth_or_ip>` | Revokes active bans and gags for target |
| `grs_inspect` | `moderation:query:inspect` | `grs_inspect <#userid\|name>` | Prints player connection and state diagnostics |
| `grs_modmenu` | `moderation:ui:menu` | `grs_modmenu` | Opens the interactive moderator panel |

## Configuration (`moderation.toml` / CVARs)

| CVAR | Type | Default | Range | Description |
| :--- | :--- | :--- | :--- | :--- |
| `grs_mod_default_ban_mins` | `i32` | `60` | `0..=525600` | Default ban duration in minutes (0 = permanent) |
| `grs_mod_default_slap_damage` | `f32` | `0.0` | `0.0..=100.0` | Default slap damage inflicted on player |
| `grs_mod_max_freeze_secs` | `i32` | `120` | `5..=600` | Maximum freeze inspection duration |
| `grs_mod_notify_chat` | `bool` | `true` | - | Broadcast sanctions to server chat |
| `grs_mod_log_audit` | `bool` | `true` | - | Persist disciplinary logs into storage |
