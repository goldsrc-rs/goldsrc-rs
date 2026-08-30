<!-- markdownlint-disable MD024 -->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.14.0] - 2026-08-30

### Added

- **Lightweight i18n & Localization Dictionary Engine (`framework/goldsrc/src/i18n`)**:
  - Modular TOML dictionary parser with lexical variable scoping (`$vars.name`), template macros (`@{tag('VIP')}`, `@{g('{name}')}`), and per-player fallback chains.
  - Access control policies (`DictAccess::Public`, `Private`, `Shared`) with fallback to global `common.toml`.
  - Fluent dictionary builder (`LangDict::builder`) and directory auto-merging (`I18nEngine::load_dir`).
  - Convenient `tr!` macro with seamless support for `AsLangCode` (`&str`, `String`, `&Player`, `Player`, `Alive<T>`, `Terrorist`, etc.).
  - `Player::lang(&self)` and `I18nEngine::server_lang()` helpers for zero-boilerplate language resolution.
- **Embedded SQLite WAL Storage Engine (`goldsrc-storage` & `framework`)**:
  - High-performance SQLite engine running in WAL mode with background `mpsc` batching.
  - Strongly typed `Bucket<T>` guest DX wrapper with zero-cost serialization.
  - Strict WASM host isolation with automatic `{plugin_id}/` key prefixes.
- **Multi-Line Console Print Splitting & Expanded Buffer**:
  - `EngineBackend::server_print` splits multiline messages line-by-line (`\n`) for clean rendering in server console and RCON.
  - Expanded `escape_server_print` line length buffer to 1024 bytes.
- **WASM Epoch Timeout Hardening**:
  - Increased epoch deadlines (`EPOCH_DEADLINE_COMMAND = 500`, `EPOCH_DEADLINE_LOAD = 1000`) providing 1-2s wall-clock headroom to prevent spurious plugin poisoning on Windows QuickEdit / console pauses.

## [0.13.1] - 2026-08-28

### Added

- **Dual-Format `plugins.toml` Parser (Named Map + Array-of-Tables)**:
  - Full support for Named Tables: `[plugins.<name>]`, `[groups.<name>]`, and `[rules.<name>]`.
  - Maintained 100% backward compatibility with `[[plugins]]` and `[[rules]]`.
- **Granular Pause Reason Tracking (`PluginStatus::Paused { reason }`)**:
  - Attached descriptive pause reasons (`"reactive rule"`, `"group 'test_suite' disabled"`, `"plugin 'test_hud' disabled in config"`).
  - Formatted reason output in `grs info <name/idx>` and plugin status introspection.
- **Recursive Boolean Conditions for Reactive Rules**:
  - Added support for `all_of = [...]` (AND), `any_of = [...]` (OR), and `none_of` / `not` (NOT) composite conditions.
- **Live Server Player Slot Tracker**:
  - Live edict-based slot verification in `EngineBackend::count_active_players` for instant reactive evaluation on player connect and disconnect events.

## [0.13.0] - 2026-08-27

### Added

- **Reactive Rule & Provider Engine (`core/goldsrc-api` & `framework/goldsrc`)**:
  - Generic, extensible `RuleEngine<Context>` with decoupled `RuleCondition` and `RuleAction` registries.
  - Built-in condition evaluators: `map` (patterns/wildcards), `players` (range strings/counts), `cvar` (operators `==`, `!=`, `>=`, `<=`, `>`, `<`), and `time`.
  - Built-in action executors: `pause`, `unpause`, `set_cvar`, and `exec` (console command execution).
- **Declarative Plugin Orchestration (`plugins.toml`)**:
  - `PluginsConfig` model parsing `[[plugins]]`, `[groups.<name>]`, and `[[rules]]`.
  - Granular `PluginDebugConfig` (log levels `trace`..`error`, profiling, dedicated plugin log files, per-plugin epoch limits).
  - Profile groups for instant batch toggling of plugin bundles.
- **Recursive Directory Bundles (`plugins/<bundle>/*.wasm`)**:
  - Recursive tree-walking loader discovering nested WASM plugin packs (e.g. `plugins/test_suite/test_hud.wasm`).
  - Recursive `notify` file system watching for instant hot-reload across nested directories.
- **Decomposed Micro-Plugins**:
  - Split monolithic test suite into dedicated, focused modules: `test_hud`, `test_menu`, and `test_ecs`.
  - Updated build and deployment automation to deploy bundle subfolders.

## [0.12.0] - 2026-08-27

### Added

- **Declarative Menu Engine**:
  - `Menu::builder` with explicit pagination (`.page(...)`), slot actions, and `ExitBehavior` (`PopParent`, `Close`).
  - Menu styles (`MenuStyle::brackets()`, `MenuStyle::classic()`) and pluggable renderers (`ShowMenu`, `Dhud`).
  - Seamless action handling with `#[menu_action(id = ...)]`.
