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

Builds the Metamod backend plugin.

```bash
python scripts/build.py                    # Build release for Windows
python scripts/build.py --debug            # Build debug
python scripts/build.py --target i686-unknown-linux-gnu  # Cross-compile for Linux
```

## deploy.py

Builds and deploys the Metamod plugin to a game server.

```bash
python scripts/deploy.py                                        # Build and deploy
python scripts/deploy.py --path "C:\Games\CS 1.6 GoldClient"    # Custom path
python scripts/deploy.py --path "..." --no-build                # Deploy existing DLL
```

## pre-commit

Git pre-commit hook that auto-formats code.

```bash
# Install as git hook
cp scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```
