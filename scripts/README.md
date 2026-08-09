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

## deploy.py

Builds and deploys the Metamod plugin to a game server.

```bash
python scripts/deploy.py                                        # Build and deploy to default path
python scripts/deploy.py --path "C:\Games\CS 1.6 GoldClient"    # Deploy to specific path
python scripts/deploy.py --path "..." --no-build                # Deploy without building
```

## pre-commit

Git pre-commit hook that auto-formats code.

```bash
# Install as git hook
cp scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```
