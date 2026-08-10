#!/usr/bin/env python3
"""GoldSrc.rs — Reference Setup Script

Clones all required reference repositories for building the project.
Detects system SDK paths and writes them to .build-config.toml.

Run this once after cloning the repo:
    python3 scripts/setup.py        # Windows / Linux / macOS
"""

import argparse
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path


def get_repo_root() -> Path:
    return Path(__file__).parent.parent


def remove_readonly(func, path, excinfo):
    """Error handler for shutil.rmtree to handle read-only files on Windows."""
    import stat
    os.chmod(path, stat.S_IWRITE)
    func(path)


def clone_repo(url: str, dest: Path, force: bool = False, shallow: bool = True) -> None:
    """Clone a git repository."""
    if dest.exists():
        if force:
            print(f"  [FORCE] Removing existing {dest.name}...")
            shutil.rmtree(dest, onexc=remove_readonly)
        else:
            print(f"  [SKIP] {dest.name} already exists")
            return

    print(f"  [CLONE] {dest.name} from {url}...")
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


def cleanup_hlsdk(hlsdk_dir: Path) -> None:
    """Remove HLSDK directories not needed for bindgen."""
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
            except PermissionError:
                print(f"  [SKIP] hlsdk/{d} (permission denied)")


def cleanup_metamod(metamod_dir: Path) -> None:
    """Remove metamod-r source files, keep only include directory."""
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
            except PermissionError:
                print(f"  [SKIP] metamod-r/{d} (permission denied)")
    for f in files_to_remove:
        full_path = metamod_dir / f
        if full_path.exists():
            try:
                full_path.unlink()
                print(f"  [REMOVED] metamod-r/{f}")
            except PermissionError:
                print(f"  [SKIP] metamod-r/{f} (permission denied)")


def cleanup_rehlds(rehlds_dir: Path) -> None:
    """Remove rehlds files not needed for reference."""
    dirs_to_remove = [
        ".git", ".github", "dep",
    ]
    files_to_remove = [
        "build.sh", "version_script.lds",
    ]
    for d in dirs_to_remove:
        full_path = rehlds_dir / d
        if full_path.exists():
            try:
                shutil.rmtree(full_path, onexc=remove_readonly)
                print(f"  [REMOVED] rehlds/{d}")
            except PermissionError:
                print(f"  [SKIP] rehlds/{d} (permission denied)")
    for f in files_to_remove:
        full_path = rehlds_dir / f
        if full_path.exists():
            try:
                full_path.unlink()
                print(f"  [REMOVED] rehlds/{f}")
            except PermissionError:
                print(f"  [SKIP] rehlds/{f} (permission denied)")


def cleanup_goldsrcmod_net(goldsrcmod_dir: Path) -> None:
    """Remove goldsrcmod-net files not needed for reference."""
    dirs_to_remove = [
        ".git", "Document", "Template",
    ]
    for d in dirs_to_remove:
        full_path = goldsrcmod_dir / d
        if full_path.exists():
            try:
                shutil.rmtree(full_path, onexc=remove_readonly)
                print(f"  [REMOVED] goldsrcmod-net/{d}")
            except PermissionError:
                print(f"  [SKIP] goldsrcmod-net/{d} (permission denied)")


# Files that are not needed for bindgen or reference
REDUNDANT_FILES = {
    ".gitignore",
    ".gitattributes",
    ".gitmodules",
    ".editorconfig",
    ".travis.yml",
    ".appveyor.yml",
    ".circleci",
    ".github",
    "README.md",
    "LICENSE",
    "LICENSE.md",
    "COPYING",
    "AUTHORS",
    "CHANGELOG.md",
    "CHANGES.md",
    "CONTRIBUTING.md",
    "Makefile",
    "CMakeLists.txt",
    "*.sln",
    "*.vcxproj",
    "*.vcxproj.filters",
    "*.vcxproj.user",
    "*.bat",
    "*.cmd",
    "*.ps1",
    "*.shamap",
    "*.map",
    "*.png",
    "*.jpg",
    "*.ico",
}


