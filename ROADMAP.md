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
  - `AsLangCode` trait, `player.lang()`, `I18nService::server_lang()`, and zero-boilerplate `tr!` macro.

## v0.15.0 — Architectural Layer Decomposition & Naming Standardization ✅

**Goal:** Cleanly decompose monolithic crates into layered generic architecture (`goldsrc-core`, `goldsrc-api`, `goldsrc-host-wasm`, `goldsrc-backend-standalone`, `goldsrc-backend-metamod`), eliminate Cargo feature entanglement across host/guest, and extract game-specific implementations into external crates.

- [x] **Unified Workspace Member Naming Scheme (`goldsrc-<category>-<name>`)**:
  - Rename `hosts/goldsrc-wasm-host` -> `hosts/goldsrc-host-wasm`.
  - Rename `backends/goldsrc-standalone` -> `backends/goldsrc-backend-standalone`.
  - Rename `backends/goldsrc-metamod` -> `backends/goldsrc-backend-metamod`.
- [x] **Pure Layer Separation (`core` / `hosts` / `backends` / `framework`)**:
  - Extract host orchestration from `framework/goldsrc` (`#[cfg(feature = "host")]`) into `core/goldsrc-core`.
  - Make `framework/goldsrc` a pure, lightweight SDK re-export for plugin developers without heavy engine dependencies.
  - Zero mod-specific logic in engine bridge (100% agnostic GoldSrc engine core).
- [x] **Standardized Workspace Dependencies & Version 0.15.0**:
  - Bump all crates to `0.15.0` with workspace inheritance.
  - Update deploy script and python tools to seamlessly compile and verify new targets.

## v0.16.0 — ReAPI Direct Bridge & Advanced Physics ✅

**Goal:** Native zero-overhead integration with ReHLDS & ReGameDLL API, engine-level capability detection, raytracing, and custom physics simulation.

- [x] **ReAPI Dynamic Capability & Feature Detection**:
  - Direct C-ABI bridge to ReHLDS and ReGameDLL with graceful fallback on Vanilla HLDS.
  - Expose extended ReGameDLL interfaces and direct memory structures where available.
- [x] **Advanced Raytracing & World Geometry (`RayTrace`, `Hull`, `DropToFloor`)**:
  - Line-of-sight checks, custom entity physics, BSP hull tracing, and hitbox intersections.
- [x] **Comprehensive 5-Phase ECS Verification & Pipeline Hardening**:
  - Multi-system integration tests covering `Validate` -> `Modify` -> `Execute` -> `React` -> `Monitor` execution sequences and topological ordering.

## v0.17.0 — Declarative Orchestration, Unified Layering & Phased DAG ✅

**Goal:** Eliminate magic priority numbers across the ecosystem, establish a deterministic phased topological dependency engine (`PhasedDag`), unify macros/builders/imperative registries, and enforce pure game-agnostic layer separation.

- [x] **Unified 3-Tier Layering Architecture (`macros -> builders -> imperative registries`)**:
  - Implemented runtime thread-safe registries for `CommandRegistry`, `EventRegistry`, `PlaceholderRegistry`, and `MenuActionRegistry`.
  - Added fluent builders with terminal `.register()` and `.subscribe()` methods.
  - Procedural macros (`#[command]`, `#[event]`, `#[menu_action]`, `#[system]`) desugar into dynamic registrations executed during `Guest::on_load()`.
- [x] **Pure Game-Agnostic Core Decoupling**:
  - Decoupled CS 1.6 domain layer into external `goldsrc-game-cstrike` repository.
  - Decoupled `goldsrc-api` and `goldsrc` framework from `goldsrc-sys` via optional `unsafe-sys` feature flag.
  - Extracted `vip_menu` demo plugin into `goldsrc-game-cstrike`.
- [x] **Universal Phased DAG Ordering Engine (`PhasedDag<P, Id, T>`)**:
  - Linear phase stratification with deterministic Kahn topological ordering and stable tie-breaking (`Phase` -> `Declaration Order` -> `Alphabetical ID`).
  - Compiler-grade diagnostics for cycle detection (`CycleDetected`), phase ordering conflicts (`PhaseConflict`), and missing dependencies (`MissingDependency`).
