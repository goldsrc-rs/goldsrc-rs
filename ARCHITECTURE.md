# GoldSrc.rs Architecture Guide

Welcome to the architectural specification for **GoldSrc.rs** — a modern, modular, memory-safe plugin framework and WebAssembly runtime for GoldSrc engine servers (Half-Life, Counter-Strike 1.6, and compatible mods).

---

## 1. System Overview

GoldSrc.rs bridges the legacy GoldSrc C/C++ engine environment (HLDS, ReHLDS) with modern Rust systems engineering and isolated WebAssembly (WASM) guest plugins.

```mermaid
flowchart TB
    subgraph GoldSrcEngine ["GoldSrc Engine / HLDS (32-bit C/C++)"]
        EngineCore["Engine Core (swds.dll / engine_i486.so)"]
        GameDLL["GameDLL (mp.dll / cs.so)"]
    end

    subgraph Backends ["Backend Layer (FFI Adapters)"]
        Standalone["goldsrc-backend-standalone\n(Proxy GameDLL)"]
        Metamod["goldsrc-backend-metamod\n(Metamod Plugin)"]
    end

    subgraph Core ["Core Runtime Layer (goldsrc-core)"]
        HostRuntime["HostRuntime (Root Container)"]
        RuleOrch["RuleOrchestrator\n(Reactive Rules & Scopes)"]
        NetDispatcher["NetworkMessageDispatcher\n(TextMsg / SayText)"]
        ConfigSvc["ConfigService & File Watchers"]
        I18nSvc["I18nService\n(Per-Player Localization)"]
        StorageEngine["SqliteStorageEngine\n(WAL Mode, MPSC Worker)"]
    end

    subgraph HostWasm ["WASM Host Layer (goldsrc-host-wasm)"]
        WasmEngine["Wasmtime Engine (Pulley32)"]
        PluginMgr["PluginManager (Lifecycle FSM)"]
        CmdRegistry["CommandRegistry & Aliases"]
        EpochTimer["Epoch Interruption Worker"]
    end

    subgraph GuestPlugins ["Guest Plugins Layer (WASM Components)"]
        PluginA["vip_core.wasm (Service)"]
        PluginB["test_hud.wasm (Gameplay)"]
        PluginC["cstrike_vip_menu.wasm (Addon)"]
    end

    EngineCore <-->|C-ABI FFI| Backends
    Backends <-->|Engine Bridge| HostRuntime
    HostRuntime --> RuleOrch
    HostRuntime --> NetDispatcher
    HostRuntime --> ConfigSvc
    HostRuntime --> I18nSvc
    HostRuntime --> StorageEngine
    HostRuntime <-->|WASM Component Model| PluginMgr
    PluginMgr --> WasmEngine
    PluginMgr --> CmdRegistry
    EpochTimer -.->|Interrupt Epochs| WasmEngine
    WasmEngine <-->|WIT Interfaces| GuestPlugins
```

---

## 2. Workspace & Crate Structure

The repository is organized into distinct functional layers:

```text
goldsrc-rs/
├── core/
│   ├── goldsrc-api/                # Safe guest/host shared domain types, traits, DAG, ECS, and builders
│   ├── goldsrc-core/               # Host runtime, config, i18n, storage, logging, rule engine, FFI bridge
│   └── goldsrc-sys/                # Low-level raw FFI bindings to GoldSrc/Metamod headers (unsafe)
├── backends/
│   ├── goldsrc-backend-metamod/    # Metamod C-ABI adapter (meta_api.h)
│   └── goldsrc-backend-standalone/ # Proxy GameDLL adapter (GetEntityAPI2 / liblist.gam)
├── hosts/
│   └── goldsrc-host-wasm/          # Wasmtime runtime, Component Model bindings, epoch timer, plugin manager
├── framework/
│   ├── goldsrc/                    # Lightweight developer SDK for WASM guest plugins
│   └── goldsrc-macros/             # Procedural macros (#[plugin], #[command], #[event], #[system])
├── references/                     # C/C++ reference headers (HLSDK, Metamod, ReHLDS, ReGameDLL)
├── resources/                      # Configuration templates, default localization files, gamedata
├── scripts/                        # Modular Python toolchain (build, deploy, verify, analyze, setup)
└── examples/demo_plugins/          # Reference demo plugins (test_suite, vip_core, admin_system)
```

