# Changelog - goldsrc:chat_director

## [0.17.0] - 2026-10-02

### Added

- Independent service crate for chat routing and rate-limiting.
- Sliding window anti-flood protection with localized warning notices.
- Say `@` staff communication pipeline routing directly to administrators.
- Multilingual localization dictionary (`resources/lang/chat_director.toml`).
- Type-safe `CvarFlags` and `range` clamping in `ChatDirectorConfig`.