- [x] **Declarative Plugin Orchestration (`plugins.toml`)**:
  - Complete elimination of integer `priority: i32`.
  - Introduction of architectural layers (`tier = "core" | "service" | "gameplay" | "addon" | "analytics"`) and canonical dependency declarations (`requires = [...]`).
- [x] **Semantic Event Phases & Commutative State Contexts**:
  - Transition event subscriptions from numeric priorities to semantic phases: `EventPhase::Filter` -> `EventPhase::Handle` -> `EventPhase::Observe`.
  - Commutative accumulators (`add_bonus`, `add_multiplier`, `add_reduction`, `cancel`) and typed blackboard property bags preventing mutation conflicts across independent plugins.
- [x] **System Taxonomy & Architectural Role Standardization (`ARCHITECTURE.md`)**:
  - Formalize canonical component roles (`Engine`, `Orchestrator`, `Manager`, `Registry`, `Service`, `Dispatcher`, `Router`, `Bridge`) in `ARCHITECTURE.md`.
  - Decouple `HostRuntime` by extracting `RuleOrchestrator`, `NetworkMessageDispatcher`, and `PluginOrchestrator`.
  - Extract dedicated `CommandRegistry` from `goldsrc-host-wasm::PluginManager`.
  - Rename `I18nEngine` -> `I18nService` across workspace.
- [x] **CLI UX Modernization & Operation Status Protocol**:
  - Humanize all CLI messages in `grs` (`successfully paused`, `resumed`, `already active/paused` idempotency warnings, bounded index validation).
  - Introduce structured status markers (`Success`, `Notice`, `Warning`, `Error`) across interactive CLI responses and audit logs.
  - Informative command dispatch feedback when target plugin is in `Paused` or `Poisoned` status.
- [x] **Unified Template & Placeholder Formatting Engine**:
  - Configurable logging format in `goldsrc.toml` utilizing strict `<source>:<placeholder>` notation (e.g. `format = "[{log:date-time}][{log:level}][{log:target}] {log:message}"`).
  - Core foundation: provide `PlaceholderRegistry` and `NetworkMessageDispatcher` primitives for guest plugins; delegate high-level chat templating, HUD overlays, and external Discord webhooks to modular plugins (`chat_manager.wasm`, `hud_display.wasm`, `discord_notifier.wasm`).
- [x] **Scoped Edge-Triggered Rule Orchestration (`RuleOrchestrator`)**:
  - Decouple reactive rule evaluation into `RuleOrchestrator` with open `RuleScope` trigger tagging.
  - Edge-triggered transition detection (`rule_states`) and `manual_overrides` tracking preventing automatic re-evaluation from overriding intentional admin commands.
- [x] **Centralized Watcher Subsystem & Hierarchical CLI Reorganization**:
  - Extract centralized `WatcherService` into `core/goldsrc-core`, completely freeing `goldsrc-host-wasm` from `notify` dependency.
  - Implement `WatchTarget` Value Object (`File` vs `Directory`), multi-strategy `WatcherFilter` (`Any`, `Extension`, `Stem`, `ExactName`, `Pattern`), and per-watcher debounce windows.
  - Reorganize CLI under clean hierarchical namespaces: `grs plugins <list|info|load|unload|reload|pause|unpause|cmds>` and `grs watchers <list|pause|resume>` with zero legacy aliases.

## v0.18.0 — Engine Core Parity, Bundle Architecture & Hierarchical PBAC 📝 Planned

**Goal:** Bridge the core gap with native engine runtime capabilities (Identity, Combat, UserMessages, Timers, CVARs), establish a secure Bundle Component Model with FS sandboxing, deploy an ergonomic RBAC-on-PBAC access control system, stabilize runtime boundaries, and implement foundational standard plugins (`admin_system`, `vip_core`).

