#!/usr/bin/env python3
"""GoldSrc.rs — Modular Environment & Reference Setup Script

Clones required reference SDKs, sets up tools, manages git hooks, detects system SDK paths,
and provides verification and revert/deletion operations.

Usage:
    python -m scripts setup                      # Full standard setup (SDKs + references + tools)
    python -m scripts setup --verify             # Verify current environment status
    python -m scripts setup --delete             # Revert/delete all references and build configs
    python -m scripts setup --delete --hooks     # Uninstall git hooks only
    python -m scripts setup --delete --repos hlsdk # Delete only specific reference repo
    python -m scripts setup --hooks              # Install git hooks into .git/hooks/
    python -m scripts setup --sdk                # Detect system SDKs only and update .build-config.toml
    python -m scripts setup --repos hlsdk rehlds # Clone only specific repositories
    python -m scripts setup --tools              # Install/update local Python tools only
"""

import argparse
import fnmatch
import os
import platform
import shutil
import stat
import subprocess
import sys
from pathlib import Path


def get_repo_root() -> Path:
    return Path(__file__).resolve().parent.parent.parent


def remove_readonly(func, path, _excinfo):
    """Error handler for shutil.rmtree to handle read-only files on Windows."""
    os.chmod(path, stat.S_IWRITE)
    func(path)


def delete_directory(path: Path) -> bool:
    """Safely delete a directory if it exists."""
    if path.exists():
        try:
            shutil.rmtree(path, onexc=remove_readonly)
            print(f"  [DELETED] {path.name}")
            return True
        except Exception as e:
            print(f"  [ERROR] Failed to delete {path.name}: {e}", file=sys.stderr)
            return False
    else:
        print(f"  [SKIP] {path.name} does not exist")
        return False


def clone_repo(
    url: str,
    dest: Path,
    force: bool = False,
    shallow: bool = True,
    sparse_paths: list[str] | None = None,
) -> bool:
    """Clone a git repository, optionally using sparse-checkout for minimal download."""
    if dest.exists():
        if force:
            print(f"  [FORCE] Removing existing {dest.name}...")
            shutil.rmtree(dest, onexc=remove_readonly)
        else:
            print(f"  [SKIP] {dest.name} already exists")
            return False

    print(f"  [CLONE] {dest.name} from {url}...")
    if sparse_paths:
        cmd = ["git", "clone", "--filter=blob:none", "--no-checkout"]
        if shallow:
            cmd += ["--depth", "1"]
        cmd += [url, str(dest)]
        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            print(f"Error: Failed to clone {dest.name}", file=sys.stderr)
            print(result.stderr, file=sys.stderr)
            sys.exit(1)

        # Set up sparse checkout
        sparse_cmd = ["git", "-C", str(dest), "sparse-checkout", "set", "--no-cone"] + sparse_paths
        result = subprocess.run(sparse_cmd, capture_output=True, text=True)
        if result.returncode != 0:
            print(f"Error: Failed to set sparse-checkout for {dest.name}", file=sys.stderr)
            print(result.stderr, file=sys.stderr)
            sys.exit(1)

        checkout_cmd = ["git", "-C", str(dest), "checkout"]
        result = subprocess.run(checkout_cmd, capture_output=True, text=True)
        if result.returncode != 0:
            print(f"Error: Failed to checkout files for {dest.name}", file=sys.stderr)
            print(result.stderr, file=sys.stderr)
            sys.exit(1)
    else:
        cmd = ["git", "clone"]
        if shallow:
            cmd += ["--depth", "1"]
        cmd += [url, str(dest)]
        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            print(f"Error: Failed to clone {dest.name}", file=sys.stderr)
            print(result.stderr, file=sys.stderr)
            sys.exit(1)

    print(f"  [OK] {dest.name} cloned successfully")
    return True


