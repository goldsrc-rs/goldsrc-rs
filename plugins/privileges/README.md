# goldsrc:privileges

High-performance VIP status, equipment distribution, and round perks feature plugin for GoldSrc.rs servers.

## Features

- **VIP Equipment Perks**: Spawn armor, grenade packs, assault weapon bundles (M4A1, AK-47, AWP, Deagle).
- **Round Restrictions**: Strict round gating (e.g. AWP unlocked from round 3) and per-round single-claim locks.
- **Passive Health Regeneration**: ECS-driven post-think tick regeneration with in-game toggle.
- **Full Multilingual Localization (i18n)**: English and Russian dictionaries (`resources/lang/privileges.toml`).
- **Interactive VIP Menu (`grs_privmenu` / `/vip`)**: Localized equipment selection.
- **Slot Reservation**: Evaluates spectator/non-VIP players for connection eviction on full servers.

## Console Commands & Chat Triggers

| Command / Trigger | Capability | Usage | Description |
| :--- | :--- | :--- | :--- |
| `grs_privmenu` | `vip.access` | `grs_privmenu` | Opens VIP equipment and perks menu |
| `vipmenu`, `/vip`, `!vip` | `vip.access` | `say /vip` | Chat alias triggers for VIP menu |
| `grs_vip_give` | `vip.manage` | `grs_vip_give <#userid\|name>` | Grants full VIP privileges to target player |
| `grs_vip_status` | `vip.access` | `grs_vip_status <#userid\|name>` | Prints player VIP active status |
| `grs_vip_heal` | `vip.heal` | `grs_vip_heal <#userid\|name>` | Heals living VIP player to 100 HP |
| `vip_slots_eval` | `vip.slot` | `vip_slots_eval` | Evicts spectator/non-VIP to liberate reserved slot |

## Configuration (`privileges.toml` / CVARs)

| CVAR | Type | Default | Range | Description |
| :--- | :--- | :--- | :--- | :--- |
| `grs_vip_bonus_hp` | `f32` | `100.0` | `0.0..=250.0` | Round starting HP provided to VIP players |
| `grs_vip_bonus_armor` | `f32` | `100.0` | `0.0..=250.0` | Round starting armor provided to VIP players |
| `grs_vip_regen_hp` | `f32` | `0.1` | `0.0..=10.0` | Health regeneration amount per tick |
| `grs_vip_auto_equip_round` | `i32` | `2` | `1..=20` | Minimum round number for auto equipment |
| `grs_vip_awp_round` | `i32` | `3` | `1..=20` | Minimum round before AWP weapon claim is unlocked |
| `grs_vip_chat_prefix` | `bool` | `true` | - | Display [VIP] prefix in chat |
| `grs_vip_reserved_slots` | `i32` | `2` | `0..=16` | Slots reserved for connecting VIP players |