### 1. Engine Core Mechanics & FFI Bridge Parity

- [x] **Runtime Boundary & FFI Stability**:
  - Export `host-time` in WIT component interface, binding mono uptime to host high-precision ticks to prevent WASI `Instant::now()` trapping.
  - Refactor `guard.rs` into `panic_barrier.rs` (clarifying role as an FFI panic boundary rather than engine crash protector).
  - Graceful console shutdown handler on Windows/Linux trapping `Ctrl+C` / `SIGINT` and invoking engine `exit\n` to prevent `tier0` `Illegal termination of worker thread 'CFileWriterThread'`.
  - Console encoding alignment: configure `SetConsoleCP(65001)` alongside `SetConsoleOutputCP(65001)` on Windows with automatic CP1251/UTF-8 input decoding for admin console commands.
- [x] **Real Network Identity & Session Tokens**:
  - Wire `pfnGetPlayerAuthId`, `pfnGetPlayerUserId`, and `pfnInfoKeyValue` (IP address) to `EngineBridge` and `PlayerIdentity`.
  - Connect dynamic template placeholders `{player:ip}`, `{player:authid}`, `{player:userid}` without mock fallbacks.
  - Introduce thread-safe, generational `PlayerSession` tokens immune to Slot Recycling Hazards.
- [x] **Frame-Driven Task & Timer Service (`TimerService`)**:
  - Implement tick-accurate `TimerService` inside `HostRuntime` driven by `on_server_frame` and `gpGlobals->time` / frame counters.
  - Dual scheduling modes: continuous game time (`Duration`) and discrete physics frame intervals (`Ticks(u64)`).
  - Replace blocking/panicking threads in WASM with safe client-bound builders: `task::after(Ticks(1) | Duration).spawn(...)` and `task::every(Ticks(64)).bound_to(session).spawn(...)` supporting next-frame deferrals and pause-resilient timers.
- [x] **Reactive CVAR & Config Adapter Engine (`ConfigModel` & `#[cvar]`)**:
  - Unify CVARs and TOML files as decoupled I/O adapters over a single typed `PluginConfig` source of truth.
  - Full FFI support for `pfnCVarRegister` with typed values (`Cvar<T>`), IDE-friendly `CvarFlags` bitmasks, and reactive `.on_change(|old, new| ...)` observer hooks.
  - Bi-directional reactive synchronization: console/RCON mutations update in-memory state and disk TOML; disk changes update engine `cvar_t` via FFI without map restarts.
  - Self-describing schema exports: automatic generation of documented `.toml` templates, engine `.cfg` files (`to_cvars`), and admin Markdown documentation.
- [x] **Engine Entity Lifecycle & Typed Builder (`EntityBuilder`)**:
  - Universal `Entity::builder(classname)` with mandatory `pfnKeyValue` parameterization prior to `pfnSpawn`.
  - Type-safe enumerations and constants for common keys (`keys::TARGET_NAME`, `RenderMode`, `SolidType`).
- [x] **Combat Hooking & User Message Interception (`CombatBridge` & `UserMessageDispatcher`)**:
  - Dual-tier interception for `TakeDamage` and `Killed`: ReGameDLL API hooks (Tier 1) with VTable virtual hook fallback (Tier 2).
  - Complete replacement of legacy AMXX forwards with phased event pipelines (`Filter` -> `Handle` -> `Observe`) with commutative modifiers.
  - Connect engine user messages (`pfnMessageBegin`, `Write*`, `pfnMessageEnd`) enabling plugins to observe and mutate `ScreenFade`, `DeathMsg`, `CurWeapon`, and `Damage`.
- [x] **Extended Edict Properties (`pev` / `entvars_t`)**:
  - Expand safe property triad (`get`/`set`/`modify`) with `Buttons` (`PlayerButtons` bitflags `IN_*`), `Flags` (`FL_ONGROUND`, `FL_DUCKING`), `MaxSpeed`, `Gravity`, `RenderEffect`, and `Model`.

### 2. Autonomous Bundle Architecture & Sandbox Isolation

