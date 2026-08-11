"""Build script for GoldSrc.rs plugins."""

import os
import subprocess
import sys
from pathlib import Path


def get_repo_root() -> Path:
    return Path(__file__).parent.parent


def build_plugin(target: str = "i686-pc-windows-msvc", release: bool = True) -> Path:
    """Build a plugin and return the path to the produced library."""
    print(f"Building for {target} ({'release' if release else 'debug'})...")
    repo_root = get_repo_root()

    env = os.environ.copy()
    env["LIBCLANG_PATH"] = r"C:\Program Files\LLVM\lib"

    cmd = ["cargo", "build", "--target", target, "-p", "goldsrc-metamod-backend"]
    if release:
        cmd.append("--release")

    result = subprocess.run(cmd, cwd=repo_root, env=env, capture_output=True, text=True)

    if result.returncode != 0:
        print("Build failed:", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)

    # Determine the output file extension
    if "windows" in target:
        lib_name = "goldsrc_metamod_backend.dll"
    else:
        lib_name = "libgoldsrc_metamod_backend.so"

    profile = "release" if release else "debug"
    lib_path = repo_root / "target" / target / profile / lib_name

    if not lib_path.exists():
        print(f"Error: Library not found at {lib_path}", file=sys.stderr)
        sys.exit(1)

    print(f"Built: {lib_path}")
    return lib_path


def build_wasm_plugins(release: bool = False) -> list[Path]:
    """Build all WASM plugins for wasm32-unknown-unknown."""
    print(f"Building WASM plugins ({'release' if release else 'debug'})...")
    repo_root = get_repo_root()

    cmd = ["cargo", "build", "--target", "wasm32-unknown-unknown", "-p", "vip_core", "-p", "vip_menu"]
    if release:
        cmd.append("--release")

    result = subprocess.run(cmd, cwd=repo_root, capture_output=True, text=True)

    if result.returncode != 0:
        print("WASM plugin build failed:", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)

    profile = "release" if release else "debug"
    wasm_dir = repo_root / "target" / "wasm32-unknown-unknown" / profile
    plugins = [p for p in wasm_dir.glob("*.wasm") if p.is_file()]

    print(f"Built {len(plugins)} WASM plugins: {[p.name for p in plugins]}")
    return plugins


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Build GoldSrc.rs plugin")
    parser.add_argument("--target", default="i686-pc-windows-msvc", help="Build target")
    parser.add_argument("--debug", action="store_true", help="Build in debug mode")
    args = parser.parse_args()

    build_plugin(target=args.target, release=not args.debug)