---

## 3. System Taxonomy & Role Suffixes

To maintain architectural purity and prevent God Objects, GoldSrc.rs strictly enforces standard role suffixes across all crates and components:

| Suffix | Responsibility | Architectural Invariants | Current / Target Examples |
| :--- | :--- | :--- | :--- |
| **`Engine`** | Low-level computational engine, execution driver, or external runtime platform. | Operates on raw bytecode, low-level OS/C-ABI, AST parsing, or DB connections. Agnostic of high-level gameplay rules. | `wasmtime::Engine`, `goldsrc_api::Engine` (C-ABI bridge), `SqliteStorageEngine`, `RuleEngine` (AST evaluator). |
| **`Orchestrator`** | High-level workflow coordinator managing lifecycle, phase DAGs, and multi-system synchronization. | Does not execute low-level operations directly. Coordinates the execution order across multiple independent subsystems. | `RuleOrchestrator` (game triggers $\to$ AST evaluation $\to$ plugin/cvar toggles), `PluginOrchestrator` (discovery $\to$ Phased DAG $\to$ load order). |
| **`Manager`** | State machine and lifecycle owner for a pool of homogeneous domain entities. | Owns collections (`Vec`, `HashMap`), executes state transitions (`Running`, `Paused`, `Unloaded`, `Blocked`), and performs CRUD. | `PluginManager` (owns `Vec<LoadedPlugin>` and Wasmtime stores), `MenuManager` (owns player menu sessions). |
| **`Registry`** | Passive or semi-passive catalog for lookups and symbol resolution. | Key-value or alias indexing. Does not own lifecycle or execute domain business logic. | `CommandRegistry` (command name/alias $\to$ owners), `PlaceholderRegistry` (tag $\to$ handler), `RuleRegistry` (predicate name $\to$ evaluator). |
| **`Service`** | Self-contained domain capability provider. | Encapsulates specific domain logic behind a clean API. May maintain internal caches or worker threads. Pluggable implementations implement service traits. | `ConfigService` (TOML watching & reload events), `I18nService` (translation by player locale), `AuthService` (player capabilities). |
| **`Dispatcher`** | Message/event router delivering payloads between producers and consumers. | Decouples senders from receivers. Routes 1-to-1 or 1-to-many. Does not hold persistent business state. | `EventDispatcher` (dispatches events to WASM plugins), `NetworkMessageDispatcher` (packs GoldSrc `TextMsg`/`SayText` network frames), `HookDispatcher`. |
| **`Router`** | Input argument parser and direct endpoint dispatcher. | Parses incoming raw command lines, text tokens, or network inputs and routes to matching handlers. | `CliRouter` (`dispatch_host_command`), `CommandRouter` (chat `/cmd` and console dispatch). |
| **`Bridge`** | Technical adapter across foreign runtime or ABI boundaries. | Connects two fundamentally different environments (e.g. C/C++ FFI, WIT component interfaces, or OS-level bindings). | `ReApiBridge` (ReHLDS/ReGameDLL FFI), `MetamodBridge`, `EngineBridge`. |

---

## 4. Key Runtime Data Flows

### 4.1. Server Frame Ticking (`on_server_frame`)