- **True Director HUD (DHUD) & Screen Effects**:
  - Full Director HUD wire format (`SVC_DIRECTOR` with `DRC_CMD_MESSAGE`) rendering smooth VGUI typography.
  - Classic 4-channel HUD (`SVC_TEMPENTITY` / `TE_TEXTMESSAGE`) with typewriter, flicker, and fade effects.
  - Screen effects: `ScreenFade` (damage flashes, flashbang blindness) and `ScreenShake` (tremors, explosions) with fluent builders.
- **Unified Requirements DSL (`require = [...]`)**:
  - Expressive requirement parser supporting `plugin:<name>[@<ver>]`, `plugin:<name>?`, `cvar:<name>[=<v>|!=<v>|>0]`, and `feature:<name>`.
  - Dynamic runtime recalculation of plugin lifecycle states.
- **Defensive Server Configuration (`goldsrc.toml`)**:
  - `HostConfig` model covering `[core]`, `[logging]`, `[watcher]`, and `[runtime]`.
  - Automatic boundary clamping for memory limits, table limits, debounce intervals, and watchdog timeouts with safe fallbacks.
- **Typed CVar Abstraction**:
  - `Cvar<i32>`, `Cvar<f32>`, `Cvar<String>` and flags (`CvarFlags::ARCHIVE`, `NOTIFY`, `SERVER`, `READ_ONLY`).

### Fixed

- **Duplicate Plugin Load**: Added `LoadError::AlreadyLoaded` check preventing multiple active instances of the same plugin in `grs load`.
- **Watcher Initialization Warning**: Ensured config directories are created before spawning `notify` file system watchers.
- **CLI Query Simplification**: Cleaned up `grs info` field matching to strictly match canonical metadata fields.
- **Legacy Dependencies Cleanup**: Removed deprecated `dependencies` attribute across macros, host, and CLI in favor of `require`.

## [0.11.0] - 2026-08-23

### Added

- **Multi-Agent Adversarial Hardening**: Resolved 10+ critical findings across 2 audit rounds:
  - Background daemon epoch timer (`goldsrc-epoch-timer`) in `PluginManager` for strict wall-clock WASM timeout enforcement.
  - Wasmtime `StoreLimits` (64MB memory, 10,000 table elements per store).
  - Preserved Metamod shared memory (`mutil_funcs_t`) ensuring 100% stable co-existence with AMX Mod X.
  - Aligned `MessageDest` enum discriminants with GoldSrc `const.h` and added dynamic `SayText` discovery via `reg_user_msg`.
  - Panic barriers (`catch_ffi_panic`) enclosing all entity factory exports in standalone backend.
- **Typed Command Extraction (`FromArg`)**: Automatic type-safe argument parsing for primitives, strings, and player entities (`Player`, `Alive<Player>`).
- **Auto Caller Binding**: In-game player commands without positional parameters (e.g. `/vip`) automatically bind caller to target entity arguments.
- **Hierarchical Capability DSL**: Boolean capability syntax (`and`, `or`, `not`, group wildcards `admin.*`) with fail-closed authorization.
- **Declarative Command System**: Structured `CommandSpec` with per-command help (`grs <cmd> --help`, `grs help <cmd>`).
- **Precache Consolidation**: Moved asset precaching to run once per map on post-`ServerActivate`.

### Fixed

- **CLI Self-Deadlock**: Replaced recursive `RUNTIME` mutex acquisition in `grs cmd` with direct borrowed manager execution.
- **SayText & Network Buffer Overflow**: Bounded chat messages and network strings to prevent GoldSrc buffer overflows.
- **SayText Sender ID**: Sent `0` (console/server) as first byte of SayText user message payload.
- **Bindgen Layout Tests**: Blocklisted `max_align_t` and passed `--target` to clang to fix 32-bit cross-compilation assertions.
- **File Logging**: Buffered file writes via `BufWriter` with proper flush implementation.

## [0.10.0] - 2026-08-22

### Added

- **Rust 2024 Edition Migration**: Modernized entire workspace to Rust 2024 edition across all crates.
- **WASM Component Model with Pulley32**: Upgraded `goldsrc-wasm-host` to pure-Rust bytecode execution via Wasmtime Pulley32 for full 32-bit HLDS stability.
- **Engine Bridge & String Pool Resolver**: Safe `Engine` composite trait with native string table resolution (`pfnGetInfoKeyBuffer`/`pfnInfoKeyValue` and `pfnSzFromIndex`).
- **Automatic Resource Precaching**: Thread-safe precache queue in `EngineBackend` for audio/model precaching.
- **Real-Time Host CLI Commands**: Integrated `pfnAddServerCommand` with synchronous print flush.
- **Functional Demo Plugin Suite**:
  - `admin_system`: Administration utilities (granting capabilities, slaying players, teleportation, gravity manipulation).
  - `vip_core`: Dynamic capability authorization (`vip.access`), player buffing and healing.
  - `vip_menu`: Interactive VIP kit deployment with sound and visual feedback.
  - `test_suite`: ECS verification, player inspection (health, armor, origin, angles), CVar manipulation, sound playback.