- [x] **Role-Based Component Model (`ComponentRole`)**:
  - Taxonomy: `Coordinator` (root public facade, max 1 per bundle), `Service` (persistence/state), `Feature` (gameplay hooks), `Ui` (menu/chat), `Peer` (symmetric participant).
  - Baked `.goldsrc.component.role` custom section metadata in WASM with `#[plugin(role = ...)]`.
  - Handshake validation: graceful degradation to safe binary defaults with clear warnings if `bundle.toml` overrides violate binary capabilities.
- [ ] **Decoupled Inter-Bundle Communication & Service Gateway**:
  - Complete elimination of legacy AMXX natives: replace with zero-cost shared WIT component model linking intra-bundle.
  - Inter-bundle: decoupled `BundleMessageBroker` request/response channels with contract versioning (`economy.v1`) and `ServiceUnavailable` resilience.
- [x] **Strict Filesystem Sandboxing (WASI Preopens)**:
  - Strict path traversal prevention (`..` blocks).
  - Absolute bundle isolation: write access jailed to `addons/goldsrc/data/<bundle_name>/`, read-only configs to `configs/<bundle_name>/`. Zero access outside the server root.
- [x] **Self-Healing Autonomous Configuration Engine**:
  - Decentralized config ownership: Host (`goldsrc.toml`), Bundle (`bundles/<name>.toml`), and Plugin (`plugins/<name>.toml`).
  - Zero-initial-config server deployment: auto-generation of missing default configs on boot with deep-merge schema updates and non-destructive preservation of admin edits.
- [x] **Deployment Layout Standardization (`bin` -> `lib`)**:
  - Rename backend binary deployment path from `bin/` to `lib/` (`addons/goldsrc/lib/` and `goldsrc/lib/`) for strict semantic alignment with shared libraries (`.dll`/`.so`) and game hosting standards.
- [x] **Separation of Plugins and Examples**:
  - Move production-grade plugins (`admin_system`, `vip_core`) into dedicated `plugins/` workspace.
  - Maintain `examples/` strictly for SDK capability demonstrations (`test_chat`, `test_ecs`, `test_hud`, `test_i18n`, `test_menu`).

### 3. Hierarchical PBAC, Granular DSL & Modular Administration

- [x] **Orthogonal Capability Namespaces & Generic Access Scopes**:
  - Elimination of leaky abstraction names (`chat:admin_say` -> `chat:channel(admin)`, `menu:vip` -> `menu:scope(vip)`).
  - Unification under root namespace containers (`engine:*`, `system:*`, `chat:*`, `menu:*`, `gameplay:*`, `bundle:<id>:*`).
  - System ECS stages & phases guarded by capability checks (`system:stage.post_think.modify`).
- [x] **Compiler-Grade Capability DSL Semantic Validator**:
  - Enforce strict separator semantics: `:` for namespace roots, `.` for hierarchical path traversal, `()` for parametric arguments (`gameplay:heal(max=150)`).
  - Support `&` and `|` boolean operators inside Group Expansions `[...]` with `,` retained as conjunctive shorthand.
  - Explanatory compiler warnings with reconstructed AST pretty-printing upon ambiguous grouping (e.g. `"... to clarify precedence. Treated as: (A & B) | C"`).
- [ ] **Modular Administration Ecosystem (`admin_system` & `vip_core` PoC)**:
  - Preserve engine core purity: implement admin capabilities, bans, slaps, and voting purely as an external WASM plugin (`admin_system`) instead of hardcoding into `goldsrc-core`.
  - Implement VIP equipment, round start hooks, and capabilities in `vip_core`.
  - Type-safe role aliases (`AdminCaps`, `VipCaps`) mapping to composite capability sets.

---

## v0.19.0 — Modular Engine Extensions, Abstract UI Renderers & Hardware Diagnostics ✅
 
