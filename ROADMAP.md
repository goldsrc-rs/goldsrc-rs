# GoldSrc.rs Roadmap

## Этап 1: Фундамент и FFI (Foundation)

- [ ] Настроить Cargo Workspace и CI/CD (сборка под `i686-pc-windows-msvc` и `i686-unknown-linux-gnu`).
- [ ] Собрать референсы в папку `references/` (HLSDK, meta_api.h).
- [ ] Написать `build.rs` скрипт для `goldsrc-sys`, который через `bindgen` сгенерирует Rust-структуры из C++ заголовков.
- [ ] Экспортировать базовые функции `GiveFnptrsToDll` и `Meta_Attach`, чтобы Metamod смог загрузить нашу Rust-библиотеку.

## Этап 2: Metamod Backend & Safe Abstractions

- [ ] Перехват и обертка логирования (`SERVER_PRINT`, `ALERT`). Сервер должен уметь выводить "Hello from Rust!" в консоль.
- [ ] Обертка над базовыми структурами движка: `edict_t`, `entvars_t`, `CBaseEntity`.
- [ ] Реализация хуков (Hooks) базовых событий (`DispatchSpawn`, `ClientConnect`, `ClientCommand`) через функции Metamod.
- [ ] Создание системы VTable-хуков (с использованием оффсетов из ReHLDS/HamSandwich для совместимости Windows/Linux).

## Этап 3: WebAssembly Plugin Host (Изоляция и Hot-Reload)

- [ ] Интеграция крейта `wasmtime` в ядро.
- [ ] Проектирование WASI-биндингов для общения Wasm-плагинов с Ядром.
- [ ] Реализация системы Hot-Reload: автоматическое отслеживание изменений `.wasm` файлов в папке `plugins/` и их перезагрузка на лету.
- [ ] Написание первого тестового Wasm-плагина на Rust (например, плагин, убивающий игрока по команде).

## Этап 4: Разработческий Фреймворк (Developer Experience)

- [ ] Создание крейта `goldsrc.rs` с макросами процедурного программирования.
- [ ] Удобная маршрутизация команд (Command Router).
- [ ] API для работы с базой данных (переиспользование экосистемы Rust — `sqlx`, `tokio`).
- [ ] Документация и шаблоны `cargo generate` для быстрого старта новых разработчиков.

## Этап 5: Standalone Backend (Будущее)

- [ ] Разработка собственного загрузчика `mp.dll`, обходящего оригинальный Metamod.
- [ ] Прямой перехват интерфейсов `hlds.exe` / `hlds_linux`.
