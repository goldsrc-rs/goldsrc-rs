# GoldSrc.rs Official Project Templates

This directory contains production-ready project templates for developers building plugins, mods, and systems on top of [GoldSrc.rs](https://github.com/goldsrc-rs/goldsrc-rs).

## Available Templates

| Template | Target / Host | Description |
| :--- | :--- | :--- |
| [`plugin-rust`](./plugin-rust) | Native (`cdylib`) | Standard GoldSrc engine plugin written in modern Rust with `goldsrc` SDK and full CI/CD. |

## Usage

### 1. Using `cargo-generate`
If you have `cargo-generate` installed:
```bash
cargo generate --git https://github.com/goldsrc-rs/goldsrc-rs.git templates/plugin-rust --name my_goldsrc_plugin
```

### 2. Manual Clone or Copy
Simply copy the desired template directory:
```bash
cp -r templates/plugin-rust/ my_plugin/
cd my_plugin
```
Replace placeholder markers (`{{project-name}}`, `{{authors}}`) in `Cargo.toml` and source files.

### 3. Using GoldSrc.rs CLI (Planned)
```bash
goldsrc new my_plugin --template plugin-rust
```
