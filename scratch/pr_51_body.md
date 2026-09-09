### Summary

This PR implements the comprehensive Entity-Property-Action architectural refactoring, introduces the foundational `Client` struct, vertical slice domain specifications, typestate sugar `#[refined(...)]`, and decomposes the procedural macro generation subsystem into modular submodules.

### Changes

#### 1. Core API & Domain Vertical Slice Architecture

- **Titular `struct Client` (`core/goldsrc-api/src/client/mod.rs`)**:
  - Encapsulates validated GoldSrc client slots `1..=32` (Player, Bot, HLTV proxy).
  - Establishes zero-cost Deref hierarchy: `Player` $\to$ `Client` $\to$ `Entity`.
  - Implements safe downcasting: `client.as_player() -> Option<Player>` (filtering out HLTV proxies).
  - Implements `ClientExt` with shared queries and actions (`print`, `print_chat`, `print_center`, `print_color`, `play_sound`, `team`, `name`, `lang`, `client_kind`, `is_bot`, `is_hltv`).
- **Domain Vertical Slice Specs**:
  - `entity/spec.rs`: `Spawned`, `Solid`, `Dormant` (`Spec<Entity>`) and domain aliases (`SolidEntity`, `SpawnedEntity`).
  - `client/spec.rs`: `Connected`, `Bot`, `Human`, `Hltv` (`Spec<Client>`, `Spec<Player>`), and `Alive`, `Dead`, `Spectator` (`Spec<Player>`), plus typed aliases (`LivingPlayer`, `LivingHuman`, `ConnectedClient`, `HumanClient`, `DeadPlayer`, `SpectatingPlayer`).
  - `spec.rs`: Pure type algebra combinators (`Spec`, `Refined`, `RefineExt`, `All`, `Any`, `Not`, `NoneOf`) and centralized re-exports.
- **Client Property & Action Consolidation**:
  - Implemented `PropGet<Client>` for `Name`, `Lang`, and `Team`.
  - Implemented `Action<Client>` for `Print` and `PlaySound`.
  - Implemented `AsLangCode` for `Client` and `&Client`.
  - Delegated `Player` properties and actions to `player.client()`, removing redundant method implementations.
- **Purge of `allow(unused_imports)`**:
  - Replaced all raw `#[allow(unused_imports)]` with surgical `#[cfg(target_arch = "wasm32")]` across 9 files in `goldsrc-api`.

#### 2. Procedural Macros Modularization (`goldsrc-macros`)

- **Decomposed `plugin/mod.rs` into SRP Submodules**:
  - `plugin/attr.rs`: `#[plugin(...)]` attribute and manifest parsing.
  - `plugin/command.rs`: `#[command]` attribute parsing, argument bindings, and dispatch code generation.
  - `plugin/system.rs`: `#[system]` parsing, `#[refined(...)]` parameter extraction, and ECS runner closure generation.
  - `plugin/event.rs`: `#[event]` parsing and event bus subscriber registration.
  - `plugin/menu.rs`: `#[menu_action]` parsing and callback registration.
  - `plugin/manifest.rs`: TOML metadata string builder for WASM guest manifests.
  - `plugin/mod.rs`: Clean orchestrator (~220 lines vs monolithic 870+ lines).
- **Typestate Parameter Sugar `#[refined(...)]`**:
  - Desugars `fn sys(#[refined(Alive)] p: &mut Player)` into compile-time / ECS validated `target.refine::<Alive>()` and passes `&mut *refined`.
  - Supports multiple specifications: `#[refined(Alive, Human)]` $\to$ `target.refine::<(Alive, Human)>()`.
  - Modernized `vip_core` demo to use `#[refined(Alive)] player: &mut Player` without manual runtime checks.

#### 3. Codebase Hygiene & Cleanup

- Removed duplicate dead file `core/goldsrc-core/src/logging/guest.rs` (was 100% duplicate of `framework/goldsrc/src/logging/guest.rs`).
- Removed redundant `examples/demo_plugins/vip_core/.gitignore`.

### Verification

- `cargo fmt --check`: Passed (clean formatting across workspace).
- `cargo clippy --workspace --all-targets -- -D warnings`: Passed (0 errors, 0 warnings).
- `cargo test --workspace`: Passed (137 tests passing).
- `cargo build -p vip_core --target wasm32-wasip1 --release`: Passed.
