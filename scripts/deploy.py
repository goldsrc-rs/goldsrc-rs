#!/usr/bin/env python3
"""Deploy script for GoldSrc.rs Metamod plugin.

Builds the plugin and deploys it to a CS 1.6 / GoldSrc server.

Usage:
    python scripts/deploy.py                    # Deploy to default path
    python scripts/deploy.py --path "C:\Games\CS 1.6 GoldClient"
    python scripts/deploy.py --path "C:\Games\CS 1.6 GoldClient" --no-build  # Skip build
"""

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path


def get_repo_root() -> Path:
    return Path(__file__).parent.parent


def build_plugin(target: str = "i686-pc-windows-msvc") -> Path:
    """Build the Metamod backend plugin."""
    print(f"Building plugin for {target}...")
    repo_root = get_repo_root()

    env = os.environ.copy()
    env["LIBCLANG_PATH"] = r"C:\Program Files\LLVM\lib"

    result = subprocess.run(
        ["cargo", "build", "--target", target, "-p", "goldsrc-metamod-backend", "--release"],
        cwd=repo_root,
        env=env,
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        print("Build failed:", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)

    # Find the built DLL
    dll_path = repo_root / "target" / target / "release" / "goldsrc_metamod_backend.dll"
    if not dll_path.exists():
        print(f"Error: DLL not found at {dll_path}", file=sys.stderr)
        sys.exit(1)

    print(f"Built: {dll_path}")
    return dll_path


def deploy_plugin(dll_path: Path, game_path: Path) -> None:
    """Deploy the plugin to the game's addons directory."""
    addons_dir = game_path / "cstrike" / "addons"
    if not addons_dir.exists():
        # Try other common paths
        addons_dir = game_path / "addons"
        if not addons_dir.exists():
            print(f"Error: Addons directory not found at {addons_dir}", file=sys.stderr)
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
        print(f"Warning: plugins.ini not found at {plugins_ini}", file=sys.stderr)
        print("You may need to install Metamod-r first.")
        return

    # Read existing plugins
    content = plugins_ini.read_text(encoding="utf-8")

    # Check if our plugin is already listed
    our_entry = "metamod-rs\\metamod-rs.dll"
    if our_entry in content:
        print(f"Plugin already listed in {plugins_ini}")
        return

    # Add our plugin to the list
    # Find the end of the file and add our entry
    lines = content.strip().split("\n")
    lines.append(f"our_entry ; GoldSrc.rs Metamod Backend")

    plugins_ini.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Added to: {plugins_ini}")


def main():
    parser = argparse.ArgumentParser(description="Deploy GoldSrc.rs Metamod plugin")
    parser.add_argument(
        "--path",
        type=str,
        default=r"C:\Games\CS 1.6 GoldClient",
        help="Path to the game directory",
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
        print(f"Error: Game path not found: {game_path}", file=sys.stderr)
        sys.exit(1)

    if args.no_build:
        # Use existing DLL
        dll_path = (
            get_repo_root()
            / "target"
            / args.target
            / "release"
            / "goldsrc_metamod_backend.dll"
        )
        if not dll_path.exists():
            print(f"Error: DLL not found at {dll_path}", file=sys.stderr)
            sys.exit(1)
    else:
        dll_path = build_plugin(args.target)

    deploy_plugin(dll_path, game_path)
    print("\nDeployment complete!")
    print("Start the server and check console for '[GoldSrc.rs] Hello from Rust!'")


if __name__ == "__main__":
    main()
