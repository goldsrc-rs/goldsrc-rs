# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.2.0] - 2026-08-12 (Stage 7: Component Model & TOML Configs)

### Added
- **WASM Component Model (`wit-bindgen`)**: Fully migrated WASM host and SDK to WebAssembly Component Model (`wit-bindgen` 0.60.0 & `wit-component` 0.256.0), eliminating raw `extern "C"` bridges.
- **Wasmtime & Cranelift Engine**: Replaced interpreter runtime with `wasmtime` JIT compiler running natively on `i686-pc-windows-msvc`.
- **TOML Central Configuration (`goldsrc.toml`)**: Introduced `cstrike/addons/goldsrc/goldsrc.toml` auto-generated configuration file for managing host paths, hot-reloading, and config watchers.
- **WASM Build Optimization**: Added automated `wasm-opt` optimization step in `build.py` reducing WASM plugin size down to ~200KB.
- **Zero JSON Overhead**: Replaced `serde_json` with TOML for metadata and configs, completely purging `serde_json` from dependencies.

### Changed
- **Binary Directory**: Renamed plugin binary path from `dlls/` to `bin/` for modern cross-platform clarity.

## [Unreleased]

### Added (Stage 6)
- **Architecture Restructuring**: Reorganized the workspace into `core/`, `backends/`, `framework/`, and `tools/` for better maintainability.
- **WASM Host Imports**: Introduced a safe FFI bridge allowing WASM plugins to call engine functions directly.
- **Elegant DX**: Added `#[on_load]` procedural macro to eliminate `unsafe` initialization code in plugins.
- **Plugin Author Field**: Added `author` metadata to `#[plugin]` macro.
- **Reduced Payload**: Optimized WASM payload sizes via Cargo release profile optimizations.

### Added

- Initial project scaffolding: Cargo workspace with 5 crates (`goldsrc-sys`, `goldsrc-api`, `goldsrc-metamod-backend`, `goldsrc-wasm-host`, `goldsrc`).
- MIT license and Rust `.gitignore`.
- Repository management rules in `AGENTS.md`: branching strategy, commit conventions, PR workflow, branch protection.
- HLSDK reference headers cloned as git submodule in `references/hlsdk/`.
- ReHLDS, metamod-r, and GoldSrcMod.Net reference sources cloned in `private/references/`.
- README.md and ROADMAP.md rewritten in English.
- `goldsrc-sys` crate with bindgen-generated FFI from HLSDK headers (enginefuncs_t, edict_t, entvars_t, globalvars_t).
- `goldsrc-api` crate with Engine trait, Plugin trait, Entity/Player handles.
- `goldsrc-metamod-backend` crate with Engine trait implementation using Metamod API.
- `goldsrc-wasm-host` crate with PluginRuntime trait and PluginManager.
- `goldsrc` public framework crate.
- Single Python setup script (`scripts/setup.py`) for reference repos and SDK detection.
- `.build-config.toml` generation (gitignored, machine-specific).
- GitHub Actions CI for Windows and Linux.
- Auto-format GitHub Action and pre-commit hook.
- Integrated `wasmi` pure-Rust WebAssembly interpreter into `goldsrc-wasm-host`.
- Full plugin lifecycle management for WASM modules: `on_load`, `on_unload`, `on_frame` callbacks.
- File system watcher (`notify`) for multi-directory hot-reloading without server restarts.
- Engine console logging (`server_print`) integration for WASM host and modules.
- Metamod `pfnStartFrame` hook integration for WASM module frame ticks.
- Dynamic versioning (`CARGO_PKG_VERSION`, `GIT_HASH`, `BUILD_TARGET`) via `build.rs` environment variables.
- Host CLI (`mrs`) commands implemented via `lexopt`: `load`, `unload`, `reload`, `pause`, `unpause`, `list`, `info`.
- Added procedural macros crate `goldsrc-macros` (`#[plugin]`, `#[command]`).
- Added WASM SDK `goldsrc` crate with logging macros (`log_info!`, `log_warn!`, `log_err!`).
- Added SemVer DAG plugin dependency resolution & topological sorting.
- Added inter-plugin Pub/Sub event bus across WASM modules.
- Added configuration hot-reloader watching `configs/` with JSON minification and broadcasting.
- Added automated deployment script `scripts/deploy.py` with MD5 hash verification.
- Added Git `pre-commit` hook combining `cargo fmt`, `cargo clippy` and `cargo test`.

### Fixed

- Fixed ReHLDS `fmtlib` crash when printing strings containing `{` or `}` by escaping them to `{{` and `}}`.
- Fixed GoldSrc CRT console buffer overflow during frame ticks by implementing a rate-limited `PRINT_QUEUE` flushed in `StartFrame_Post`.
- Resolved all Clippy warnings across the entire Cargo workspace.

- Default branch set to `dev` on GitHub.
- Branch protection enabled on both `main` and `dev`.
- Squash-and-merge enabled as the default merge strategy.
- Moved all reference repositories from `private/references/` to `references/` (gitignored).
- Converted setup scripts from PowerShell/Bash to single Python script.
- Converted pre-commit hook from Bash to Python.

### Fixed

- Corrected TOML config format (array instead of repeated keys).
- Fixed backslash escaping in setup script.
- Disabled bindgen layout tests (fail on 32-bit Linux with max_align_t).
- Fixed LIBCLANG_PATH for Windows CI.
- Addressed clippy warnings (unused imports, missing Default impl, unnecessary unsafe blocks).