**Goal:** Decouple engine-specific modifications (ReAPI, ReHLDS, ReGameDLL, Xash3D) into dynamic `EngineExtension` modules with DSL requirements (`ext:<name>`), introduce a full MVC in-game UI system (`MenuRenderer` + `MenuInputDriver`) with Ghost Slot Trapping, rich `messagemode` text inputs, and provide a host Hardware Inspector.
 
### 1. Modular Engine Extensions (`trait EngineExtension`)
 
- [x] **Engine Extension Architecture**:
  - Extract engine-specific C-ABI hooks out of core runtime into modular `EngineExtension` providers.
  - Separate Metamod adapter into pure **Transport Backend** and optional **Metamod Extension**.
  - Dynamically discoverable extension registry with graceful fallback: if running on Vanilla HLDS or Xash3D, runtime gracefully disables features without crashing.
- [x] **ReAPI Subsystem as an Extension (`goldsrc-ext-reapi`)**:
  - Encapsulate `IRehldsApi` and `IReGameApi` into `goldsrc-ext-reapi`.
  - Expose extended memory offsets, custom entity hooks, and ReGameDLL-specific game events to the SPI.
- [x] **DSL Extension Requirements (`ext:<name>`)**:
  - Extend plugin dependency DSL to support `ext:<name>[@<version>]` requirements (e.g. `require = ["ext:reapi@>=5.21.0"]`).
  - Automatic FSM state management: plugins requiring missing extensions transition safely to `PluginStatus::Blocked` instead of throwing runtime panics.
 
### 2. In-Game Menu Architecture & Renderers

- [x] **Native Engine Renderers**:
  - `ClassicMenuRenderer`: standard Half-Life `ShowMenu` formatted text pages (slots 1..9, 0) with colors (`\w`, `\y`, `\r`, `\d`) and automatic pagination.
  - `DhudMenuRenderer`: high-fidelity Director HUD overlay.
- [ ] **Rich Client GUI & Custom Shaders**:
  - Deferred to sovereign client horizon (`wgpu` / client-side WebAssembly runtime) where input trapping and raw mouse clicks are natively supported.
- [x] **Menu Action Model & Dynamic Dispatch**:
  - Structured `MenuItem` with action callback bindings and typestate requirements (`MenuItem::require_spec`).
  - Native `messagemode` integration: captures user text and seamlessly returns to previous menu page.

### 3. Host Hardware Inspector & System Diagnostics

- [x] **Hardware Telemetry Provider (`SystemInfoService`)**:
  - Host-side system metrics collection via `sysinfo` exposed through SPI to WASM plugins.
  - Real-time CPU detection (vendor, model, logical/physical core allocation, CPU load %).
  - Memory statistics (allocated RAM to HLDS process, free system RAM, swap).
  - Frame time jitter and engine tickrate stability monitoring (measuring deviation from 1000 FPS).
- [x] **Admin System Inspection Tool (`system_monitor.wasm` / `admin_system`)**:
  - Host audit console command (`grs hardware`) enabling administrators to verify VPS/cloud hosting resource claims and detect overselling or throttling.

---

## v0.20.0 — Ecosystem Decomposition, Zero-Dep SPI & Core Sovereignty 🚧 In Progress

**Goal:** Invert and purify architecture dependencies via zero-dep `goldsrc-spi` (DIP), eliminate all legacy AMXX assumptions, establish memory and arithmetic invariants, lay the groundwork for pipeline console utilities (`uutils`), and decouple the monorepo.

### 1. Zero-Dep SPI & Domain Foundation

- [x] **Service Provider Interface Decoupling (`goldsrc-spi`)**:
  - Remove all dependencies from `goldsrc-spi` on `goldsrc-api`, establishing a clean DAG.
  - Relocate core value objects (`SteamId`, `PlayerGuid`, `murmur3_128`, `AuthIdentity`, `AuthSubject`, `AuthState`, `PlayerIdentity`, `PlayerSessionToken`, `CvarFlags`, `CvarEngine`, `EntitySpawner`) into `goldsrc-spi`.
  - Re-export all foundational types in `goldsrc-api` for seamless backward compatibility.