## [0.9.0] - 2026-08-19

### Added

- **Host / Backend Separation**: Separated engine adapters (`backends/`) from plugin execution environments (`hosts/`), moving `goldsrc-wasm-host` into `hosts/` and introducing the abstract `PluginHost` trait.
- **Unified Python Automation CLI**: Created centralized `scripts` CLI (`__main__.py`) supporting `setup`, `build`, `deploy`, `verify`, `pre-commit`, `analyze` (crash-analyzer), and `logo` (vector SVG / raster PNG generator).
- **Organization Infrastructure**: Migrated to GitHub organization `goldsrc-rs`, established `.github` profile landing page, and added global community health files (`SECURITY.md`, `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`).
- **Dual Licensing**: Adopted standard Rust dual-license `MIT OR Apache-2.0`.

### Fixed

- **Panic Isolation**: Kept `panic = "unwind"` in release profile so `catch_ffi_panic` (`catch_unwind`) stops Rust panics at the C-ABI boundary before crashing HLDS.
- **Standalone Console Drain**: `PRINT_QUEUE` is drained each frame in `hook_start_frame` with fmtlib-safe `%`/`{}` escaping to avoid leaks and ReHLDS crashes.
- **Aliasing Soundness**: Eliminated raw `&'static mut PluginManager` re-borrowing in `wasm_manager` using safe singletons.
- **WASM Host Bridge**: Replaced mock host constants with `Arc<dyn EngineOps>` delegation across both backends.
- **Watcher Debouncing**: Added 150ms reload debounce in `PluginManager` to prevent loading partially written files during compilation.

### Changed

- **Backend Common Layer**: Backends (`goldsrc-metamod` and `goldsrc-standalone`) refactored into thin adapters over shared `EngineBackend` and `PrintQueue` in `goldsrc::backend`.
- **SDK Facade & Prelude**: Exposed `goldsrc::api`, `goldsrc::macros`, and convenient `goldsrc::prelude::*` in `framework/goldsrc`.
- **Safe Abstraction Layer**: Isolated raw `edict_t` handles behind `unsafe-sys` feature flag.

### Removed

- Removed legacy C artifacts: `exports.def`, `metamod.def`, `wrapper.c`.
- Dead legacy code: unused `wit-component` in `goldsrc-api`, empty `[build-dependencies]` in `goldsrc-standalone`, and dead types.

## [0.8.0] - 2026-08-13

### Added

- **Standalone Backend** (`goldsrc-standalone`): New proxy GameDLL backend loaded directly by the
  engine via `liblist.gam` `gamedll` key, eliminating the hard Metamod dependency.
- **Universal GameDLL Auto-Detection**: Automatically locates `mp.dll` / `cs.so` at runtime with
  `GetEntityAPI2` / `GetEntityAPI` version fallbacks.
- **`PathResolver` Integration**: All runtime paths (`plugins/`, `configs/`, `logs/`) resolved
  relative to the server working directory; no hardcoded absolute paths remain.

### Fixed

- **Memory Corruption** in `GetNewDLLFunctions`: 512-byte buffer was overwriting the 20-byte
  `NEW_DLL_FUNCTIONS` engine struct, causing random HLDS crashes and freezes.
- **Mutex Re-Entrancy Deadlocks**: Proxy callbacks now release locks before forwarding to the real
  GameDLL, resolving infinite hang on map load.

### Removed

- Hardcoded developer paths (`C:\Users\Administrator\...`) from standalone backend logging.

## [0.7.0] - 2026-08-12

### Added

- **Wasmtime JIT Engine**: Replaced `wasmi` interpreter with `wasmtime` + Cranelift for native
  x86 JIT compilation of WASM plugins.
- **WASM Component Model**: Migrated host and SDK to `wit-bindgen` 0.60.0 and `wit-component`
  0.256.0, eliminating all raw `unsafe extern "C"` WASM bridges.
- **Central TOML Configuration** (`goldsrc.toml`): Auto-generated config file in
  `cstrike/addons/goldsrc/` managing host paths, hot-reload flags, and config watchers.
- **Capability-Based Access Control (RBAC)**: Per-plugin capability declarations enforced by the
  WASM host at load time.
- **`wasm-opt` Build Pipeline**: Automated size optimization in `build.py` reducing WASM plugin
  binaries to ~200 KB (90% reduction).

### Removed

- `serde_json` dependency: replaced entirely by TOML and the Canonical ABI.

