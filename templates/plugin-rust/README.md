# {{project-name}}

[![CI](https://github.com/{{authors}}/{{project-name}}/actions/workflows/ci.yml/badge.svg)](https://github.com/{{authors}}/{{project-name}}/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)

A GoldSrc engine plugin written in safe, modern Rust using [GoldSrc.rs](https://github.com/goldsrc-rs/goldsrc-rs).

## Features

- Built with modern Rust (Edition 2024).
- Safe abstractions over GoldSrc engine and Metamod.
- Automated CI pipeline testing on Linux and Windows.
- Ready to be loaded on HLDS / ReHLDS servers.

## Building

```bash
cargo build --release
```

Compiled shared library will be available in `target/release/`:
- Linux: `lib{{project-name}}.so`
- Windows: `{{project-name}}.dll`

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
