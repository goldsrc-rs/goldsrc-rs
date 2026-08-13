# GoldSrc.rs Roadmap

## Stage 1: Foundation & FFI — ✅ Complete

- [x] Set up Cargo Workspace and CI/CD (build for `i686-pc-windows-msvc` and `i686-unknown-linux-gnu`).
- [x] Collect reference headers in `references/` (HLSDK, `meta_api.h`).
- [x] Write `build.rs` for `goldsrc-sys` that generates Rust structs from C++ headers via `bindgen`.
- [x] Export entry-point functions `GiveFnptrsToDll` and `Meta_Attach` so Metamod can load our Rust library.

## Stage 2: Metamod Backend & Safe Abstractions — ✅ Complete

- [x] Wrap logging (`SERVER_PRINT`, `ALERT`). Server should print "Hello from Rust!" to console.
- [x] Wrap basic engine structures: `edict_t`, `entvars_t`, `CBaseEntity`.
- [x] Implement hooks for basic events (`DispatchSpawn`, `ClientConnect`, `ClientCommand`) via Metamod.
- [x] Build VTable-hook system (using offsets from ReHLDS/HamSandwich for Windows/Linux compatibility).

## Stage 3: WebAssembly Plugin Host (Isolation & Hot-Reload) — ✅ Complete

- [x] Integrate `wasmi` (pure-Rust interpreter runtime) into the core.
- [x] Design WASI / host bindings (`server_print`) for WASM plugins to communicate with the core.
- [x] Implement hot-reload: watch `.wasm` files in `addons/metamod-rs/plugins/` and reload on change.
- [x] Complete plugin lifecycle management (Create, Modify, Delete, Error handling, `on_unload` callback).

## Stage 4: Developer Framework & Host CLI (DX) — ✅ Complete

- [x] Host Console Management CLI (`meta-rs` / `mrs` with `lexopt`, `-a/--all` flags, pagination, multi-target).
- [x] Create `goldsrc-macros` crate with procedural macros (`#[plugin(systems=...)]`, `#[command]`).
- [x] SDK `goldsrc` crate with Flat / Hybrid ECS API for WASM plugins.
- [x] Plugin DAG Dependency Resolution with SemVer validation (`semver` crate).
- [x] Global Event Bus (Pub/Sub) for inter-plugin communication across WASM modules.
- [x] In-game & Console Command Router (`#[command]`, `dispatch_command`).
- [x] JSON/TOML configuration file watchers (`configs/` folder auto-reload & event broadcasting).
- [x] Automated deployment & post-deploy MD5 hash verification script (`deploy.py`).

## Stage 5: Framework, ECS & High-Level DX — ✅ Complete

- [x] Refactor Host WASM FFI layer (`LoadedPlugin::invoke_two_slices` generic helper).
- [x] Implement `goldsrc` Flat ECS (Sparse-Set Entity Component System for WASM plugins).
- [x] High-Level Player & Entity Safe API wrappers (`Player`, `Entity`, `Vector3`).
- [x] Granular Config Event System (`action`: `created`/`modified`/`deleted`) with private per-plugin config isolation (`configs/plugins/<name>/`).
- [x] Host Logger Service with structured levels (`Trace`, `Info`, `Warn`, `Error`) and auto-created log directories (`logs/metamod-rs.log`).

## Stage 6: Architecture Refactoring & Elegant DX — ✅ Completed

- [x] Reorganize workspace into `core/`, `backends/`, `framework/`, `tools/`.
- [x] Rename `metamod-rs` to `goldsrc-metamod`.
- [x] Optimize WASM payload size via Cargo `profile.release`.
- [x] Implement WASM Host Imports for safe Engine FFI boundary crossing.
- [x] Refactor `goldsrc-api` to provide `Player` / `Entity` structs with elegant methods for WASM.
- [x] Add `#[on_load]` procedural macro to eliminate `unsafe` initialization.

## Stage 7: Component Model & TOML Architecture — ✅ Complete

- [x] Transition `goldsrc-wasm-host` from `wasmi` to `wasmtime` with native JIT execution engine.
- [x] Adopt WASM Component Model (`wit-bindgen` & `wit-component`) to completely replace `unsafe extern "C"` bridges.
- [x] Implement centralized TOML configuration system (`goldsrc.toml`) with dynamic path resolution.
- [x] Integrate `wasm-opt` pipeline in `build.py` for 90% WASM payload size reduction (~200KB).
- [x] Implement Capability-based Access Control system (RBAC) in host and SDK.
- [x] Purge `serde_json` in favor of zero-overhead TOML & Canonical ABI.

## Stage 8: Standalone Backend & Direct Engine Integration — ✅ Complete