- [x] **Memory Governance, Signal Safety & Arithmetic Correctness**:
  - Fixed `Vector3::sub_assign` vector math copy-paste bug (`-= rhs.z`).
  - Standardized `Health` arithmetic operators to preserve clamping invariants and finite floats while allowing mod-friendly overrides (e.g. Zombie Plague 5000 HP).
  - Enforced terminal `_exit(128 + sig)` on fatal POSIX signals (`SIGSEGV`, `SIGBUS`, `SIGILL`) on Linux to prevent infinite recursion loops.
  - Wrapped console spew hook callbacks in `catch_unwind` FFI panic barrier.
  - Enforced `user_id` consistency validation in `SessionManager::validate_token`.
- [x] **Core Sovereignty & AMXX Purity**:
  - Completely purged AMX Mod X assumptions, configuration keys, and command relics (`amx_sysinfo`) from core specifications.
  - Removed speculative Menu MVC dead code (`driver.rs`, `widgets.rs`, `GhostSlotTrap`).

### 2. Console Streaming Tooling & Command Pipelines

- [ ] **Multi-Command Server Utilities (`uutils / coreutils.wasm`)**:
  - Embed lightweight streaming utilities for server console inspection: `cat`, `grep`, `tail`, `ls`, `wc`.
  - Compile utilities as a single multicall WASM plugin or native host extensions.
- [ ] **Console Pipeline Preprocessor**:
  - Server console piping support (`|`) connecting stdout of one command to stdin of another.
- [ ] **Hierarchical Command Registry**:
  - Tree-based command router replacing monolithic matching in `router.rs`.
- [ ] **Smart Preset Execution & State Snapshot Engine (`grs exec / grs_exec`)**:
  - **Infrastructure as Code (IaC) for Game Servers**: declarative presets (`presets/<mode>.toml` / `.kdl`) replacing naive sequential `.cfg` execution.
  - **Atomic Validation & Bounds Clamping**: validate full preset configuration, cvar ranges, and plugin requirements before mutation to prevent half-broken server states.
  - **State Snapshots & Reversible Rollback**: snapshot modified cvars and plugin states (`grs exec --restore`) for clean returns to base public mode after CW/Overtime matches without map restarts.
  - **Reactive Orchestration Event**: emit `ModeChanged { from, to }` across EventBus notifying `menu_frontend`, `chat_director`, and gameplay plugins to adapt UI and rules.
  - **WASM Bridge**: export `host-config-exec(preset_path: string) -> result<_, string>` to `goldsrc.wit` enabling single-click mode switching from `administration` menus.

### 3. Monorepo & Ecosystem Decomposition

- [ ] **`goldsrc` (Pure Plugin SDK)**:
  - Lightweight, zero-native-dependency SDK crate publishable to crates.io targeting `wasm32-wasip1`.
- [ ] **`goldsrc-runtime` (Host Engine & Platform)**:
  - Host execution container including `goldsrc-core`, `goldsrc-host-wasm`, and backend loaders (`standalone`, `metamod`).
- [ ] **`goldsrc-plugins-standard` (Standard Reference Plugins)**:
  - Standalone repository of production-grade plugins (`admin_system`, `vip_core`, `chat_manager`, `menu_system`, `map_chooser`, `stats_core`).

---

## v0.21.0 — Network & Threat Intelligence (`grlg-geo`) 📝 Planned

**Goal:** Integrate the standalone zero-copy `grlg-geo` threat intelligence engine directly into the GoldSrc host networking layer for sub-microsecond player classification, proxy/VPN mitigation, and connection screening.

### 1. Host Network Screening & Zero-Copy GeoIP

- [ ] **Host `mmap` Database Resolver**:
  - Integrate pure-Rust `GrlgReader` into `goldsrc-core` with memory-mapped zero-heap lookups (<1 µs latency).
  - Background asynchronous updates: hot-reload database memory maps without server hitch or player disconnects.