def cleanup_hlsdk(hlsdk_dir: Path) -> None:
    """Remove HLSDK directories not needed for bindgen."""
    if not hlsdk_dir.exists():
        return
    dirs_to_remove = [
        "cl_dll", "dmc", "ricochet", "utils", "dedicated",
        "game_shared", "linux", "network", "pm_shared", "third_party",
        "external", "lib",
    ]
    for d in dirs_to_remove:
        full_path = hlsdk_dir / d
        if full_path.exists():
            try:
                shutil.rmtree(full_path, onexc=remove_readonly)
                print(f"  [REMOVED] hlsdk/{d}")
            except Exception:
                pass


def cleanup_metamod(metamod_dir: Path) -> None:
    """Remove metamod-r source files, keep only include directory."""
    if not metamod_dir.exists():
        return
    dirs_to_remove = [
        ".git", ".github",
        "metamod/src", "metamod/plugins", "metamod/build",
        "metamod/build_64", "metamod/build_64_opt", "metamod/build_opt",
        "metamod/buildbot", "metamod/docs", "metamod/extra/example/src",
    ]
    files_to_remove = [
        "build.sh", "CNAME", "version_script.lds",
        "metamod/README.md", "metamod/LICENSE",
    ]
    for d in dirs_to_remove:
        full_path = metamod_dir / d
        if full_path.exists():
            try:
                shutil.rmtree(full_path, onexc=remove_readonly)
                print(f"  [REMOVED] metamod-r/{d}")
            except Exception:
                pass
    for f in files_to_remove:
        full_path = metamod_dir / f
        if full_path.exists():
            try:
                full_path.unlink()
                print(f"  [REMOVED] metamod-r/{f}")
            except Exception:
                pass


def cleanup_rehlds(rehlds_dir: Path) -> None:
    """Remove rehlds files not needed for reference."""
    if not rehlds_dir.exists():
        return
    dirs_to_remove = [".git", ".github", "dep"]
    files_to_remove = ["build.sh", "version_script.lds"]
    for d in dirs_to_remove:
        full_path = rehlds_dir / d
        if full_path.exists():
            try:
                shutil.rmtree(full_path, onexc=remove_readonly)
                print(f"  [REMOVED] rehlds/{d}")
            except Exception:
                pass
    for f in files_to_remove:
        full_path = rehlds_dir / f
        if full_path.exists():
            try:
                full_path.unlink()
                print(f"  [REMOVED] rehlds/{f}")
            except Exception:
                pass


def cleanup_goldsrcmod_net(goldsrcmod_dir: Path) -> None:
    """Remove goldsrcmod-net files not needed for reference."""
    if not goldsrcmod_dir.exists():
        return
    dirs_to_remove = [".git", "Document", "Template"]
    for d in dirs_to_remove:
        full_path = goldsrcmod_dir / d
        if full_path.exists():
            try:
                shutil.rmtree(full_path, onexc=remove_readonly)
                print(f"  [REMOVED] goldsrcmod-net/{d}")
            except Exception:
                pass


def cleanup_amxmodx(amxmodx_dir: Path) -> None:
    """Remove amxmodx files not needed for reference."""
    if not amxmodx_dir.exists():
        return
    dirs_to_remove = [".git", ".github", "doc", "build"]
    for d in dirs_to_remove:
        full_path = amxmodx_dir / d
        if full_path.exists():
            try:
                shutil.rmtree(full_path, onexc=remove_readonly)
                print(f"  [REMOVED] amxmodx/{d}")
            except Exception:
                pass


def cleanup_regamedll(regamedll_dir: Path) -> None:
    """Remove regamedll files not needed for reference."""
    if not regamedll_dir.exists():
        return
    dirs_to_remove = [".git", ".github", "dep", "build"]
    files_to_remove = ["build.sh", "version_script.lds"]
    for d in dirs_to_remove:
        full_path = regamedll_dir / d
        if full_path.exists():
            try:
                shutil.rmtree(full_path, onexc=remove_readonly)
                print(f"  [REMOVED] regamedll/{d}")
            except Exception:
                pass
    for f in files_to_remove:
        full_path = regamedll_dir / f
        if full_path.exists():
            try:
                full_path.unlink()
                print(f"  [REMOVED] regamedll/{f}")
            except Exception:
                pass


