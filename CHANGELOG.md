<!-- markdownlint-disable MD024 -->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **Panic Isolation**: Kept `panic = "unwind"` in release profile so `catch_ffi_panic` (`catch_unwind`) stops Rust panics at the C-ABI boundary before crashing HLDS.
- **Standalone Console Drain**: `PRINT_QUEUE` is drained each frame in `hook_start_frame` with fmtlib-safe `%`/`{}` escaping to avoid leaks and ReHLDS crashes.
- **Aliasing Soundness**: Eliminated raw `&'static mut PluginManager` re-borrowing in `wasm_manager` using safe singletons.
- **WASM Host Bridge**: Replaced mock host constants with `Arc<dyn EngineOps>` delegation across both backends.
- **Watcher Debouncing**: Added 150ms reload debounce in `PluginManager` to prevent loading partially written files during compilation.

### Changed

- **Backend Common Layer**: Backends (`goldsrc-metamod` and `goldsrc-standalone`) refactored into thin adapters over shared `EngineBackend` and `PrintQueue` in `goldsrc::backend`.
- **SDK Facade & Prelude**: Exposed `goldsrc::api`, `goldsrc::macros`, and convenient `goldsrc::prelude::*` in `framework/goldsrc`.

### Removed

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