- [ ] **Threat Bitmask Pipeline in `ClientConnect`**:
  - Immediate bitflag screening during initial handshake: `is_datacenter`, `is_proxy`, `is_botnet`, `is_spam`.
  - Declarative connection policies in `goldsrc.toml`: `block_vpn`, `block_datacenter`, `allow_countries`.
- [ ] **SPI Network Filter Extension**:
  - Expose `GeoRecord` (Country, Region, City, Coordinates, ASN, ISP, ThreatFlags) through SPI to WASM and Native plugins.

---

## v0.22.0 — Next-Gen Demo Subsystem (`goldsrc-demo`) 📝 Planned

**Goal:** Design an event-driven, tamper-proof demo container format inspired by modern esports engines (CS2), featuring Zstandard stream compression, Ed25519 digital signatures, and backward compatibility with downstream analyzers.

### 1. CS2-Style Event-Driven Container

- [ ] **Structured Event Markers**:
  - Round boundaries (`RoundStart`, `RoundEnd`, `FreezePeriodEnd`).
  - Match economy & objectives (bomb plant/defuse, hostage rescue, weapon buy/drop).
  - High-precision killfeed and damage matrices with tick-accurate player origins and hitgroup indices.
- [ ] **Lossless Zstandard (`zstd`) Stream Compression**:
  - On-the-fly chunk compression shrinking 20–30 MB raw demos to 4–7 MB.
- [ ] **Cryptographic Signing (Ed25519)**:
  - Asymmetric cryptographic signing of demo headers and keyframe checkpoints to prevent post-game tampering or spoofed replays.
- [ ] **Streaming & Client-Server Relay**:
  - Bi-directional demo streaming: automatic client-to-server replay upload on player bans or in-game cheat reports.
- [ ] **Payload Backward Compatibility**:
  - Preserve 100% backward compatibility for the raw inner Half-Life network stream with legacy analyzers (UnrealDemoScanner, HL Demo Player).

---

## v0.23.0 — Behavioral Anti-Cheat Engine (`goldsrc-ac`) 📝 Planned

**Goal:** Build a server-authoritative, zero-cost behavioral anti-cheat plugin synthesizing ReAimDetector 3D raycasting and UnrealDemoScanner temporal input signatures with Adaptive Deep-Scan attention and regression testing against real demo corpora.

### 1. Hybrid Detection Core (ReAimDetector + UDS Heuristics)

- [ ] **Server-Authoritative 3D Hitbox Matrix (ReAimDetector Synthesis)**:
  - Server-side hitbox reconstruction with lag-compensation validation (`sv_unlag`).
  - True 3D raycasts calculating angular deviation to bone centers and closest hitbox bounding box facets.
- [ ] **Temporal Input Heuristics (UnrealDemoScanner Extraction)**:
  - **ViewAngle GCD / Pitch-Yaw Quantization**: detect mouse sensor step discreteness vs synthetic software floats.
  - **Angular Acceleration & Jerk Curvature**: identify unnatural bell-curve violations and instant linear interpolations.
  - **PunchAngle RCS Compensation**: detect sub-15ms recoil compensation ignoring human neuromuscular reaction latency.
  - **Sub-tick Button Distribution**: detect zero-variance `IN_JUMP` (Bhop) and `IN_ATTACK` (FastZoom / KnifeBot) patterns.

### 2. Adaptive Attention & Deep-Scan Lock-in

- [ ] **Selective Attention Architecture**:
  - Tier 1 Lightweight Triage: $O(1)$ fast filters for 95% of verified players; CPU overhead near zero.
  - Suspicion Accumulator: Leaky-bucket anomaly counter with exponential temporal decay.
  - Tier 2 Deep-Scan Lock-in: full hitbox history recording, multi-ray collision tests, and micro-timing analysis activated only upon threshold breach.

### 3. Demo Corpus CI/CD Regression Suite

- [ ] **Automated Test Fixtures (`tests/fixtures/demos/`)**:
  - Integrate clean and dirty `.dem` test suite from UDS corpus into automated CI.
  - Automated verification of **0 False Positives** on legitimate professional player demos and **100% True Positives** on confirmed cheat signatures.


