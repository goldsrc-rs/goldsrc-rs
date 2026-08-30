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

## v0.10.0 — Rust 2024 Migration, Engine Bridge & WASM Plugin Ecosystem ✅

**Goal:** Modernize codebase to Rust 2024 Edition, implement pure-Rust WASM runtime with Pulley32, unify Standalone & Metamod backends with direct engine FFI, implement automatic precaching lifecycle, and demonstrate a full suite of functional in-game plugins.

- [x] **Rust 2024 Edition Migration**: Modernized entire workspace to Rust 2024 edition across all crates, resolving all lint rules and 2024 idioms.
- [x] **WASM Component Model with Pulley32**: Upgraded `goldsrc-wasm-host` to pure-Rust bytecode execution via Wasmtime Pulley32 for full 32-bit HLDS stability.
- [x] **Engine Bridge & String Pool Resolver**: Implemented safe `EngineOps` with native engine string table resolution (`pfnGetInfoKeyBuffer`/`pfnInfoKeyValue` and `pfnSzFromIndex`) preventing memory faults on string offsets.
- [x] **Automatic Resource Precaching**: Implemented thread-safe precache queue in `EngineBackend` executed during `hook_spawn` (`worldspawn`) for flawless audio/model precaching without engine panics.
- [x] **Direct Real-Time Console & Server Commands**: Integrated `pfnAddServerCommand` with synchronous print flush, providing real-time plugin CLI commands (`grs cmds`, `test_player`, `test_buff`, `test_sound`, `test_cvar`, `vipmenu`, `vip_add`, `vip_heal`, `vip_armor`, `admin_grant`, `admin_slay`, `admin_teleport`, `admin_gravity`).
- [x] **Functional Demo Plugin Suite**:
  - `test_suite`: ECS verification, player inspection (health, armor, origin, angles), CVar manipulation, sound playback.
  - `vip_core`: Dynamic capability authorization (`vip.access`), player buffing and healing.
  - `vip_menu`: Interactive VIP kit deployment with sound and visual feedback.
  - `admin_system`: Administration utilities (granting capabilities, slaying players, teleportation, gravity manipulation).

## v0.11.0 — Advanced Command Engine, Sandbox Hardening & Capability DSL ✅

**Goal:** Provide an ergonomic, declarative command system, a hierarchical capability DSL, resilient WASM sandbox interruption, and complete Metamod/AMX Mod X co-existence safety.

- [x] **Command Targets & Channels**:
  - Declarative routing for `Server`, `ClientConsole`, `Chat` (`say`, `say_team`), and `MessageMode` dialogs.
  - Silent chat triggers (e.g. executing `/vip` or `!vip` with engine message suppression via `MRES_SUPERCEDE`).
- [x] **Typestate Guards & Extractors**:
  - Type-driven precondition checks and typed extraction (`Player`, `Alive<Player>`, `FromArg` trait).
  - Auto-binding caller player index to target parameters when executing chat commands without positional args.
- [x] **Command Error Pipeline (`CommandResult`)**:
  - Typed error taxonomy (`AccessDenied`, `InvalidArguments`, `InvalidState`, `TargetNotFound`, `Cooldown`, `Custom`).
  - Declarative CLI specs (`CommandSpec`) with specialized per-command help (`grs <cmd> --help`, `grs help <cmd>`).
- [x] **Hierarchical Capability DSL**:
  - Rich Boolean grammar: namespaces (`admin.*`), wildcards, negation (`!admin.rcon`), logical combinators (`&`, `|`, `all_of!`, `any_of!`).
  - Fail-closed capability evaluation and eviction lifecycle.
- [x] **Runtime Command Builder API**:
  - Programmatic `Command::builder(...)` for dynamic runtime command registration.
- [x] **Adversarial Sandbox & ABI Hardening**:
  - Background daemon epoch timer (`increment_epoch` every 2ms) ensuring real wall-clock timeouts on infinite loops.
  - Wasmtime `StoreLimits` (64MB memory limit, 10k table elements).
  - Preserved Metamod shared memory (`mutil_funcs_t`) ensuring 100% stable co-existence with AMX Mod X.
  - Correct `MessageDest` discriminants and dynamic `SayText` user message lookup via `reg_user_msg`.
  - Panic barrier encapsulation (`catch_ffi_panic`) on entity factories.