- [x] Implement `goldsrc-standalone` backend bypassing Metamod dependency (proxy GameDLL via `liblist.gam`).
- [x] Fix Memory Corruption in `GetNewDLLFunctions` (buffer size overflow over engine struct).
- [x] Fix Mutex re-entrancy deadlocks in proxy layer across forwarded engine callbacks.
- [x] Remove hardcoded developer paths; route all logging through `PathResolver`.
- [x] Universal GameDLL auto-detection (`mp.dll` / `cs.so`) with `GetEntityAPI2` / `GetEntityAPI` fallbacks.
- [ ] Dynamic ReHLDS (`ReHLDS_Api`) & ReGameDLL (`ReGameDLL_Api`) detection with HLSDK fallback.
- [ ] Direct interception of `hlds.exe` / `hlds_linux` interfaces using reference headers.
- [ ] Modularize `goldsrc-wasm-host` and `goldsrc-metamod` crates into clean component sub-modules.
- [ ] Implement advanced declarative macros (`#[command]`, `#[hook]`) with State Injection (`&mut World`).

## Stage 9: Core Refactoring, Safe Abstraction & Unified Logger — 🏗 In Progress

Goal: eliminate code duplication between backends, harden FFI safety, and establish a production-grade
logging subsystem. After this stage both backends become thin adapters over a shared `framework/goldsrc` core.

- [ ] **Core Refactoring**: Move MRS CLI, plugin manager, command registration, and event hooks out of
  `goldsrc-standalone` and `goldsrc-metamod` into `framework/goldsrc`. Both backends become thin adapters.
- [ ] **Panic Isolation**: Wrap every `#[no_mangle] pub unsafe extern "C"` export in `std::panic::catch_unwind`
  to prevent Rust panics from crossing the C-ABI boundary and crashing HLDS.
- [ ] **Safe Abstraction Layer**: All raw C pointers (`*mut edict_t`, `*const c_char`) must be wrapped in safe
  Rust types (`Entity`, `Player`, `CStr`/`String`). Plugin-facing API must be fully `unsafe`-free.
- [ ] **Unified Logger (`goldsrc_log`)**: Implement a structured logger with categories (`Core`, `Proxy`, `Wasm`,
  `Plugin`) and levels (`Trace`, `Debug`, `Info`, `Warn`, `Error`). Controlled via `goldsrc.toml`:
  ```toml
  [logging]
  level = "debug"
  file_output = true   # -> cstrike/goldsrc/logs/
  console_output = true
  targets = ["core", "wasm"]
  ```
- [ ] **Path Normalization**: Extend `PathResolver` with a unified normalization method so all paths use a
  consistent separator regardless of OS (`Path::display()` / `to_slash_lossy()`).
- [ ] **Purge legacy C artifacts**: Remove `exports.def`, `metamod.def`, `wrapper.c` from `goldsrc-metamod`
  (obsolete C/MSVC-era files superseded by `#[no_mangle] pub unsafe extern "C"`).

## Stage 10: Full HLSDK & WASM Host API Coverage — 📝 Planned

Goal: bring engine API coverage from ~15% to production completeness (~80%+) and expose the full
GoldSrc surface to WASM plugins.

- [ ] **Engine Functions (`enginefuncs_t`)**: Systematically cover the 140+ engine functions
  (`pfnCreateNamedEntity`, `pfnMessageBegin`, `pfnRegUserMsg`, `pfnTraceLine`, `pfnPrecacheModel`, ...).
- [ ] **DLL Hooks (`DLL_FUNCTIONS`)**: Expand hook coverage from the current 5 to all 50+ GameDLL callbacks
  (`TraceAttack`, `PlayerKilled`, `Touch`, `Think`, `Use`, `PlayerPostThink`, ...).
- [ ] **ReGameDLL / ReHLDS API**: Optional detection and binding to extended ReAPI interfaces for modern
  CS 1.6 servers running on ReHLDS.
- [ ] **WASM Bindings (WIT)**: Expose the expanded engine & DLL surface to WASM plugins through typed
  WIT interfaces:
  - Console output: `server_print`, `client_print`.
  - Command registration: `pfnAddServerCommand`, player command hooks.
  - Entity access: edict coordinates, health, model, weapon, team.
  - Damage & death hooks: `TraceAttack`, `PlayerKilled`, damage multipliers.
  - Menu system: `ShowMenu`, `pfnMessageBegin` / `pfnMessageEnd` wrappers.
- [ ] **Demo plugin validation**: Verify `vip_core` and `admin_system` against the expanded API on a live HLDS.

## Stage 11: Game-Specific Framework (CS 1.6) — 📝 Planned

- [ ] Split the SDK into a core engine module and game-specific extensions.
- [ ] Create `goldsrc-cstrike` crate (CS 1.6 bindings) for specific entities, weapons, and game events.
- [ ] Provide abstraction layers for game rules, map objectives, round state, and player states.
