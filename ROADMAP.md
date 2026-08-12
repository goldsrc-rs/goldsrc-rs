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

## Stage 8: Standalone Backend & Direct Engine Integration — 🏗 In Progress

- [ ] Implement `goldsrc-standalone` backend bypassing Metamod dependency.
- [ ] Implement dynamic ReHLDS (`ReHLDS_Api`) & ReGameDLL (`ReGameDLL_Api`) detection with HLSDK fallback.
- [ ] Direct interception of `hlds.exe` / `hlds_linux` interfaces using reference headers.
- [ ] Modularize `goldsrc-wasm-host` and `goldsrc-metamod` crates into clean component sub-modules.
- [ ] Implement advanced declarative macros (`#[command]`, `#[hook]`) with State Injection (`&mut World`).

## Stage 9: Game-Specific Framework Extensions — 📝 Planned

- [ ] Split the SDK into a core engine module and game-specific extensions.
- [ ] Create `goldsrc-cstrike` (CS 1.6 bindings) for specific entities, weapons, and game events.
- [ ] Provide abstraction layers for game rules, map objectives, and player states.