### Changed

- Plugin binary directory renamed from `dlls/` to `bin/`.

## [0.6.0] - 2026-08-11

### Added

- **Workspace Restructuring**: Reorganized into `core/`, `backends/`, `framework/`, `tools/` layers.
- **WASM Host Imports**: Safe FFI bridge allowing WASM plugins to call engine functions directly.
- **`#[on_load]` Macro**: Eliminates `unsafe` initialization boilerplate in plugin entry points.
- **`author` field**: Added to `#[plugin]` procedural macro metadata.

### Changed

- Renamed `metamod-rs` crate to `goldsrc-metamod`.

### Performance

- Optimized WASM payload sizes via Cargo release profile (`lto`, `opt-level = "z"`, `strip`).

## [0.5.0] - 2026-08-10

### Added

- **Flat ECS**: Sparse-Set Entity Component System in `goldsrc` framework crate for WASM plugins.
- **High-Level API Wrappers**: `Player`, `Entity`, `Vector3` safe Rust types over raw `edict_t`.
- **Granular Config Events**: `created` / `modified` / `deleted` actions with per-plugin config
  directory isolation (`configs/plugins/<name>/`).
- **Host Logger Service**: Structured log levels (`Trace`, `Info`, `Warn`, `Error`) with
  auto-created output directories (`logs/metamod-rs.log`).
- **`invoke_two_slices`**: Generic WASM FFI helper eliminating duplicated slice-passing boilerplate.

## [0.4.0] - 2026-08-09

### Added

- **Host Management CLI (`mrs`)**: `lexopt`-based console interface with `load`, `unload`, `reload`,
  `pause`, `unpause`, `list`, `info` commands and `-a/--all` flag.
- **`goldsrc-macros`** crate: `#[plugin(systems=...)]` and `#[command]` procedural macros.
- **SDK `goldsrc`** crate: Flat/Hybrid ECS API and logging macros (`log_info!`, `log_warn!`, `log_err!`).
- **Plugin DAG Dependency Resolution**: SemVer-validated topological sort via `semver` crate.
- **Global Event Bus**: Pub/Sub inter-plugin communication across WASM module boundaries.
- **Config Hot-Reloader**: `configs/` directory watcher with event broadcasting.
- **Deployment Script** (`scripts/deploy.py`): Automated build, copy, and MD5 hash verification.
- **Pre-Commit Hook**: Python script combining `cargo fmt`, `cargo clippy`, and `cargo test`.

## [0.3.0] - 2026-08-08

### Added

- **WASM Plugin Host** (`goldsrc-wasm-host`): Integrated `wasmi` pure-Rust interpreter.
- **Host Bindings**: `server_print` WASI binding for plugins to write to the server console.
- **Hot-Reload**: `notify`-based file watcher for `.wasm` files; plugins reload without server restart.
- **Plugin Lifecycle**: Full `on_load` → `on_frame` → `on_unload` callback chain with error isolation.

## [0.2.0] - 2026-08-07

### Added

- **Metamod Backend**: Full `GiveFnptrsToDll` / `Meta_Attach` / `Meta_Query` implementation.
- **Engine Hooks**: `DispatchSpawn`, `ClientConnect`, `ClientDisconnect`, `ClientCommand` via Metamod.
- **VTable Hook System**: Cross-platform (Windows/Linux) vtable patching using offsets from ReHLDS
  and HamSandwich.
- **Engine Structure Wrappers**: Safe Rust wrappers over `edict_t`, `entvars_t`, `CBaseEntity`.
- **Console Logging**: `SERVER_PRINT` / `ALERT` integration. Server prints "Hello from Rust!".

### Fixed

- ReHLDS `fmtlib` crash when printing strings containing `{` or `}` (escaped to `{{` / `}}`).

## [0.1.0] - 2026-08-06

### Added

- Initial Cargo workspace with crates: `goldsrc-sys`, `goldsrc-api`, `goldsrc-metamod`,
  `goldsrc-wasm-host`, `goldsrc`, `goldsrc-macros`.
- `goldsrc-sys`: `bindgen`-generated FFI from HLSDK headers (`enginefuncs_t`, `edict_t`,
  `entvars_t`, `globalvars_t`).
- `goldsrc-api`: `Engine` and `Plugin` traits, `Entity` / `Player` handle types.
- Reference repositories in `references/` (HLSDK, metamod-r, ReHLDS, GoldSrcMod.Net).
- `scripts/setup.py`: Single Python script for reference repo setup and SDK detection.
- `.build-config.toml`: Machine-specific gitignored build configuration.
- GitHub Actions CI for Windows (`i686-pc-windows-msvc`) and Linux (`i686-unknown-linux-gnu`).
- Auto-format GitHub Action and Python pre-commit hook.
- MIT License, README, and ROADMAP.