def cleanup_reapi(reapi_dir: Path) -> None:
    """Remove reapi files not needed for reference."""
    if not reapi_dir.exists():
        return
    dirs_to_remove = [".git", ".github"]
    for d in dirs_to_remove:
        full_path = reapi_dir / d
        if full_path.exists():
            try:
                shutil.rmtree(full_path, onexc=remove_readonly)
                print(f"  [REMOVED] reapi/{d}")
            except Exception:
                pass


REDUNDANT_FILES = {
    ".gitignore", ".gitattributes", ".gitmodules", ".editorconfig",
    ".travis.yml", ".appveyor.yml", ".circleci", ".github",
    "README.md", "LICENSE", "LICENSE.md", "COPYING", "AUTHORS",
    "CHANGELOG.md", "CHANGES.md", "CONTRIBUTING.md", "Makefile",
    "CMakeLists.txt", "*.sln", "*.vcxproj", "*.vcxproj.filters",
    "*.vcxproj.user", "*.bat", "*.cmd", "*.ps1", "*.shamap",
    "*.map", "*.png", "*.jpg", "*.ico",
}


def cleanup_redundant_files(refs_dir: Path) -> None:
    """Remove redundant non-header files from all reference directories."""
    if not refs_dir.exists():
        return
    for repo_dir in refs_dir.iterdir():
        if not repo_dir.is_dir():
            continue
        for root, dirs, files in os.walk(str(repo_dir), topdown=False):
            for name in files:
                file_path = Path(root) / name
                relative = file_path.relative_to(repo_dir)
                redundant = name in REDUNDANT_FILES or any(fnmatch.fnmatch(name, pat) for pat in REDUNDANT_FILES)
                if "metamod/extra/example/include" in str(relative):
                    continue
                if repo_dir.name == "hlsdk" and str(relative).startswith(("engine/", "public/", "common/", "dlls/")):
                    continue
                if redundant:
                    try:
                        file_path.unlink()
                        print(f"  [REMOVED] {repo_dir.name}/{relative}")
                    except Exception:
                        pass
            for name in dirs:
                dir_path = Path(root) / name
                try:
                    if dir_path.exists() and not any(dir_path.iterdir()):
                        dir_path.rmdir()
                except OSError:
                    pass


