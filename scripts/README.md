# Scripts

Development and deployment scripts for GoldSrc.rs.

## setup.py

Clones reference repositories and detects system SDK paths.

```bash
python scripts/setup.py                    # Clone references and detect SDK
python scripts/setup.py --force            # Re-clone everything
python scripts/setup.py --no-shallow       # Full clone (with git history)
python scripts/setup.py --no-delete        # Keep all files (don't clean up)
```

## build.py

Builds the backend plugin (Metamod or Standalone) and WASM plugins.

```bash
python scripts/build.py                                        # Build Metamod release
python scripts/build.py --backend standalone                   # Build Standalone release
python scripts/build.py --debug                                # Build debug
python scripts/build.py --target i686-unknown-linux-gnu       # Cross-compile for Linux
```

## deploy.py

Builds and deploys the backend plugin and WASM modules to a game server.

```bash
python scripts/deploy.py --path "C:\hlds"                      # Build and deploy Metamod backend
python scripts/deploy.py --backend standalone --path "C:\hlds" # Build and deploy Standalone backend
python scripts/deploy.py --path "C:\hlds" --no-build          # Deploy existing binaries
python scripts/deploy.py --path "C:\hlds" --verify            # Verify deployment
```

> **Tip:** You can avoid passing `--path` every time by setting the `GOLDSRC_SERVER_DIR` environment variable, or by creating a local uncommitted `deploy.local.toml` file in the repo root:
> ```toml
> server_path = "C:\\path\\to\\hlds"
> ```

## pre-commit

Git pre-commit hook that auto-formats code.

```bash
# Install as git hook
cp scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```
