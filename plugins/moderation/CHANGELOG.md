# Changelog - goldsrc:moderation

## [0.17.0] - 2026-10-02

### Added

- Complete rewrite replacing legacy `admin_system` prototype.
- Implementation of PBAC capabilities for all moderation primitives.
- Multilingual localization support (English and Russian) via `resources/lang/moderation.toml`.
- Dynamic moderator control panel (`grs_modmenu`) with player target selector.
- In-memory punishment repository with automatic expiration cleanup.
- Typed `ModerationError` with localized error string formatting.
- Type-safe `CvarFlags` and `range` clamping in `ModerationConfig`.