def detect_windows_sdk() -> tuple[list[str], str]:
    """Detect Windows SDK, MSVC, and LLVM paths."""
    include_paths: list[str] = []
    llvm_path = ""

    # Windows SDK
    win_sdk_base = Path(r"C:\Program Files (x86)\Windows Kits\10\Include")
    if win_sdk_base.exists():
        versions = sorted(win_sdk_base.iterdir(), key=lambda p: p.name, reverse=True)
        if versions:
            win_sdk_path = versions[0]
            print(f"  [FOUND] Windows SDK {win_sdk_path}")
            for d in ["ucrt", "shared", "um"]:
                full_path = win_sdk_path / d
                if full_path.exists():
                    include_paths.append(str(full_path))

    # MSVC
    msvc_base = Path(r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC")
    if msvc_base.exists():
        versions = sorted(msvc_base.iterdir(), key=lambda p: p.name, reverse=True)
        if versions:
            msvc_path = versions[0]
            print(f"  [FOUND] MSVC {msvc_path}")
            for d in ["include", "atlmfc/include"]:
                full_path = msvc_path / d
                if full_path.exists():
                    include_paths.append(str(full_path))

    # LLVM
    llvm_dir = Path(r"C:\Program Files\LLVM\bin")
    if llvm_dir.exists():
        llvm_path = str(llvm_dir)
        print(f"  [FOUND] LLVM {llvm_path}")

    return include_paths, llvm_path


def detect_linux_paths() -> tuple[list[str], str]:
    """Detect Linux include paths."""
    include_paths: list[str] = []
    llvm_path = ""
    result = subprocess.run(["which", "clang"], capture_output=True, text=True)
    if result.returncode == 0:
        print(f"  [FOUND] clang: {result.stdout.strip()}")
    return include_paths, llvm_path


def detect_macos_paths() -> tuple[list[str], str]:
    """Detect macOS include paths."""
    include_paths: list[str] = []
    llvm_path = ""
    result = subprocess.run(["xcode-select", "-p"], capture_output=True, text=True)
    if result.returncode == 0:
        print(f"  [FOUND] Xcode CLI tools: {result.stdout.strip()}")
    return include_paths, llvm_path


def write_build_config(repo_root: Path, include_paths: list[str], llvm_path: str, server_path: str = "") -> None:
    """Write unified local config preserving any existing custom settings."""
    existing_cfg = None
    for name in [".goldsrc.local.toml", ".goldsrc.toml", "goldsrc.local.toml"]:
        p = repo_root / name
        if p.exists():
            existing_cfg = p
            break
    config_path = existing_cfg if existing_cfg else (repo_root / ".goldsrc.local.toml")
    existing_server = server_path

    # If file exists, try preserving existing [deploy] server_path if not provided
    if not existing_server and config_path.exists():
        try:
            import tomllib
        except ImportError:
            try:
                import tomli as tomllib
            except ImportError:
                tomllib = None
        if tomllib:
            try:
                data = tomllib.loads(config_path.read_text(encoding="utf-8"))
                existing_server = data.get("deploy", {}).get("server_path", "")
            except Exception:
                pass

    lines = [
        "# GoldSrc.rs Local Configuration",
        "# Generated via 'python -m scripts setup'. Gitignored — machine-specific.",
        "",
        "[build]",
        "include_paths = [",
    ]
    for i, path in enumerate(include_paths):
        escaped = path.replace("\\", "\\\\")
        comma = "," if i < len(include_paths) - 1 else ""
        lines.append(f'    "{escaped}"{comma}')
    lines.append("]")
    if llvm_path:
        escaped = llvm_path.replace("\\", "\\\\")
        lines.append(f'llvm_path = "{escaped}"')

    lines.append("")
    lines.append("[deploy]")
    if existing_server:
        escaped_server = existing_server.replace("\\", "\\\\")
        lines.append(f'server_path = "{escaped_server}"')
    else:
        lines.append('# server_path = "C:\\\\path\\\\to\\\\hlds"')
    lines.append('backend = "standalone"')
    lines.append("")

    config_path.write_text("\n".join(lines), encoding="utf-8", newline="\n")
    print(f"\nLocal configuration written to {config_path}")


def install_git_hooks(repo_root: Path) -> None:
    """Copy hooks from scripts/hooks/ to .git/hooks/."""
    hooks_src = repo_root / "scripts" / "hooks"
    hooks_dst = repo_root / ".git" / "hooks"
    if not hooks_src.exists():
        print("  [WARN] No scripts/hooks directory found.")
        return
    if not hooks_dst.exists():
        print("  [WARN] .git/hooks directory not found (not a git root?).")
        return

    print("Installing git hooks into .git/hooks/...")
    for hook_file in hooks_src.iterdir():
        if hook_file.is_file():
            dst = hooks_dst / hook_file.name
            shutil.copy2(hook_file, dst)
            try:
                dst.chmod(dst.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
            except Exception:
                pass
            print(f"  [OK] Installed hook: {hook_file.name}")


def uninstall_git_hooks(repo_root: Path) -> None:
    """Remove installed hooks from .git/hooks/."""
    hooks_src = repo_root / "scripts" / "hooks"
    hooks_dst = repo_root / ".git" / "hooks"
    if not hooks_dst.exists():
        return

    print("Uninstalling git hooks from .git/hooks/...")
    if hooks_src.exists():
        for hook_file in hooks_src.iterdir():
            target = hooks_dst / hook_file.name
            if target.exists():
                target.unlink()
                print(f"  [REMOVED] Hook: {target.name}")
            else:
                print(f"  [SKIP] Hook not present: {target.name}")


def install_python_tools(tools: list[dict], force: bool = False) -> None:
    """Install local Python tool packages in editable mode."""
    print("Setting up Python tools...")
    for tool in tools:
        p = tool["path"]
        if not p.exists() and (tool["path"].parent.parent / tool["name"]).exists():
            p = tool["path"].parent.parent / tool["name"]
        elif not p.exists() and "url" in tool and not tool.get("is_ref", False):
            clone_repo(tool["url"], p, force=force)

        if p.exists() and ((p / "pyproject.toml").exists() or (p / "setup.py").exists()):
            print(f"  [INSTALL] Installing {tool['name']} (editable mode from {p})...")
            res = subprocess.run(
                [sys.executable, "-m", "pip", "install", "-q", "-e", str(p)],
                capture_output=True,
                text=True,
            )
            if res.returncode == 0:
                print(f"  [OK] {tool['name']} installed successfully")
            else:
                print(f"  [WARN] Failed to install {tool['name']}: {res.stderr.strip()}", file=sys.stderr)


def uninstall_python_tools(tools: list[dict]) -> None:
    """Uninstall Python tool packages."""
    print("Uninstalling Python tools...")
    for tool in tools:
        if not tool.get("is_ref", False):
            print(f"  [UNINSTALL] Removing {tool['name']} via pip...")
            subprocess.run([sys.executable, "-m", "pip", "uninstall", "-y", "-q", tool["name"]], capture_output=True)
            print(f"  [OK] {tool['name']} uninstalled")


def detect_and_setup_build_tools(force: bool = False) -> None:
    """Check for sccache and wasm-opt in environment and suggest/perform installation if missing."""
    print("Checking build acceleration tools (sccache, wasm-opt)...")
    
    # 1. sccache
    sccache_path = shutil.which("sccache")
    if sccache_path:
        print(f"  [FOUND] sccache -> {sccache_path}")
    else:
        print("  [WARN] 'sccache' not found in PATH! Installing sccache via cargo...")
        try:
            res = subprocess.run(["cargo", "install", "sccache", "--locked"], capture_output=True, text=True)
            if res.returncode == 0:
                print("  [OK]   sccache installed successfully via cargo!")
            else:
                print(f"  [INFO] Could not auto-install sccache: {res.stderr.strip()[:100]}... (run manually: cargo install sccache)")
        except Exception as e:
            print(f"  [INFO] cargo install sccache skipped: {e}")

    # 2. wasm-opt
    wasm_opt_path = shutil.which("wasm-opt")
    if wasm_opt_path:
        print(f"  [FOUND] wasm-opt -> {wasm_opt_path}")
    else:
        print("  [INFO] 'wasm-opt' is optional for WASM size optimization. (Install: npm install -g wasm-opt or cargo install wasm-opt)")


def verify_setup(repo_root: Path, all_repos: list[dict], args) -> bool:
    """Verify state of references, SDK config, python tools, and git hooks selectively or fully."""
    print("=== GoldSrc.rs Environment & Reference Verification ===\n")
    all_ok = True
    has_specific = bool(args.hooks or args.sdk or args.tools or args.repos is not None)

    # 1. References verification
    if (args.repos is not None) or (not has_specific):
        if args.repos:
            target_names = [n.lower() for n in args.repos]
            repos_to_check = [r for r in all_repos if r["name"].lower() in target_names]
        else:
            repos_to_check = all_repos

        print("[1] Reference Repositories:")
        for repo in repos_to_check:
            p = repo["path"]
            exists = p.exists() and any(p.iterdir())
            status = "[OK]  " if exists else "[FAIL]"
            print(f"    {status} {repo['name']:<18} -> {p}")
            if not exists:
                all_ok = False

    # 2. Build configuration verification
    if args.sdk or (not has_specific):
        print("\n[2] Build & Local Configuration (.goldsrc.local.toml):")
        config_path = repo_root / ".goldsrc.local.toml"
        if not config_path.exists():
            config_path = repo_root / "goldsrc.local.toml"
        if config_path.exists():
            print(f"    [OK]   Found: {config_path.name}")
        else:
            print(f"    [FAIL] Missing: .goldsrc.local.toml (Run: python -m scripts setup --sdk)")
            all_ok = False

    # 3. Python tools verification
    if args.tools or (not has_specific):
        print("\n[3] Python Tools Packages:")
        for tool in all_repos:
            if not tool.get("is_ref", False):
                try:
                    mod_name = tool["name"].replace("-", "_")
                    __import__(mod_name)
                    print(f"    [OK]   {tool['name']:<18} is installed and importable")
                except ImportError:
                    print(f"    [FAIL] {tool['name']:<18} is NOT installed in current Python env")
                    all_ok = False

    # 4. Git hooks verification
    if args.hooks or (not has_specific):
        print("\n[4] Git Hooks Status:")
        hooks_src = repo_root / "scripts" / "hooks"
        hooks_dst = repo_root / ".git" / "hooks"
        if hooks_src.exists() and hooks_dst.exists():
            for h in hooks_src.iterdir():
                target = hooks_dst / h.name
                if args.hooks:
                    # When --hooks explicitly verified, it must be installed
                    status = "[OK]  " if target.exists() else "[FAIL]"
                    state = "installed" if target.exists() else "NOT installed"
                    if not target.exists():
                        all_ok = False
                else:
                    status = "[OK]  " if target.exists() else "[INFO]"
                    state = "installed" if target.exists() else "not installed (optional: python -m scripts setup --hooks)"
                print(f"    {status} {h.name:<18} is {state}")

    # 5. Build Tools (sccache, wasm-opt)
    if not has_specific:
        print("\n[5] Build Tools:")
        sccache_bin = shutil.which("sccache")
        status_sccache = "[OK]  " if sccache_bin else "[WARN]"
        desc_sccache = sccache_bin if sccache_bin else "NOT in PATH (run: cargo install sccache)"
        print(f"    {status_sccache} {'sccache':<18} -> {desc_sccache}")

        wasm_opt_bin = shutil.which("wasm-opt")
        status_wasm_opt = "[OK]  " if wasm_opt_bin else "[INFO]"
        desc_wasm_opt = wasm_opt_bin if wasm_opt_bin else "NOT in PATH (optional: npm i -g wasm-opt)"
        print(f"    {status_wasm_opt} {'wasm-opt':<18} -> {desc_wasm_opt}")

    print("\n" + ("=" * 55))
    if all_ok:
        print("  Verification PASSED!")
    else:
        print("  Some checks FAILED!")
    print("=" * 55)
    return all_ok


def revert_setup(repo_root: Path, all_repos: list[dict], args) -> None:
    """Revert/delete references, tools, hooks, or build config based on flags."""
    print("=== Reverting / Deleting Environment Setup ===\n")
    has_specific = bool(args.hooks or args.sdk or args.tools or args.repos is not None)

    # 1. Repos deletion
    if (args.repos is not None) or (not has_specific):
        if args.repos:
            target_names = [n.lower() for n in args.repos]
            repos_to_del = [r for r in all_repos if r["name"].lower() in target_names]
        else:
            repos_to_del = all_repos

        print("Deleting repositories...")
        for r in repos_to_del:
            delete_directory(r["path"])

    # 2. Tools deletion
    if args.tools or (not has_specific):
        uninstall_python_tools(all_repos)

    # 3. SDK config deletion
    if args.sdk or (not has_specific):
        config_path = repo_root / "goldsrc.local.toml"
        if config_path.exists():
            config_path.unlink()
            print(f"  [DELETED] {config_path.name}")
        else:
            print(f"  [SKIP] {config_path.name} does not exist")

    # 4. Hooks deletion
    if args.hooks or (not has_specific):
        uninstall_git_hooks(repo_root)

    print("\nRevert / deletion complete.")


def main(argv=None):
    parser = argparse.ArgumentParser(description="GoldSrc.rs Modular Environment Setup")
    parser.add_argument("-f", "--force", action="store_true", help="Re-clone even if directories exist")
    parser.add_argument("--no-shallow", action="store_true", help="Full clone (skip --depth 1)")
    parser.add_argument("--verify", action="store_true", help="Verify environment, SDKs, references, and tools status")
    parser.add_argument(
        "--delete",
        "--revert",
        action="store_true",
        dest="delete",
        help="Delete/revert references, tools, hooks, or config (can combine with --repos, --hooks, --tools, --sdk)",
    )
    parser.add_argument("--hooks", action="store_true", help="Manage git hooks (install, verify, or delete)")
    parser.add_argument("--sdk", action="store_true", help="Manage SDK paths / goldsrc.local.toml (detect or delete)")
    parser.add_argument("--tools", action="store_true", help="Manage Python tool packages (install or delete)")
    parser.add_argument("--server", type=str, default="", help="Optional game/server path to write into goldsrc.local.toml")
    parser.add_argument("--cleanup", action="store_true", help="Run cleanup on reference directories")
    parser.add_argument("--skip-cleanup", action="store_true", help="Skip reference cleanup during setup")
    parser.add_argument(
        "--ci",
        action="store_true",
        help="Ultra-fast CI setup: minimal sparse-checkout of only required C headers (hlsdk + metamod-r), skip optional repos and python tools",
    )
    parser.add_argument(
        "--repos",
        nargs="*",
        default=None,
        metavar="NAME",
        help="Target specific repositories: hlsdk metamod-r rehlds goldsrcmod-net amxmodx regamedll reapi crash-analyzer",
    )
    args = parser.parse_args(argv)

    repo_root = get_repo_root()
    refs_dir = repo_root / "references"
    scripts_dir = repo_root / "scripts"

    # Define all managed repositories with optional sparse checkout paths
    all_repos = [
        {
            "name": "hlsdk",
            "url": "https://github.com/alliedmodders/hlsdk.git",
            "path": refs_dir / "hlsdk",
            "is_ref": True,
            "sparse_paths": ["public", "engine", "common", "dlls"],
        },
        {
            "name": "metamod-r",
            "url": "https://github.com/theAsmodai/metamod-r.git",
            "path": refs_dir / "metamod-r",
            "is_ref": True,
            "sparse_paths": ["metamod/extra/example/include/metamod", "metamod/extra/example/include"],
        },
        {"name": "rehlds", "url": "https://github.com/s1lentq/ReHLDS.git", "path": refs_dir / "rehlds", "is_ref": True},
        {"name": "regamedll", "url": "https://github.com/s1lentq/ReGameDLL_CS.git", "path": refs_dir / "regamedll", "is_ref": True, "sparse_paths": ["regamedll"]},
        {"name": "reapi", "url": "https://github.com/s1lentq/reapi.git", "path": refs_dir / "reapi", "is_ref": True, "sparse_paths": ["reapi/include"]},
        {"name": "goldsrcmod-net", "url": "https://github.com/DrAbcOfficial/GoldSrcMod.Net.git", "path": refs_dir / "goldsrcmod-net", "is_ref": True},
        {"name": "amxmodx", "url": "https://github.com/alliedmodders/amxmodx.git", "path": refs_dir / "amxmodx", "is_ref": True},
        {"name": "crash-analyzer", "url": "https://github.com/ulquiorracode/crash-analyzer.git", "path": scripts_dir / "crash-analyzer", "is_ref": False},
    ]

    # Operation: Verify
    if args.verify:
        ok = verify_setup(repo_root, all_repos, args)
        sys.exit(0 if ok else 1)

    # Operation: Revert / Delete
    if args.delete:
        revert_setup(repo_root, all_repos, args)
        return

    # Operation: Setup (CI, Standard, or Selective)
    if args.ci:
        print("Running ultra-fast CI setup (sparse-checkout of minimal headers)...")
        refs_dir.mkdir(exist_ok=True)
        ci_target_names = ["hlsdk", "metamod-r"]
        ci_repos = [r for r in all_repos if r["name"].lower() in ci_target_names]
        for repo in ci_repos:
            clone_repo(
                repo["url"],
                repo["path"],
                force=args.force,
                shallow=not args.no_shallow,
                sparse_paths=repo.get("sparse_paths"),
            )

        print("\nDetecting system SDK paths...")
        system = platform.system()
        include_paths, llvm_path = (
            detect_windows_sdk() if system == "Windows" else
            detect_linux_paths() if system == "Linux" else
            detect_macos_paths() if system == "Darwin" else ([], "")
        )
        write_build_config(repo_root, include_paths, llvm_path, server_path=args.server)
        print("\n[SUCCESS] CI setup complete.")
        return

    has_selective_action = bool(args.hooks or args.sdk or args.tools or args.cleanup or args.repos is not None)

    # 1. Repositories Cloning
    should_clone = (args.repos is not None) or (not has_selective_action)
    if should_clone:
        if args.repos:
            target_names = [n.lower() for n in args.repos]
            repos_to_clone = [r for r in all_repos if r["name"].lower() in target_names]
        else:
            repos_to_clone = all_repos

        refs_dir.mkdir(exist_ok=True)
        print("Cloning repositories...")
        for repo in repos_to_clone:
            clone_repo(
                repo["url"],
                repo["path"],
                force=args.force,
                shallow=not args.no_shallow,
                sparse_paths=repo.get("sparse_paths") if args.repos else None,
            )

    # 2. Cleanup of Reference Directories
    should_cleanup = args.cleanup or (should_clone and not args.skip_cleanup)
    if should_cleanup:
        print("\nCleaning up reference directories...")
        cleanup_hlsdk(refs_dir / "hlsdk")
        cleanup_metamod(refs_dir / "metamod-r")
        cleanup_rehlds(refs_dir / "rehlds")
        cleanup_regamedll(refs_dir / "regamedll")
        cleanup_reapi(refs_dir / "reapi")
        cleanup_goldsrcmod_net(refs_dir / "goldsrcmod-net")
        cleanup_amxmodx(refs_dir / "amxmodx")
        cleanup_redundant_files(refs_dir)

    # 3. Python Tool Packages Installation
    should_install_tools = args.tools or (not has_selective_action)
    if should_install_tools:
        print()
        install_python_tools(all_repos, force=args.force)

    # 4. System SDK Detection
    should_detect_sdk = args.sdk or args.server or (not has_selective_action)
    if should_detect_sdk:
        print("\nDetecting system SDK paths...")
        system = platform.system()
        include_paths, llvm_path = (
            detect_windows_sdk() if system == "Windows" else
            detect_linux_paths() if system == "Linux" else
            detect_macos_paths() if system == "Darwin" else ([], "")
        )
        write_build_config(repo_root, include_paths, llvm_path, server_path=args.server)

    # 5. Build Acceleration Tools (sccache, wasm-opt)
    if not has_selective_action or args.tools:
        print()
        detect_and_setup_build_tools(force=args.force)

    # 6. Git Hooks Installation (Only if explicitly requested)
    if args.hooks:
        print()
        install_git_hooks(repo_root)

    print("\nSetup complete.")
    system = platform.system()
    if system == "Windows":
        print("You can now run: cargo build --target i686-pc-windows-msvc")
    else:
        print("You can now run: cargo build --target i686-unknown-linux-gnu")


if __name__ == "__main__":
    main()