## v0.12.0 — Declarative UI, Requirements DSL & Server Engine ✅

**Goal:** Expand declarative UI builders for HUD/Menus/Effects, unify plugin lifecycle state machine, introduce a unified Requirements DSL, and provide defensive server configuration.

- [x] **Declarative Multi-Page Menus & Renderers**:
  - Declarative `Menu::builder` with explicit page breaks, `ExitBehavior` (`PopParent`, `Close`), and dynamic action handlers (`#[menu_action]`).
  - Pluggable renderers (`ShowMenu` and `Dhud`).
- [x] **True Director HUD (DHUD) & Screen Effects**:
  - Full Director HUD wire format (`SVC_DIRECTOR` opcode 51 with `DRC_CMD_MESSAGE`) rendering smooth VGUI typography.
  - Classic 4-channel HUD (`SVC_TEMPENTITY` / `TE_TEXTMESSAGE`).
  - Screen effects: `ScreenFade` (damage flashes, flashbang blindness) and `ScreenShake` (tremors, explosions) with fluent builders.
- [x] **Unified Plugin Lifecycle FSM (`PluginStatus`)**:
  - State machine (`Loaded`, `Running`, `Paused`, `Blocked`, `Degraded`, `Poisoned`, `Unloaded`) replacing scattered boolean flags.
  - Automatic isolation and safe recovery on panics.
- [x] **Unified Requirements DSL (`require = [...]`)**:
  - Replaced legacy `dependencies` with a rich DSL: `plugin:<name>[@<ver>]`, `cvar:<name>[=<v>|!=<v>|>0]`, `feature:<name>`.
  - Dynamic runtime status recalculation (`Blocked` if missing, `Degraded` if paused).
- [x] **Defensive Server Configuration (`goldsrc.toml`)**:
  - Unified configuration with `[core]`, `[logging]`, `[watcher]`, and `[runtime]`.
  - Automated bounds clamping (`debounce_ms: [50, 5000]`, `memory: [16, 512] MB`, `tables: [100, 100k]`) with resilient fallbacks.
- [x] **Typed CVar Abstraction**:
  - Type-safe `Cvar<i32>`, `Cvar<f32>`, `Cvar<String>` and flags (`CvarFlags::ARCHIVE`, `NOTIFY`, `SERVER`, `READ_ONLY`).

## v0.13.0 — Reactive Rule Engine, Modular Bundles & Plugin Orchestration ✅

**Goal:** Build a unified, extensible Reactive Rule & Extension Engine (`Core + Pluggable Providers`) powering declarative lifecycle orchestration (`plugins.toml`), directory bundles, profile groups, and dynamic server conditions.

- [x] **Reactive Rule & Provider Engine (`goldsrc-api` & `framework/goldsrc`)**:
  - Generic `RuleEngine<Context>` with decoupled `RuleCondition` and `RuleAction` provider registries.
  - Built-in condition evaluators: `map` (patterns/wildcards), `players` (ranges/counts), `time` (server clock intervals), `cvar` (operators `==`, `!=`, `>`, `<`), `plugin_state`.
  - Built-in action executors: `pause`, `unpause`, `load`, `unload`, `enable_group`, `disable_group`, `set_cvar`, `exec`, `broadcast`.
  - Dynamic ad-hoc registration API allowing host modules and plugins to expose custom conditions and actions.
- [x] **Recursive Directory Bundles (`plugins/<bundle>/*.wasm`)**:
  - Recursive directory tree walking for plugin packs (e.g. `plugins/test_suite/test_hud.wasm`).
  - Recursive `notify` file system watching for instant hot-reloading across nested bundle subfolders.
- [x] **Declarative Plugin Orchestration (`plugins.toml`)**:
  - Fine-grained plugin controls: `enabled`, `priority`, and structured `debug` (logging levels, per-plugin log files, profiling).
  - Profile groups (`[groups.vip_pack]`, `[groups.match_mode]`) for instant multi-plugin toggling.
  - Reactive rule evaluations triggered on server lifecycle events (`ServerActivate`, `ClientConnect`, `ClientDisconnect`, `CvarChange`).
