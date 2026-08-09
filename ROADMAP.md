# GoldSrc.rs Roadmap

## Stage 1: Foundation & FFI

- [ ] Set up Cargo Workspace and CI/CD (build for `i686-pc-windows-msvc` and `i686-unknown-linux-gnu`).
- [ ] Collect reference headers in `references/` (HLSDK, `meta_api.h`).
- [ ] Write `build.rs` for `goldsrc-sys` that generates Rust structs from C++ headers via `bindgen`.
- [ ] Export entry-point functions `GiveFnptrsToDll` and `Meta_Attach` so Metamod can load our Rust library.

## Stage 2: Metamod Backend & Safe Abstractions

- [ ] Wrap logging (`SERVER_PRINT`, `ALERT`). Server should print "Hello from Rust!" to console.
- [ ] Wrap basic engine structures: `edict_t`, `entvars_t`, `CBaseEntity`.
- [ ] Implement hooks for basic events (`DispatchSpawn`, `ClientConnect`, `ClientCommand`) via Metamod.
- [ ] Build VTable-hook system (using offsets from ReHLDS/HamSandwich for Windows/Linux compatibility).

## Stage 3: WebAssembly Plugin Host (Isolation & Hot-Reload)

- [ ] Integrate `wasm3` runtime into the core.
- [ ] Design WASI bindings for Wasm plugins to communicate with the core.
- [ ] Implement hot-reload: watch `.wasm` files in `plugins/` and reload on change.
- [ ] Write a test Wasm plugin (e.g., a plugin that kills a player on command).

## Stage 4: Developer Framework (DX)

- [ ] Create `goldsrc` crate with procedural macros.
- [ ] Command router.
- [ ] Database API (reuse `sqlx`, `tokio`).
- [ ] Documentation and `cargo generate` templates.

## Stage 5: Standalone Backend (Future)

- [ ] Build a custom `mp.dll` loader that bypasses the original Metamod.
- [ ] Direct interception of `hlds.exe` / `hlds_linux` interfaces.
