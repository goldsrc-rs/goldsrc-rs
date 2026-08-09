# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial project scaffolding: Cargo workspace with 5 crates (`goldsrc-sys`, `goldsrc-api`, `goldsrc-metamod-backend`, `goldsrc-wasm-host`, `goldsrc`).
- MIT license and Rust `.gitignore`.
- HLSDK reference headers cloned as git submodule in `references/hlsdk/`.
- ReHLDS, metamod-r, and GoldSrcMod.Net reference sources cloned in `private/references/`.
- README.md and ROADMAP.md rewritten in English.

### Changed

- Default branch set to `dev` on GitHub.
- Branch protection enabled on both `main` and `dev`.
- Squash-and-merge enabled as the default merge strategy.
