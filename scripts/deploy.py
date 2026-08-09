#!/usr/bin/env python3
r"""Deploy script for GoldSrc.rs Metamod plugin.

Copies the built plugin to a game server and registers it in Metamod's plugins.ini.

Usage:
    python scripts/deploy.py                                    # Deploy to default path
    python scripts/deploy.py --path "C:\Games\CS 1.6 GoldClient"
    python scripts/deploy.py --path "..." --no-build            # Skip build step
"""

import argparse
import shutil
import sys
from pathlib import Path

# Import build function from sibling module
try:
    from build import build_plugin
except ImportError:
    # Allow running directly without package context
    sys.path.insert(0, str(Path(__file__).parent))
    from build import build_plugin


def deploy_plugin(dll_path: Path, game_path: Path) -> None:
    """Deploy the plugin to the game's addons directory."""
    # Find addons directory
    addons_dir = game_path / "cstrike" / "addons"
    if not addons_dir.exists():
        addons_dir = game_path / "addons"
        if not addons_dir.exists():
            print(f"Error: Addons directory not found", file=sys.stderr)
            print(f"  Tried: {game_path / 'cstrike' / 'addons'}", file=sys.stderr)
            print(f"  Tried: {game_path / 'addons'}", file=sys.stderr)
            sys.exit(1)

    # Create our plugin directory
    plugin_dir = addons_dir / "metamod-rs"
    plugin_dir.mkdir(exist_ok=True)

    # Copy the DLL
    dest_dll = plugin_dir / "metamod-rs.dll"
    shutil.copy2(dll_path, dest_dll)
    print(f"Copied to: {dest_dll}")

    # Update plugins.ini
    plugins_ini = addons_dir / "metamod" / "plugins.ini"
    if not plugins_ini.exists():
        print(f"\nWarning: plugins.ini not found at {plugins_ini}", file=sys.stderr)
        print("You may need to install Metamod-r first.")
        print("Add this line to plugins.ini manually:")
        print(f"  metamod-rs\\metamod-rs.dll")
        return

    # Read existing plugins
    content = plugins_ini.read_text(encoding="utf-8")

    # Check if our plugin is already listed
    our_entry = "metamod-rs\\metamod-rs.dll"
    if our_entry in content:
        print(f"Plugin already listed in {plugins_ini}")
        return

    # Add our plugin to the list
    lines = content.strip().split("\n")
    lines.append(f"{our_entry} ; GoldSrc.rs Metamod Backend v0.1.0")

    plugins_ini.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Added to: {plugins_ini}")


def main():
    parser = argparse.ArgumentParser(description="Deploy GoldSrc.rs Metamod plugin")
    parser.add_argument(
        "--path",
        type=str,
        default=r"C:\Games\CS 1.6 GoldClient",
        help="Path to the game/server directory",
    )
    parser.add_argument(
        "--no-build",
        action="store_true",
        help="Skip building (use existing DLL)",
    )
    parser.add_argument(
        "--target",
        type=str,
        default="i686-pc-windows-msvc",
        help="Build target (default: i686-pc-windows-msvc)",
    )
    args = parser.parse_args()

    game_path = Path(args.path)
    if not game_path.exists():
        print(f"Error: Path not found: {game_path}", file=sys.stderr)
        sys.exit(1)

    if args.no_build:
        # Use existing DLL
        repo_root = Path(__file__).parent.parent
        dll_path = repo_root / "target" / args.target / "release" / "goldsrc_metamod_backend.dll"
        if not dll_path.exists():
            print(f"Error: DLL not found at {dll_path}", file=sys.stderr)
            print("Run without --no-build to build first.", file=sys.stderr)
            sys.exit(1)
        print(f"Using existing DLL: {dll_path}")
    else:
        dll_path = build_plugin(target=args.target, release=True)

    deploy_plugin(dll_path, game_path)
    print("\nDeployment complete!")
    print("Start the server and check console for '[GoldSrc.rs] Hello from Rust!'")


if __name__ == "__main__":
    main()
