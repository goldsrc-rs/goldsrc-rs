# GoldSrc.rs Roadmap

## v0.1.0 — Foundation & FFI ✅

**Goal:** Establish the Cargo workspace, CI pipeline, and raw C FFI bindings so that a Rust
binary can be loaded by the GoldSrc engine at all.

- [x] Set up Cargo workspace and CI/CD (Windows `i686-pc-windows-msvc`, Linux `i686-unknown-linux-gnu`).
- [x] Collect reference headers in `references/` (HLSDK, `meta_api.h`).
- [x] Write `build.rs` for `goldsrc-sys` that generates Rust structs from C++ headers via `bindgen`.
- [x] Export entry-point functions `GiveFnptrsToDll` and `Meta_Attach` so Metamod can load our library.

## v0.2.0 — Metamod Backend & Engine Hooks ✅

**Goal:** Prove that Rust code can intercept real GoldSrc engine events through Metamod and interact
with the live server.

- [x] Wrap logging (`SERVER_PRINT`, `ALERT`). Server prints "Hello from Rust!" to console.
- [x] Wrap basic engine structures: `edict_t`, `entvars_t`, `CBaseEntity`.
- [x] Implement hooks for basic events (`DispatchSpawn`, `ClientConnect`, `ClientCommand`) via Metamod.
- [x] Build VTable-hook system (using offsets from ReHLDS/HamSandwich for Windows/Linux compatibility).

## v0.3.0 — WebAssembly Plugin Host ✅

**Goal:** Isolate plugin code inside a pure-Rust WASM sandbox with hot-reload, so a crashed plugin
can never bring down the server.

- [x] Integrate `wasmi` (pure-Rust interpreter runtime) into the core.
- [x] Design WASI / host bindings (`server_print`) for WASM plugins to communicate with the core.
- [x] Implement hot-reload: watch `.wasm` files in `addons/metamod-rs/plugins/` and reload on change.
- [x] Complete plugin lifecycle management (Create, Modify, Delete, Error handling, `on_unload` callback).

## v0.4.0 — Developer Framework & Host CLI ✅

**Goal:** Give plugin authors a productive developer experience: macros, SDK primitives, in-game CLI,
dependency management, event bus, and automated deployment.

- [x] Host Console Management CLI (`mrs`) with `lexopt`: `load`, `unload`, `reload`, `pause`, `list`, `info`.
- [x] `goldsrc-macros` crate with procedural macros (`#[plugin(systems=...)]`, `#[command]`).
- [x] SDK `goldsrc` crate with Flat / Hybrid ECS API for WASM plugins.
- [x] Plugin DAG dependency resolution with SemVer validation (`semver` crate).
- [x] Global Event Bus (Pub/Sub) for inter-plugin communication across WASM modules.
- [x] In-game & Console Command Router (`#[command]`, `dispatch_command`).
- [x] JSON/TOML configuration file watchers (`configs/` folder auto-reload & event broadcasting).
- [x] Automated deployment & post-deploy MD5 hash verification script (`deploy.py`).

## v0.5.0 — Framework Internals & High-Level DX ✅

**Goal:** Polish the host WASM FFI layer, implement ECS, and provide clean high-level Player/Entity
wrappers so plugin code reads like idiomatic Rust.

- [x] Refactor Host WASM FFI layer (`LoadedPlugin::invoke_two_slices` generic helper).
- [x] Implement `goldsrc` Flat ECS (Sparse-Set Entity Component System for WASM plugins).
- [x] High-level `Player` & `Entity` safe API wrappers with `Vector3`.
- [x] Granular Config Event System (`action`: `created`/`modified`/`deleted`) with per-plugin config isolation.
- [x] Host Logger Service with structured levels (`Trace`, `Info`, `Warn`, `Error`) and auto-created log dirs.

## v0.6.0 — Architecture Restructuring ✅

**Goal:** Reorganize the monolithic workspace into a layered `core / backends / framework` structure
and eliminate all unsafe initialization boilerplate from plugin authoring.

- [x] Reorganize workspace into `core/`, `backends/`, `framework/`, `tools/`.
- [x] Rename `metamod-rs` → `goldsrc-metamod`.
- [x] Optimize WASM payload size via Cargo `profile.release`.
- [x] Implement WASM Host Imports for safe Engine FFI boundary crossing.
- [x] Refactor `goldsrc-api` to provide `Player` / `Entity` structs with elegant methods for WASM.
- [x] Add `#[on_load]` procedural macro to eliminate `unsafe` initialization in plugins.

## v0.7.0 — Component Model & TOML Configuration ✅

**Goal:** Replace all raw `extern "C"` WASM bridges with the typed WASM Component Model,
switch to a central TOML config, and cut binary size by 90% through `wasm-opt`.

- [x] Transition `goldsrc-wasm-host` from `wasmi` to `wasmtime` with native JIT (Cranelift).
- [x] Adopt WASM Component Model (`wit-bindgen` & `wit-component`) to replace `unsafe extern "C"` bridges.
- [x] Implement centralized TOML configuration system (`goldsrc.toml`) with dynamic path resolution.
- [x] Integrate `wasm-opt` pipeline in `build.py` for 90% WASM payload size reduction (~200 KB).
- [x] Implement Capability-based Access Control system (RBAC) in host and SDK.
- [x] Purge `serde_json` in favor of zero-overhead TOML & Canonical ABI.

