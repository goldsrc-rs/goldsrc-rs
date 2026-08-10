# GoldSrc.rs Roadmap

## Stage 1: Foundation & FFI — ✅ Complete

- [x] Set up Cargo Workspace and CI/CD (build for `i686-pc-windows-msvc` and `i686-unknown-linux-gnu`).
- [x] Collect reference headers in `references/` (HLSDK, `meta_api.h`).
- [x] Write `build.rs` for `goldsrc-sys` that generates Rust structs from C++ headers via `bindgen`.
- [x] Export entry-point functions `GiveFnptrsToDll` and `Meta_Attach` so Metamod can load our Rust library.

## Stage 2: Metamod Backend & Safe Abstractions — ✅ Complete

- [x] Wrap logging (`SERVER_PRINT`, `ALERT`). Server should print "Hello from Rust!" to console.
- [x] Wrap basic engine structures: `edict_t`, `entvars_t`, `CBaseEntity`.
- [x] Implement hooks for basic events (`DispatchSpawn`, `ClientConnect`, `ClientCommand`) via Metamod.
- [x] Build VTable-hook system (using offsets from ReHLDS/HamSandwich for Windows/Linux compatibility).

## Stage 3: WebAssembly Plugin Host (Isolation & Hot-Reload) — ✅ Complete

- [x] Integrate `wasmi` (pure-Rust interpreter runtime) into the core.
- [x] Design WASI / host bindings (`server_print`) for WASM plugins to communicate with the core.
- [x] Implement hot-reload: watch `.wasm` files in `addons/metamod-rs/plugins/` and reload on change.
- [x] Complete plugin lifecycle management (Create, Modify, Delete, Error handling, `on_unload` callback).

## Stage 4: Developer Framework & Host CLI (DX)

- [x] Host Console Management CLI (`meta-rs` / `mrs` with `lexopt`, `-a/--all` flags, pagination, multi-target).
- [ ] Create `goldsrc-macros` crate with procedural macros (`#[plugin(systems=...)]`, `#[command]`).
- [ ] SDK `goldsrc` crate with Flat / Hybrid ECS API for WASM plugins.
- [ ] Plugin DAG Dependency Resolution with SemVer validation.
- [ ] Global Event Bus (Pub/Sub) for inter-plugin communication across WASM modules.
- [ ] Database API (`sqlx`, async runtime integration) & JSON/TOML configuration watchers.

## Stage 5: Standalone Backend (Future)

- [ ] Build a custom `mp.dll` loader that bypasses the original Metamod.
- [ ] Direct interception of `hlds.exe` / `hlds_linux` interfaces.
