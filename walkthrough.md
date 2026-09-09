# Walkthrough: Entity-Property-Action Architecture, Client Struct & Plugin Macro Modularization

## Overview

This refactoring establishes clean Domain-Driven vertical slice architecture across `goldsrc-api`, introduces the foundational `struct Client`, provides compile-time typestate specification sugar `#[refined(...)]`, cleans up code duplication and obsolete files, and decomposes the procedural macro generation subsystem in `goldsrc-macros` into modular single-responsibility submodules.

---

## Key Changes

### 1. Titular `struct Client` & Deref Hierarchy

- **`Client` struct (`core/goldsrc-api/src/client/mod.rs`)**:
  - Represents validated engine client slots `1..=32` (Player, Bot, HLTV proxy).
  - Establishes zero-cost Deref chain: `Player` $\to$ `Client` $\to$ `Entity`.
  - Added safe downcast: `client.as_player() -> Option<Player>` (filtering HLTV proxy).
  - Consolidated shared client capabilities into `ClientExt`: `print`, `print_chat`, `print_center`, `print_color`, `play_sound`, `team`, `name`, `lang`, `client_kind`, `is_bot`, `is_hltv`.
  - Delegated `Player` properties and actions directly to `player.client()`, eliminating method and hook duplication.
  - Implemented `AsLangCode` for `Client` and `&Client`.

### 2. Domain Vertical Slice Specs & Pure Type Algebra

- **Modular Specification Architecture**:
  - [`core/goldsrc-api/src/entity/spec.rs`](file:///d:/Repo/goldsrc-rs/core/goldsrc-api/src/entity/spec.rs): Domain markers (`Spawned`, `Solid`, `Dormant`) and aliases (`SolidEntity<'a>`, `SpawnedEntity<'a>`).
  - [`core/goldsrc-api/src/client/spec.rs`](file:///d:/Repo/goldsrc-rs/core/goldsrc-api/src/client/spec.rs): Domain markers (`Connected`, `Bot`, `Human`, `Hltv`, `Alive`, `Dead`, `Spectator`) and domain aliases (`LivingPlayer<'a>`, `LivingHuman<'a>`, `ConnectedClient<'a>`, `HumanClient<'a>`, `DeadPlayer<'a>`, `SpectatingPlayer<'a>`).
  - [`core/goldsrc-api/src/spec.rs`](file:///d:/Repo/goldsrc-rs/core/goldsrc-api/src/spec.rs): Pure type algebra combinators (`Spec`, `Refined`, `RefineExt`, `All`, `Any`, `Not`, `NoneOf`) and centralized re-exports.

### 3. Typestate Macro Sugar `#[refined(...)]`

- Handlers in `#[plugin]` can express specifications directly on parameters:

  ```rust
  #[system]
  fn vip_passive_regen(#[refined(Alive)] player: &mut Player) {
      // Compile-time & runtime guaranteed to only execute on living players!
  }
  ```

- Macro desugars parameter into ECS target refinement:
  `target.refine::<#specs>()` $\to$ `sys(&mut *refined)`.
- Supports multiple specifications: `#[refined(Alive, Human)]` $\to$ `(Alive, Human)`.
- Replaced manual `if player.is_alive()` in demo plugin `vip_core`.

### 4. Decomposition of `goldsrc-macros/src/plugin`

Monolithic `plugin/mod.rs` (870+ lines) was decomposed into focused, single-responsibility submodules:

- [`plugin/attr.rs`](file:///d:/Repo/goldsrc-rs/framework/goldsrc-macros/src/plugin/attr.rs): `#[plugin(...)]` attribute and manifest parsing.
- [`plugin/command.rs`](file:///d:/Repo/goldsrc-rs/framework/goldsrc-macros/src/plugin/command.rs): `#[command]` attribute parsing, argument binding generation, and registration AST.
- [`plugin/system.rs`](file:///d:/Repo/goldsrc-rs/framework/goldsrc-macros/src/plugin/system.rs): `#[system]` parsing, `#[refined(...)]` parameter extraction, and ECS runner closure generation.
- [`plugin/event.rs`](file:///d:/Repo/goldsrc-rs/framework/goldsrc-macros/src/plugin/event.rs): `#[event]` parsing and event bus subscriber registration.
- [`plugin/menu.rs`](file:///d:/Repo/goldsrc-rs/framework/goldsrc-macros/src/plugin/menu.rs): `#[menu_action]` parsing and callback registration.
- [`plugin/manifest.rs`](file:///d:/Repo/goldsrc-rs/framework/goldsrc-macros/src/plugin/manifest.rs): TOML metadata builder for WASM guest manifests.
- [`plugin/mod.rs`](file:///d:/Repo/goldsrc-rs/framework/goldsrc-macros/src/plugin/mod.rs): Clean, compact orchestrator (~220 lines).

### 5. Codebase Hygiene & Purge of Duplicates

- **Removed dead duplicate file**: `core/goldsrc-core/src/logging/guest.rs` was an exact 100% duplicate of `framework/goldsrc/src/logging/guest.rs` and unused by the host engine.
- **Removed redundant config**: `examples/demo_plugins/vip_core/.gitignore` was completely redundant with root `.gitignore`.
- **Purged `allow(unused_imports)`**: Replaced raw compiler attribute suppressions with surgical `#[cfg(target_arch = "wasm32")]` across 9 files in `goldsrc-api`.

---

## Verification Results

| Suite / Check | Command | Status |
| :--- | :--- | :--- |
| **Code Formatting** | `cargo fmt --check` | **PASS** (0 diffs) |
| **Linter / Static Analysis** | `cargo clippy --workspace --all-targets -- -D warnings` | **PASS** (0 warnings, 0 errors) |
| **Unit & Integration Tests** | `cargo test --workspace` | **PASS** (137 tests passing) |
| **WASM Guest Compilation** | `cargo build -p vip_core --target wasm32-wasip1 --release` | **PASS** |
| **GitHub Actions CI: Auto-format** | CI workflow | **PASS** (green) |
| **GitHub Actions CI: Code Formatting** | CI workflow | **PASS** (green) |
| **GitHub Actions CI: Build (Linux)** | CI workflow | **PASS** (green) |
| **GitHub Actions CI: Build (Windows)** | CI workflow | **PASS** (green) |