- [x] **Decomposed Micro-Plugins (`examples/demo_plugins`)**:
  - Split monolithic test plugins into clean, focused demonstration modules (`test_hud`, `test_menu`, `test_ecs`).

## v0.13.1 — Orchestration Polish & Map-Format Configuration ✅

**Goal:** Refine `plugins.toml` to support expressive Named Map headers (`[plugins.<name>]`, `[rules.<name>]`), fine-grained rule condition logic (`AND`/`OR`/`NOT`), and detailed pause reason tracking.

- [x] **Named Map TOML Configuration (`[plugins.<name>]`, `[rules.<name>]`)**:
  - Transition from array-of-tables `[[plugins]]` to clean named tables: `[plugins.admin_system]`, `[plugins.vip_core.debug]`.
  - Dual-format parser ensuring backward compatibility with array-of-tables syntax.
- [x] **Granular Pause Reason Tracking (`PluginStatus::Paused { reason }`)**:
  - Record the origin rule or group name that caused a plugin pause (displayed in `grs info <idx>` and `grs ls`).
- [x] **Boolean Condition Expressions for Reactive Rules**:
  - Support `all_of = [...]`, `any_of = [...]`, `none_of = [...]` (AND/OR/NOT logic) inside `when = { ... }` blocks.
- [x] **Direct Engine Live Player Tracker**:
  - Real-time slot-based player count queries (`pfnGetPlayerStats` / edict validation) for immediate rule triggering on connect/disconnect.

## v0.14.0 — Storage Engine (SQLite WAL & KV-Buckets) & Localization (i18n) ✅

**Goal:** Provide a high-performance, non-blocking storage architecture tailored for GoldSrc 1000 FPS servers (SQLite in WAL mode, MPSC background batching, typed `Bucket<T>`, and strict WASM isolation) alongside structured per-player i18n localization.

- [x] **Dual Storage Port Abstraction (`core/goldsrc-api`)**:
  - `trait StorageProvider` (KV port with `get`, `set`, and atomic `fetch_add`).
  - `trait SqlDatabase` (Query port for relational operations and rank/ELO aggregations).
  - Strongly typed `Bucket<T>` guest DX wrapper delegating to `StorageProvider` without redundant memory caching.
- [x] **Unified SQLite WAL Driver & Zero-Frame-Cost Runtime (`goldsrc-storage` / `framework`)**:
  - Embedded zero-config SQLite driver in WAL mode (`cstrike/data/goldsrc.db`) serving both `goldsrc_kv` and custom relational tables.
  - Zero-latency main-thread IO: writes dispatched via non-blocking `mpsc` channel to a background worker with 500ms batch flush.
  - Guaranteed transactional flush on `client_disconnect` and `ServerDeactivate`.
- [x] **Strict WASM Host Storage Sandbox & Bucket Access Control**:
  - Automatic `{plugin_id}/` prefix injection on all `host_storage_*` calls preventing cross-plugin data tampering.
  - Explicit bucket sharing via plugin metadata allowlist (`[goldsrc.share] buckets = ["global/ranks"]`).
- [x] **Domain Decomposition of Large Modules (1000+ LoC Refactoring)**:
  - Refactor `hosts/goldsrc-wasm-host/src/manager.rs` into `manager/` submodule (`loader.rs`, `lifecycle.rs`, `state.rs`, `watcher.rs`).
  - Refactor `framework/goldsrc/src/cli.rs` into `cli/` submodule (`router.rs`, `specs.rs`, `handlers.rs`).
  - Refactor `framework/goldsrc/src/backend.rs` into modular engine domain adapters (`engine_bridge.rs`, `print_queue.rs`).
- [x] **Per-Player Localization & i18n Dictionary Engine (`framework/goldsrc/src/i18n`)**:
  - Structured language dictionaries (`data/lang/*.toml`) with lexical variable scoping, color/macro expansions, and access controls.
  - `AsLangCode` trait, `player.lang()`, `I18nEngine::server_lang()`, and zero-boilerplate `tr!` macro.

## v0.15.0 — Gameplay Engine, Game-Specific SDK (`goldsrc-cstrike`) & Unified DSL / Chat 📝 Planned

