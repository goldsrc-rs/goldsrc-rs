# Scripts & Automation

Unified CLI and automation scripts for building, testing, deploying, and diagnosing GoldSrc.rs.

## Unified CLI (`python -m scripts`)

You can run any workflow through the main entry point:

```bash
python -m scripts --help
```

### Commands

1. **`setup`** — Clones reference SDKs, sets up tools, verifies environment, or reverts setup:

   ```bash
   python -m scripts setup                      # Full standard setup
   python -m scripts setup --verify             # Verify all SDKs, tools, and hooks
   python -m scripts setup --verify --hooks     # Verify git hooks status only
   python -m scripts setup --verify --repos hlsdk # Verify specific reference SDK only
   python -m scripts setup --hooks              # Install git hooks into .git/hooks/
   python -m scripts setup --sdk                # Detect system SDKs and write .build-config.toml
   python -m scripts setup --repos hlsdk rehlds # Clone only specific SDKs
   python -m scripts setup --tools              # Install local Python tools only
   python -m scripts setup --delete             # Revert/delete all reference SDKs & config
   python -m scripts setup --delete --hooks     # Uninstall git hooks only
   python -m scripts setup --delete --repos hlsdk # Delete only specific SDK repo
   ```

2. **`build`** — Compiles the backend host DLL (Metamod/Standalone) and WASM plugins:

   ```bash
   python -m scripts build --backend standalone                  # Release standalone
   python -m scripts build --backend metamod --wasm              # Build only WASM plugins
   python -m scripts build --target i686-unknown-linux-gnu       # Cross-compile for Linux
   ```

3. **`deploy` / `verify`** — Builds, deploys to HLDS server, registers configs, and validates ABI:

   ```bash
   python -m scripts deploy --backend standalone --path "C:\hlds"
   python -m scripts verify --backend standalone --path "C:\hlds"
   ```

4. **`pre-commit` / `check`** — Runs format checks, Clippy, workspace tests, and WASM compilation:

   ```bash
   python -m scripts pre-commit
   ```

5. **`analyze` / `abi` / `dump` / `module`** — Integrates with `crash-analyzer` for ABI validation and crash dump inspection:

   ```bash
   # Compare and validate calling conventions:
   python -m scripts abi --proxy "target/.../goldsrc_standalone.dll" --real "C:\hlds\cstrike\dlls\mp.dll"

   # Inspect PE/ELF exports & validate GoldSrc ABI:
   python -m scripts module "target/.../goldsrc_standalone.dll" --validate-exports

   # Analyze crash dump:
   python -m scripts dump crash.mdmp --symbols-dir target/i686-pc-windows-msvc/release
   ```

> **Tip:** You can avoid passing `--path` every time by specifying `server_path` in `goldsrc.local.toml` (or via `python -m scripts setup --server "C:\path\to\server"`), or by setting the `GOLDSRC_SERVER_DIR` environment variable:
>
> ```toml
> [deploy]
> server_path = "C:\\path\\to\\server"
> ```

---

## Standalone Execution

All scripts remain compatible with standalone direct execution if preferred:

```bash
python scripts/commands/setup.py
python scripts/commands/build.py
python scripts/commands/deploy.py
python scripts/commands/pre_commit.py
```
