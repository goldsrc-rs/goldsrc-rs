# GoldSrc.rs

<!-- Project Status & Metrics -->
![Status](https://img.shields.io/badge/status-prototype-orange?logo=valve) [![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license) [![CI](https://github.com/goldsrc-rs/goldsrc-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/goldsrc-rs/goldsrc-rs/actions/workflows/ci.yml)  
<!-- Repository & Community -->
![GitHub Created At](https://img.shields.io/github/created-at/goldsrc-rs/goldsrc-rs?logo=github) [![Last Commit](https://img.shields.io/github/last-commit/goldsrc-rs/goldsrc-rs)](https://github.com/goldsrc-rs/goldsrc-rs/commits/main) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md) ![GitHub contributors](https://img.shields.io/github/contributors/goldsrc-rs/goldsrc-rs?logo=github) [![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?logo=readme)](https://github.com/RichardLitt/standard-readme)  
<!-- Tech Stack & Targets -->
[![Rust: 2024 Edition](https://img.shields.io/badge/rust-2024_edition-orange.svg?logo=rust&logoColor=orange)](https://doc.rust-lang.org/edition-guide/rust-2024/) [![Python 3.10+](https://img.shields.io/badge/python-3.10+-3776AB?logo=python)](https://www.python.org/) ![Targets: i686-windows | i686-linux](https://img.shields.io/badge/targets-i686--windows%20%7C%20i686--linux-lightgray.svg?logo=linux&logoColor=black)

> A modern, memory-safe Rust framework and WebAssembly plugin runtime for GoldSrc engine modding.

**GoldSrc.rs** is a next-generation framework written in Rust for developing server-side plugins, extensions, and modifications for the GoldSrc engine, with first-class (Tier-1) specialization for **Counter-Strike 1.6 (ReHLDS / ReGameDLL / GoldClient)** as well as broader GoldSrc games.

It replaces the legacy C++ / Metamod / AMX Mod X (Pawn) stack with memory-safe Rust, WebAssembly (WASM)
sandboxing, zero-overhead FFI, ergonomic ECS abstractions, and dynamic hot-reloading.

> [!WARNING]
> **Early Development & Testing Status**:  
> GoldSrc.rs is currently in active early-stage development (prototype/alpha). While core architectures (WASM Component Model, SQLite WAL, and Metamod/Standalone backends) are functional and verified locally, the framework has not yet undergone extensive battle-testing across diverse production servers, operating systems, and varied GoldSrc game modifications (e.g. Day of Defeat, Team Fortress Classic, Sven Co-op). APIs and configurations may evolve between minor versions.

## Table of Contents

- [Background](#background)
- [Features](#features)
- [Architecture](#architecture)
- [Install & Prerequisites](#install--prerequisites)
- [Usage](#usage)
  - [1. Setup Reference SDKs](#1-setup-reference-sdks)
  - [2. Build Backends and Plugins](#2-build-backends-and-plugins)
  - [3. Deploy to Server](#3-deploy-to-server)
  - [In-Game Management (`grs` CLI)](#in-game-management-grs-cli)
- [Maintainers](#maintainers)
- [Contributing](#contributing)
- [Acknowledgements](#acknowledgements)
- [License](#license)

## Background

Developing server-side modifications for GoldSrc games has historically relied on C++ (Metamod) and Pawn (AMX Mod X).
While functional, this stack suffers from lack of memory safety, global server crashes on single-plugin segfaults,
cumbersome tooling, and the requirement of restarting the server process to update code.

**GoldSrc.rs** solves these challenges by:

1. **Sandboxing plugins in WebAssembly (WASM):** Malfunctioning or crashing plugins cannot take down the game server.
2. **True dynamic hot-reloading:** Developers can iterate, compile, and reload `.wasm` plugins instantly without changing maps or restarting server.
3. **Ergonomic Safe Rust SDK:** Rich, strongly-typed representations for entities, vectors, player authentication, and event buses.
4. **Dual Backend Compatibility:** Seamless operation as a standard Metamod-r plugin or as a direct standalone GameDLL proxy.

## Features

- **Memory Safety & Panic Isolation:** Safe Rust abstractions prevent pointer corruption. FFI error boundaries isolate panics with `catch_unwind`.
- **WASM Component Model & JIT:** Plugins compile to WebAssembly components executed by **Wasmtime** at near-native JIT execution speed.
- **Live Hot-Reloading:** Add, update, or remove plugins on the fly with live state preservation.
- **Embedded SQLite WAL Storage:** High-performance, non-blocking asynchronous state persistence with isolated typed `Bucket<T>` sandboxing.
- **Per-Player Localization (i18n):** Modular TOML language dictionaries with dynamic color macros, fallback chains, and player typestate resolution.
- **Dual Backend Architecture:**
  - **Metamod Backend:** Operates as a Metamod-r plugin (`.dll` / `.so`), preserving full compatibility with Reunion, WHBlocker, VoiceTranscoder, etc.
  - **Standalone Backend:** Direct proxy GameDLL (`liblist.gam` → `gamedll`), running directly inside the engine without third-party loaders.
- **Host Management CLI:** Built-in `grs` in-game and server console management CLI, SemVer DAG dependency sorting, and TOML configuration.
- **Safe Entity & ECS API:** High-level abstractions for players, edicts, vectors, trace lines, and event buses.

## Architecture

```text
goldsrc-rs/
├── core/
│   ├── goldsrc-sys/          # Low-level FFI bindings generated by bindgen from HLSDK/Metamod headers
│   └── goldsrc-api/          # Pure Rust traits, Entity/Player handles, typestates, and event signatures
├── backends/
│   ├── goldsrc-metamod/      # Metamod-r C-ABI plugin adapter (Meta_Query, Meta_Attach)
│   └── goldsrc-standalone/   # Standalone proxy GameDLL adapter (GiveFnptrsToDll, GetEntityAPI2)
├── hosts/
│   └── goldsrc-wasm-host/    # Embedded Wasmtime runtime, JIT engine, and plugin manager
├── framework/
│   ├── goldsrc/              # High-level SDK: ECS, Event Bus, logging, config watchers, storage, i18n
│   └── goldsrc-macros/       # Procedural macros: #[plugin], #[command], #[on_load]
├── resources/
│   └── lang/                 # Global shared localization dictionaries (common.toml, test_i18n.toml)
├── examples/
│   └── demo_plugins/         # Example WASM plugins (admin_system, test_hud, test_menu, test_i18n, vip_core, vip_menu)
└── scripts/                  # Unified Python CLI automation tools (setup, build, deploy, diagnostics)
```

## Install & Prerequisites

Ensure the following tools are installed before building:

1. **Rust Toolchain (1.80+):**

   ```bash
   rustup default stable
   rustup target add i686-pc-windows-msvc      # Windows 32-bit HLDS target
   rustup target add i686-unknown-linux-gnu   # Linux 32-bit HLDS target
   rustup target add wasm32-unknown-unknown   # WASM plugin target
   ```

2. **C/C++ Build Tools:**
   - **Windows:** Visual Studio 2022 / C++ Build Tools with MSVC x86 compilers.
   - **Linux:** `gcc-multilib`, `g++-multilib`, `libclang-dev`.

3. **Python 3.10+:** Required for repository automation and tooling.
4. **wasm-opt (Binaryen) (Optional):** Recommended for optimizing and shrinking WASM binaries.

## Usage

### 1. Setup Reference SDKs

Clone the repository and run the setup command to download reference SDK headers:

```bash
git clone https://github.com/goldsrc-rs/goldsrc-rs.git
cd goldsrc-rs

# Download HLSDK / Metamod reference headers and verify environment
python -m scripts setup
```

### 2. Build Backends and Plugins

Build the workspace (backends and WASM plugins) for release:

```bash
# Build standalone backend and all demo plugins
python -m scripts build --backend standalone --release

# Or build Metamod backend
python -m scripts build --backend metamod --release

# Or build all backends
python -m scripts build --backend all --release
```

### 3. Deploy to Server

Deploy directly to your local HLDS test server:

```bash
# Deploy Standalone backend and WASM plugins (updates liblist.gam automatically)
python -m scripts deploy --path "C:/path/to/hlds/server" --backend standalone

# Or deploy Metamod backend (updates plugins.ini automatically)
python -m scripts deploy --path "C:/path/to/hlds/server" --backend metamod

# Or run standalone verification check only
python -m scripts verify --path "C:/path/to/hlds/server" --backend metamod
```

### In-Game Management (`grs` CLI)

GoldSrc.rs provides a built-in server console and client command `grs` (or `goldsrc-rs`):

> [!NOTE]
>
> - **Standalone backend** supports only `grs` and `goldsrc-rs` commands.
> - **Metamod backend** supports `grs`, `goldsrc-rs` and additionally `mrs` / `meta-rs` command aliases.

| Command | Description |
| --- | --- |
| `grs list` | List all loaded WASM plugins and their current status |
| `grs info <plugin>` | Display detailed metadata and capabilities of a plugin |
| `grs load <name.wasm>` | Load a new plugin from `goldsrc/plugins/` |
| `grs unload <plugin>` | Gracefully unload an active plugin |
| `grs reload <plugin>` | Hot-reload a plugin without restarting the server |
| `grs pause / unpause` | Pause or resume event dispatching for a plugin |
| `grs status` | Print engine backend, memory usage, and runtime info |

## Maintainers

- [@ulquiorracode](https://github.com/ulquiorracode) — Project Lead & Creator

## Contributing

1. Review [Branching Strategy and Guidelines](ROADMAP.md).
2. Create a feature branch (`git checkout -b feature/amazing-feature` from `dev`).
3. Commit your changes using [Conventional Commits](https://www.conventionalcommits.org/).
4. Ensure all pre-commit checks pass (`python -m scripts pre-commit`).
5. Open a Pull Request targeting the `dev` branch.

## Acknowledgements

GoldSrc.rs builds upon decades of community reverse engineering and tooling:

- **Valve Software** — for Half-Life and the GoldSrc engine.
- **[AlliedModders](https://github.com/alliedmodders)** & **Will Day** — for canonical HLSDK headers, original Metamod, and AMX Mod X.
- **[Metamod-r Team](https://github.com/theAsmodai/metamod-r)** (Asmodai, Dreamstalker, s1lentq) — for modern optimized Metamod-r architecture and reverse-engineering contributions.
- **[ReHLDS Team](https://github.com/dreamstalker/rehlds)** & **[ReGameDLL Team](https://github.com/s1lentq/ReGameDLL_CS)** — for reverse-engineered engine/gamedll insights.
- **[AmxxModularEcosystem](https://github.com/AmxxModularEcosystem)** (VipModular) — for pioneering modular plugin architecture and decoupled capability management in the GoldSrc ecosystem.
- **[MultiMod Manager](https://github.com/FEDERICOMB96/amxx-multimod-manager)** — for inspiration on dynamic plugin lifecycle orchestration, profile bundles, and mode switching.
- **[GoldSrcMod.Net](https://github.com/DrAbcOfficial/GoldSrcMod.Net)** — for architectural inspiration on modern language runtimes in GoldSrc.
- **[Bytecode Alliance](https://bytecodealliance.org/)** — for the `wasmtime` runtime and WASM Component Model.

> [!NOTE]
> Half-Life, GoldSrc, and the Half-Life logo are trademarks and/or registered trademarks of Valve Corporation.  
> GoldSrc.rs is an independent, non-commercial open-source project and is not affiliated with, endorsed by, or sponsored by Valve Corporation or Rust Foundation.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
