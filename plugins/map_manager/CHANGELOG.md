# Changelog - goldsrc:map_manager

## [0.17.0] - 2026-10-02

### Added

- Independent feature crate for map management and voting.
- Accurate timelimit tracking with engine cvar synchronization.
- Automated end-of-map vote orchestration with anti-repetition filter.
- Multilingual localization dictionary (`resources/lang/map_manager.toml`).
- Elimination of legacy `amx_` naming in favor of `grs_nextmap`.
- Type-safe `CvarFlags` and `range` clamping in `MapManagerConfig`.
