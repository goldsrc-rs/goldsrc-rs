"""Build script for GoldSrc.rs plugins."""

import os
import subprocess
import sys
from pathlib import Path


def get_repo_root() -> Path:
    return Path(__file__).resolve().parent.parent.parent


import time

def build_plugin(backend: str = "metamod", target: str = "i686-pc-windows-msvc", release: bool = True) -> Path:
    """Build a backend plugin (metamod or standalone) and return the path to the produced library."""
    t0 = time.perf_counter()
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

    elapsed = time.perf_counter() - t0
    print(f"Built {backend} DLL: {lib_path.name} in {elapsed:.2f}s")
    return lib_path


def discover_wasm_plugins() -> list[str]:
    """Dynamically discover all WASM cdylib plugin crates in the workspace."""
    repo_root = get_repo_root()
    plugins = []

    try:
        import tomllib
    except ImportError:
        try:
            import tomli as tomllib
        except ImportError:
            tomllib = None

    if not tomllib:
        # Fallback if tomllib is unavailable: scan examples/demo_plugins/ and plugins/
        for root, dirs, files in os.walk(repo_root):
            if "Cargo.toml" in files and ("plugins" in root or "demo_plugins" in root):
                p = Path(root)
                if p != repo_root:
                    plugins.append(p.name)
        return sorted(list(set(plugins)))

    # Parse root Cargo.toml workspace members
    root_manifest = repo_root / "Cargo.toml"
    if root_manifest.exists():
        data = tomllib.loads(root_manifest.read_text(encoding="utf-8"))
        members = data.get("workspace", {}).get("members", [])
        for member in members:
            member_manifest = repo_root / member / "Cargo.toml"
            if member_manifest.exists():
                try:
                    mdata = tomllib.loads(member_manifest.read_text(encoding="utf-8"))
                    crate_type = mdata.get("lib", {}).get("crate-type", [])
                    if "cdylib" in crate_type and ("plugins" in member or "demo_plugins" in member):
                        pkg_name = mdata.get("package", {}).get("name")
                        if pkg_name:
                            plugins.append(pkg_name)
                except Exception:
                    pass

    return sorted(list(set(plugins)))


def resolve_default_backend() -> str:
    """Resolve default backend ('metamod' or 'standalone') from .goldsrc.local.toml or environment."""
    env_backend = os.environ.get("GOLDSRC_BACKEND")
    if env_backend in ["metamod", "standalone"]:
        return env_backend

    repo_root = get_repo_root()
    try:
        import tomllib
    except ImportError:
        try:
            import tomli as tomllib
        except ImportError:
            tomllib = None

    if tomllib:
        for cfg_name in [".goldsrc.local.toml", ".goldsrc.toml", "goldsrc.local.toml"]:
            local_config = repo_root / cfg_name
            if local_config.exists():
                try:
                    data = tomllib.loads(local_config.read_text(encoding="utf-8"))
                    backend = (
                        data.get("deploy", {}).get("backend")
                        or data.get("build", {}).get("backend")
                        or data.get("backend")
                    )
                    if backend in ["metamod", "standalone"]:
                        return backend
                except Exception:
                    pass

    return "metamod"


def build_wasm_plugins(release: bool = False) -> list[Path]:
    """Build all WASM plugins for wasm32-unknown-unknown."""
    t_start = time.perf_counter()
    repo_root = get_repo_root()
    discovered_plugins = discover_wasm_plugins()

    if not discovered_plugins:
        print("No WASM plugin crates discovered in workspace.")
        return []

    print(f"Building {len(discovered_plugins)} WASM plugins ({', '.join(discovered_plugins)}) ({'release' if release else 'debug'})...")

    cmd = [
        "cargo",
        "build",
        "--target",
        "wasm32-unknown-unknown",
    ]
    for p in discovered_plugins:
        cmd.extend(["-p", p])

    if release:
        cmd.append("--release")

    t_cargo = time.perf_counter()
    result = subprocess.run(cmd, cwd=repo_root, capture_output=True, text=True)
    cargo_time = time.perf_counter() - t_cargo

    if result.returncode != 0:
        print("WASM plugin build failed:", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)

    profile = "release" if release else "debug"
    wasm_dir = repo_root / "target" / "wasm32-unknown-unknown" / profile
    plugins = [p for p in wasm_dir.glob("*.wasm") if p.is_file()]

    import shutil
    from concurrent.futures import ThreadPoolExecutor

    wasm_opt_path = shutil.which("wasm-opt")
    opt_time = 0.0
    if wasm_opt_path and release:
        t_opt = time.perf_counter()
        def optimize_wasm(p: Path) -> None:
            try:
                subprocess.run(
                    [wasm_opt_path, "-Oz", "--strip-debug", str(p), "-o", str(p)],
                    check=True,
                    capture_output=True,
                )
            except Exception as e:
                print(f"Warning: wasm-opt failed for {p.name}: {e}")

        with ThreadPoolExecutor() as executor:
            list(executor.map(optimize_wasm, plugins))
        opt_time = time.perf_counter() - t_opt
        print(f"Optimized {len(plugins)} plugins with wasm-opt in {opt_time:.2f}s")
    elif release:
        print("\n[INFO] 'wasm-opt' not found in PATH! Install it (e.g. 'npm install -g wasm-opt') to reduce WASM plugin sizes by up to 90%!\n")

    total_wasm_time = time.perf_counter() - t_start
    print(f"Built {len(plugins)} WASM plugins in {total_wasm_time:.2f}s (Cargo: {cargo_time:.2f}s, Opt: {opt_time:.2f}s)")
    return plugins


def main(argv=None):
    import argparse

    default_backend = resolve_default_backend()

    parser = argparse.ArgumentParser(description="Build GoldSrc.rs backend plugin")
    parser.add_argument(
        "--backend",
        choices=["metamod", "standalone"],
        default=default_backend,
        help=f"Backend type to build (default: {default_backend} from .goldsrc.local.toml)",
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