## v0.8.0 — Standalone Backend & Direct Engine Integration ✅

**Goal:** Eliminate the hard Metamod dependency by implementing a proxy GameDLL backend that loads
directly via `liblist.gam`, proving the architecture works without any third-party plugin loader.

- [x] Implement `goldsrc-standalone` backend (proxy GameDLL loaded via `liblist.gam` `gamedll` key).
- [x] Fix Memory Corruption in `GetNewDLLFunctions` (buffer size overflow into engine struct).
- [x] Fix Mutex re-entrancy deadlocks in proxy layer across forwarded engine callbacks.
- [x] Remove hardcoded developer paths; route all logging through `PathResolver`.
- [x] Universal GameDLL auto-detection (`mp.dll` / `cs.so`) with `GetEntityAPI2` / `GetEntityAPI` fallbacks.

---

## v0.9.0 — Core Refactoring, Panic Isolation & Host Separation ✅

**Goal:** Eliminate code duplication between backends, harden the FFI safety boundary so no Rust
panic can crash HLDS, introduce a production-grade structured logger, and cleanly separate backends from plugin hosts.

- [x] **Host / Backend Architecture Separation**: Segregated engine adapters (`backends/goldsrc-metamod`, `backends/goldsrc-standalone`) from plugin execution runtimes (`hosts/goldsrc-wasm-host`) with abstract `PluginHost` interface.
- [x] **Core Refactoring**: Move MRS CLI, plugin manager, command registration, and event hooks out of
  `goldsrc-standalone` and `goldsrc-metamod` into `framework/goldsrc`. Both backends become thin adapters.
- [x] **Panic Isolation**: Wrap every `#[no_mangle] pub unsafe extern "C"` export in `catch_ffi_panic` (`std::panic::catch_unwind`)
  to prevent Rust panics from crossing the C-ABI boundary and crashing HLDS.
- [x] **Safe Abstraction Layer**: All raw C pointers (`*mut edict_t`, `*const c_char`) wrapped in safe Rust
  types (`Entity`, `Player`, `CStr`/`String`). Plugin-facing API becomes fully `unsafe`-free with raw FFI isolated behind `unsafe-sys`.
- [x] **Unified Logger (`goldsrc_log`)**: Structured logger with categories (`Core`, `Proxy`, `Wasm`, `Plugin`)
  and levels (`Trace`, `Debug`, `Info`, `Warn`, `Error`). Controlled via `goldsrc.toml`:

  ```toml
  [logging]
  level = "debug"
  file_output = true   # -> cstrike/goldsrc/logs/
  console_output = true
  targets = ["core", "wasm"]
  ```

- [x] **Path Normalization**: Extend `PathResolver` with a unified normalization method (consistent separator
  across OS via `Path::display()` / `to_slash_lossy()`).
- [x] **Modularize backends**: Break `goldsrc-standalone` and `goldsrc-metamod` into clean component sub-modules.
- [x] **Centralized Project Toolchain**: Modular Python CLI (`__main__.py` with `setup`, `build`, `deploy`, `verify`, `pre-commit`, `analyze`, `logo`).
- [x] **Purge legacy C artifacts**: Remove `exports.def`, `metamod.def`, `wrapper.c` from `goldsrc-metamod`.

---

## v0.10.0 — Full HLSDK & WASM Host API Coverage 🏗 In Progress

**Goal:** Bring engine API coverage from ~15% to production completeness and expose the full GoldSrc
surface to WASM plugins through typed WIT interfaces.

- [ ] **Engine Functions (`enginefuncs_t`)**: Cover the 140+ engine functions
  (`pfnCreateNamedEntity`, `pfnMessageBegin`, `pfnRegUserMsg`, `pfnTraceLine`, `pfnPrecacheModel`, …).
- [ ] **DLL Hooks (`DLL_FUNCTIONS`)**: Expand from the current 5 to all 50+ GameDLL callbacks
  (`TraceAttack`, `PlayerKilled`, `Touch`, `Think`, `Use`, `PlayerPostThink`, …).
- [ ] **ReHLDS / ReGameDLL API**: Dynamic detection and optional binding to extended ReAPI interfaces.
- [ ] **WASM Bindings (WIT)**: Expose the expanded engine surface to plugins:
  - Console: `server_print`, `client_print`.
  - Commands: `pfnAddServerCommand`, per-player command hooks.
  - Entities: coordinates, health, model, weapon, team.
  - Damage & death: `TraceAttack`, `PlayerKilled`, damage multipliers.
  - Menus: `ShowMenu`, `pfnMessageBegin` / `pfnMessageEnd` wrappers.
- [ ] **Demo plugin validation**: Verify `vip_core` and `admin_system` on a live HLDS with the expanded API.

## v0.11.0 — Game-Specific Framework (CS 1.6) 📝 Planned

**Goal:** Layer CS 1.6–specific abstractions on top of the generic engine API so plugin authors
write game logic, not FFI glue.

- [ ] Split the SDK into a core engine module and game-specific extension crates.
- [ ] Create `goldsrc-cstrike` crate: CS 1.6 entities, weapons, buy zones, game events.
- [ ] Provide abstraction layers for game rules, map objectives, round state, and player states.