**Goal:** Provide core gameplay hooks (`TakeDamage`, `Spawn`, `Killed`, `TraceAttack`), automated `gamedata.toml` offset generation, `goldsrc-cstrike` framework (Money, CS Teams, Defuse, Bomb), and unified Expression DSL / Placeholder engine with rich chat interception.

- [ ] **Unified GoldSrc Expression DSL & Placeholder Engine (`goldsrc_api::dsl`)**:
  - Unified zero-allocation AST/lexer powering Requirements, Capabilities, and Placeholders.
  - Procedural macro `#[placeholder(name = "...", usage = "...")]` with typed arguments (`{ip(target='Player')}`).
  - Built-in diagnostic suggestions ("Did you mean...?") and server CLI introspection (`goldsrc placeholders <plugin>`).
- [ ] **Chat Processing & Multi-Chunk SayText Router (`goldsrc_api::chat`)**:
  - Interceptor pipeline for `say` / `say_team` with dynamic color formatting and safe multi-chunk packet splitting exceeding 185 bytes.
- [ ] **Automated Gamedata Pipeline (`data/gamedata/*.toml`)**:
  - Offline/CLI gamedata generator and zero-crash memory signature validator with hot-patching.
- [ ] **VTable & Entity Hooking Engine (`TakeDamage`, `Spawn`, `Killed`)**:
  - Safe interceptors for `CBasePlayer` / `CBaseEntity` virtual tables.
- [ ] **Game-Specific Framework (`goldsrc-cstrike`)**:
  - High-level abstractions for Counter-Strike 1.6: `CsTeam`, `CsWeapon`, `Money`, `DefuseKit`, `Bomb`.

## v0.16.0 — ReAPI Direct Bridge & Advanced Physics 📝 Planned

**Goal:** Native zero-overhead integration with ReHLDS & ReGameDLL API, round events (`RoundStart`, `RoundEnd`), and raytracing physics.

- [ ] **ReAPI Dynamic Capability & Feature Detection**:
  - Direct C-ABI bridge to ReHLDS and ReGameDLL with graceful fallback on Vanilla HLDS.
- [ ] **Advanced Raytracing & World Geometry (`RayTrace`, `Hull`, `DropToFloor`)**:
  - Line-of-sight checks, custom entity physics, and hitbox intersections.

## v0.17.0 — Multi-Host Ecosystem (C#, Python, Dynamic DLLs) 📝 Planned

**Goal:** Support polyglot plugin development by dynamically loading external language runtimes (C# .NET, Python) from `hosts/` with strict C-ABI handshakes.

- [ ] **Dynamic Host Runtime Architecture**:
  - Modular `cstrike/goldsrc/hosts/` discovery directory with configurable resolution policy (`prefer_builtin` vs `prefer_external`).
  - C-ABI `PluginHostFactory` handshake with version validation.
- [ ] **C# Plugin Host (`goldsrc-csharp-host`)**:
  - Native AOT / .NET runtime embedding for high-performance C# GoldSrc plugins.
- [ ] **Python Plugin Host (`goldsrc-python-host`)**:
  - Python 3.x bindings with `@plugin`, `@command`, and `@event` decorators.
- [ ] **Multi-Version Host Isolation**:
  - Ability to run multiple versions or types of runtime hosts simultaneously on the same server backend.

## Future Milestones & Ecosystem Tools

### Ecosystem Plugins & Developer Tooling

- [ ] **`goldsrc-coreutils` (POSIX Shell & Diagnostic Tools for ReHLDS)**:
  - Integration of modular `uutils/coreutils` Rust crates (`uu_ls`, `uu_cat`, `uu_head`, `uu_tail`, `uu_wc`, `uu_grep`, `uu_sort`).
  - Sandboxed execution within the `cstrike/` root directory (Path Traversal protection & capability checks `admin.shell`).
  - Native console I/O streaming directly into server console, RCON, and client admin chat.
- [ ] **`grs` Dedicated CLI Tool (`cargo-goldsrc`)**:
  - Scaffolding commands: `grs new <plugin> [--bundle <bundle>] [--multi-bin]`.
  - Packaging & verification: `grs build`, `grs pack` (`.gsp` bundle format), and `grs lint`.
  - Plugin registry integration: `grs install <plugin>`, `grs update`.
