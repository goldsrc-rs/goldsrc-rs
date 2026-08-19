"""Build script for GoldSrc.rs plugins."""

import os
import subprocess
import sys
from pathlib import Path


def get_repo_root() -> Path:
    return Path(__file__).resolve().parent.parent.parent


def build_plugin(backend: str = "metamod", target: str = "i686-pc-windows-msvc", release: bool = True) -> Path:
    """Build a backend plugin (metamod or standalone) and return the path to the produced library."""
    crate_name = "goldsrc-standalone" if backend == "standalone" else "goldsrc-metamod"
    lib_basename = "goldsrc_standalone" if backend == "standalone" else "goldsrc_metamod"

    print(f"Building {backend} backend ({crate_name}) for {target} ({'release' if release else 'debug'})...")
    repo_root = get_repo_root()

    env = os.environ.copy()
    if "LIBCLANG_PATH" not in env:
        default_llvm = Path(r"C:\Program Files\LLVM\lib")
        if default_llvm.exists():
            env["LIBCLANG_PATH"] = str(default_llvm)

    cmd = ["cargo", "build", "--target", target, "-p", crate_name]
    if release:
        cmd.append("--release")

    result = subprocess.run(cmd, cwd=repo_root, env=env, capture_output=True, text=True)

    if result.returncode != 0:
        print("Build failed:", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)

    # Determine the output file extension
    if "windows" in target:
        lib_name = f"{lib_basename}.dll"
    else:
        lib_name = f"lib{lib_basename}.so"

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

    cmd = [
        "cargo",
        "build",
        "--target",
        "wasm32-unknown-unknown",
        "-p",
        "vip_core",
        "-p",
        "vip_menu",
        "-p",
        "test_suite",
        "-p",
        "admin_system",
    ]
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

    import shutil
    wasm_opt_path = shutil.which("wasm-opt")
    if wasm_opt_path and release:
        for p in plugins:
            print(f"Optimizing {p.name} with wasm-opt...")
            try:
                subprocess.run([
                    wasm_opt_path, "-Oz", "--strip-debug", 
                    str(p), "-o", str(p)
                ], check=True)
            except Exception as e:
                print(f"Warning: wasm-opt failed for {p.name}: {e}")
    elif release:
        print("\n[INFO] 'wasm-opt' not found in PATH! Install it (e.g. 'npm install -g wasm-opt') to reduce WASM plugin sizes by up to 90%!\n")

    print(f"Built {len(plugins)} WASM plugins: {[p.name for p in plugins]}")
    return plugins


def main(argv=None):
    import argparse

    parser = argparse.ArgumentParser(description="Build GoldSrc.rs backend plugin")
    parser.add_argument(
        "--backend",
        choices=["metamod", "standalone"],
        default="metamod",
        help="Backend type to build (default: metamod)",
    )
    parser.add_argument("--target", default="i686-pc-windows-msvc", help="Build target")
    parser.add_argument("--debug", action="store_true", help="Build in debug mode")
    parser.add_argument("--wasm", action="store_true", help="Build only WASM plugins")
    parser.add_argument("--all", action="store_true", help="Build both backend DLL and WASM plugins")
    args = parser.parse_args(argv)

    if args.wasm:
        build_wasm_plugins(release=not args.debug)
    elif args.all:
        build_plugin(backend=args.backend, target=args.target, release=not args.debug)
        build_wasm_plugins(release=not args.debug)
    else:
        build_plugin(backend=args.backend, target=args.target, release=not args.debug)


if __name__ == "__main__":
    main()

