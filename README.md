# GoldSrc.rs (GoldSrc Rust Framework)

**GoldSrc.rs** — это современный фреймворк на базе Rust для создания плагинов и серверных модификаций для движка GoldSrc (Half-Life 1, Counter-Strike 1.6).

Проект призван полностью заменить устаревшие связки C++ / AMX Mod X / Pawn, предоставляя безопасную работу с памятью (Safe Rust), поддержку WebAssembly (WASM), настоящий Hot-Reloading и современный Developer Experience.

## Философия проекта

1. **Инверсия зависимостей:** Единый API `goldsrc.rs-core`, который может работать поверх старого C++ Metamod (режим совместимости), так и как самостоятельное ядро (Standalone).
2. **Нулевой Reverse Engineering:** Мы не угадываем смещения памяти. Мы используем знания, накопленные комьюнити (ReHLDS, Metamod-r, Orpheu, HamSandwich), и автоматизируем создание FFI через `rust-bindgen`.
3. **Безопасность превыше всего:** Плагины не должны ронять сервер (Segfault). В будущем плагины будут изолироваться через Wasm-песочницу (`wasmtime`).

## Архитектура (Workspace)

Проект разбит на независимые Cargo-крейты:

* `goldsrc.rs-sys` — сырые `unsafe` FFI-биндинги (сгенерированные `bindgen` из HLSDK и Metamod).
* `goldsrc.rs-api` — чистые Rust-трейты (интерфейсы) для работы с движком.
* `goldsrc.rs-metamod-backend` — реализация API в виде `.dll`/`.so` плагина для классического Metamod-r (сохраняет совместимость с Reunion, WHBlocker и др.).
* `goldsrc.rs-wasm-host` — встроенный движок для загрузки Wasm-плагинов на лету.
* `goldsrc.rs` — публичный фреймворк (SDK) для разработчиков плагинов с удобными макросами (`#[hook]`, `#[plugin]`).
