# Changelog - goldsrc:menu_frontend

## [0.17.0] - 2026-10-02

### Added

- Modular UI navigation hub providing `/menu` entrypoint.
- Multilingual localization dictionary (`resources/lang/menu_frontend.toml`).
- Session manager enforcing click debouncing and active page tracking.
- Extensible `MenuRegistry` for dynamic sub-menu contribution.
- Type-safe `CvarFlags` and `range` clamping in `MenuFrontendConfig`.
