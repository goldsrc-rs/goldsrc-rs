# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- Added multi-target and pagination support (`-p`, `-s`, `-a`) to Host CLI commands.

### Changed

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
