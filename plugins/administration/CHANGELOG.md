# Changelog - goldsrc:administration

## [0.17.0] - 2026-10-02
### Added
- Independent coordinator crate for administrative controls.
- Role-based staff hierarchy with loose case-insensitive string parsing.
- Multilingual localization dictionary (`resources/lang/administration.toml`).
- Safe execution of engine server commands (`changelevel`, `pause`, `exec`).
- Type-safe `CvarFlags` and `range` clamping in `AdministrationConfig`.