def cleanup_redundant_files(refs_dir: Path) -> None:
    """Remove redundant files from all reference directories."""
    import fnmatch

    for repo_dir in refs_dir.iterdir():
        if not repo_dir.is_dir():
            continue

        for root, dirs, files in os.walk(str(repo_dir), topdown=False):
            for name in files:
                file_path = Path(root) / name
                relative = file_path.relative_to(repo_dir)

                # Check against redundant file patterns
                redundant = name in REDUNDANT_FILES
                if not redundant:
                    for pattern in REDUNDANT_FILES:
                        if fnmatch.fnmatch(name, pattern):
                            redundant = True
                            break

                # Keep files in the include directory for metamod-r
                if "metamod/extra/example/include" in str(relative):
                    continue
                # Keep HLSDK header/source files
                if repo_dir.name == "hlsdk" and str(relative).startswith(("engine/", "public/", "common/", "dlls/")):
                    continue

                if redundant:
                    file_path.unlink()
                    print(f"  [REMOVED] {repo_dir.name}/{relative}")

            # Remove empty directories
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

    # Check for clang
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


def write_build_config(repo_root: Path, include_paths: list[str], llvm_path: str) -> None:
    """Write .build-config.toml."""
    config_path = repo_root / ".build-config.toml"

    lines = [
        "# GoldSrc.rs build configuration",
        "# Generated by scripts/setup.py — do not edit manually.",
        "# This file is gitignored — each developer runs setup on their own machine.",
        "",
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

    config_path.write_text("\n".join(lines), encoding="utf-8", newline="\n")
    print(f"\nBuild config written to {config_path}")


def main():
    parser = argparse.ArgumentParser(description="GoldSrc.rs reference setup script")
    parser.add_argument("-f", "--force", action="store_true", help="Re-clone even if directories exist")
    parser.add_argument("--no-shallow", action="store_true", help="Full clone (skip --depth 1)")
    parser.add_argument("--no-delete", action="store_true", help="Do not delete redundant directories and files")
    args = parser.parse_args()

    repo_root = get_repo_root()
    refs_dir = repo_root / "references"

    # Create references directory
    refs_dir.mkdir(exist_ok=True)

    # Define repositories
    repos = [
        {"name": "hlsdk", "url": "https://github.com/alliedmodders/hlsdk.git", "path": refs_dir / "hlsdk"},
        {"name": "metamod-r", "url": "https://github.com/theAsmodai/metamod-r.git", "path": refs_dir / "metamod-r"},
        {"name": "rehlds", "url": "https://github.com/s1lentq/ReHLDS.git", "path": refs_dir / "rehlds"},
        {"name": "goldsrcmod-net", "url": "https://github.com/DrAbcOfficial/GoldSrcMod.Net.git", "path": refs_dir / "goldsrcmod-net"},
    ]

    # Clone repositories
    print("Cloning reference repositories...")
    for repo in repos:
        clone_repo(repo["url"], repo["path"], force=args.force, shallow=not args.no_shallow)

    # Clean up unnecessary directories and files
    if not args.no_delete:
        print("\nCleaning up unnecessary directories...")
        cleanup_hlsdk(refs_dir / "hlsdk")
        cleanup_metamod(refs_dir / "metamod-r")
        cleanup_rehlds(refs_dir / "rehlds")
        cleanup_goldsrcmod_net(refs_dir / "goldsrcmod-net")
        cleanup_redundant_files(refs_dir)

    # Detect system SDK paths
    print("\nDetecting system SDK paths...")
    system = platform.system()
    if system == "Windows":
        include_paths, llvm_path = detect_windows_sdk()
    elif system == "Linux":
        include_paths, llvm_path = detect_linux_paths()
    elif system == "Darwin":
        include_paths, llvm_path = detect_macos_paths()
    else:
        print(f"  [WARN] Unsupported platform: {system}")
        include_paths, llvm_path = [], ""

    # Write build config
    write_build_config(repo_root, include_paths, llvm_path)

    # Print next steps
    print("\nSetup complete.")
    if system == "Windows":
        print("You can now run: cargo build --target i686-pc-windows-msvc")
    else:
        print("You can now run: cargo build --target i686-unknown-linux-gnu")


if __name__ == "__main__":
    main()