```mermaid
sequenceDiagram
    participant Engine as GoldSrc Engine
    participant Backend as Backend (Standalone/Metamod)
    participant Host as HostRuntime
    participant Watcher as ConfigWatcher
    participant PluginMgr as PluginManager
    participant Guest as WASM Guest Plugins

    Engine->>Backend: StartFrame / DispatchThink
    Backend->>Host: HostRuntime::on_server_frame()
    Host->>PluginMgr: with_manager()
    PluginMgr->>Watcher: drain_watcher_events()
    Watcher-->>PluginMgr: changed_paths (.wasm, .toml)
    PluginMgr->>Guest: call_on_frame() (all executable plugins)
    Host->>Host: reload changed configs / re-evaluate rules if needed
    Host->>Host: logging::flush()
```

### 4.2. Command & Chat Ingestion Flow

```mermaid
flowchart LR
    PlayerClient["Player Client\n(say /vip or console vipmenu)"] --> Backend
    Backend --> Dispatcher["Command Dispatcher\n(dispatcher.rs)"]
    Dispatcher --> CmdRegistry{"CommandRegistry\nLookup"}
    CmdRegistry -- "Target Plugin Paused" --> Notice["CLI Notice:\nPlugin is PAUSED"]
    CmdRegistry -- "Target Plugin Active" --> CheckCap{"Capability Check\n(admin.*, vip.*)"}
    CheckCap -- "Denied" --> Denied["Access Denied"]
    CheckCap -- "Allowed" --> Guest["WASM Plugin:\non_command(cmd, caller, args)"]
    Guest -- "Handled (true)" --> Suppress["Suppress GameDLL\n(MRES_SUPERCEDE)"]
    Guest -- "Unhandled (false)" --> Forward["Forward to GameDLL"]
```

### 4.3. Reactive Rule Orchestration Flow

```mermaid
flowchart TB
    Trigger["Game Event Trigger\n(MapChange, PlayerConnect, CvarChange)"] --> ScopeFilter{"RuleScope Filter\n(Evaluate affected scope only)"}
    ScopeFilter --> RuleOrch["RuleOrchestrator"]
    RuleOrch --> CheckOverride{"Is Plugin in\nmanual_overrides?"}
    CheckOverride -- "Yes (Admin Override)" --> Skip["Preserve Manual Admin State"]
    CheckOverride -- "No (Dynamic)" --> ASTEval["RuleEngine AST Evaluation\n(when: map, players, time, cvar)"]
    ASTEval --> EdgeDetect{"Edge Detected?\n(State changed from previous)"}
    EdgeDetect -- "No Change" --> NoOp["No-Op (Suppress redundant execution)"]
    EdgeDetect -- "State Transition" --> Actions["Execute Rule Actions\n(pause, unpause, set_cvar, group)"]
    Actions --> UpdatePlugins["Update Plugin Status & Recalculate DAG"]
```

---

## 5. Architectural Principles

1. **Zero Hardcoded Environment Paths**:
   All filesystem interactions must resolve paths dynamically through `PathResolver` relative to game directory (`cstrike/`, `valve/`) or localized `.goldsrc.local.toml`.
2. **Panic Boundary & Sandbox Isolation**:
   - Host Rust panics must never cross the C-ABI boundary (all entry points guarded with `catch_ffi_panic`).
   - WASM guest panics are isolated via `catch_unwind` and epoch interruption (preventing infinite loops from hanging HLDS). A crashed plugin becomes `Poisoned` without destabilizing the server process.
3. **Deterministic Dependency Ordering (`PhasedDag`)**:
   Plugin execution order, ECS systems, and event listeners strictly resolve via Kahn's topological sort with phased stratification (`Core` $\to$ `Service` $\to$ `Gameplay` $\to$ `Addon` $\to$ `Analytics`). Magic priority integers are forbidden.
4. **Game-Agnostic Core**:
   `goldsrc-core` and `goldsrc-api` contain zero game-specific assumptions (no CS 1.6 specific weapons, teams, or buyzone rules). Mod-specific features reside in dedicated extension crates (e.g. `goldsrc-game-cstrike`).
5. **Defensive Resource Management & Narrow Lock Scopes**:
   Re-entrant mutex calls are actively guarded (`HostRuntime::with_manager`). Long operations (rule evaluation, file reading) drop locks before execution.
