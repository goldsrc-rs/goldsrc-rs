# goldsrc:administration

Technical administration, staff hierarchy resolution, and match orchestrator plugin for GoldSrc.rs servers.

## Features
- **Staff Access Registry**: Hierarchical roles (`Moderator`, `Admin`, `SuperAdmin`, `HeadAdmin`).
- **Match Controls**: Pausing, unpausing, round restarts, and map changes.
- **Config Presets (`grs_exec`)**: Executes `.toml` and `.cfg` match configurations safely.
- **Multilingual Support (i18n)**: Full English and Russian localization (`resources/lang/administration.toml`).
- **Interactive Menu (`grs_adminmenu`)**: Fast in-game administrator interface.

## Console Commands
| Command | Capability | Usage | Description |
| :--- | :--- | :--- | :--- |
| `grs_who` | `admin:who` | `grs_who` | Displays online staff members and assigned roles |
| `grs_staff_reload` | `admin:root` | `grs_staff_reload` | Reloads `staff.toml` without restarting the server |
| `grs_map` | `admin:map` | `grs_map <mapname>` | Changes level immediately via safe server command |
| `grs_pause` | `admin:pause` | `grs_pause` | Toggles server technical pause |
| `grs_exec` | `admin:cfg` | `grs_exec <preset>` | Safely executes match or warmup configuration |
| `grs_adminmenu` | `admin:menu` | `grs_adminmenu` | Opens the administrator control menu |

## Configuration (`administration.toml` / CVARs)
| CVAR | Type | Default | Range | Description |
| :--- | :--- | :--- | :--- | :--- |
| `grs_adm_default_match_cfg` | `String` | `"clanwar.cfg"` | - | Default competitive match configuration |
| `grs_adm_default_warmup_cfg` | `String` | `"warmup.cfg"` | - | Default warmup configuration |
| `grs_adm_restart_delay_secs` | `i32` | `1` | `0..=60` | Delay in seconds for round restarts |
| `grs_adm_max_audit_fetch` | `i32` | `20` | `1..=1000` | Maximum audit records displayed |
| `grs_adm_notify_actions` | `bool` | `true` | - | Broadcast admin actions to server |
