#!/usr/bin/env python3
r"""Deploy script for GoldSrc.rs Metamod plugin.

Copies the built plugin to a game server and registers it in Metamod's plugins.ini or liblist.gam.

Usage:
    python scripts/deploy.py --path "/path/to/hlds"              # Deploy to specified server
    python scripts/deploy.py --backend standalone --path "..."   # Deploy standalone backend
    python scripts/deploy.py --verify                            # Verify deployment
"""

import argparse
import hashlib
import os
import shutil
import sys
from pathlib import Path

# Import build function from sibling module
try:
    from .build import build_plugin, build_wasm_plugins
except ImportError:
    # Allow running directly without package context
    sys.path.insert(0, str(Path(__file__).parent))
    from .build import build_plugin, build_wasm_plugins


def get_platform_prefix(target: str) -> str:
    """Get the Metamod platform prefix for a build target."""
    if "windows" in target:
        return "win32"
    elif "linux" in target:
        return "linux"
    else:
        # Default to win32 for unknown targets
        return "win32"


# Framework Path Constants
FRAMEWORK_NAME = "goldsrc"
DEFAULT_MOD = "cstrike"
ADDONS_DIR_NAME = "addons"


def get_dest_name(backend: str, target: str) -> str:
    """Get the destination library filename for a given backend and build target."""
    basename = "goldsrc_standalone" if backend == "standalone" else "goldsrc_metamod"
    if "windows" in target:
        return f"{basename}.dll"
    else:
        return f"lib{basename}.so"


def update_liblist_gam(game_path: Path, dest_name: str, target: str) -> None:
    """Update liblist.gam to point to goldsrc_standalone."""
    liblist_path = game_path / DEFAULT_MOD / "liblist.gam"
    if not liblist_path.exists():
        liblist_path = game_path / "liblist.gam"

    if not liblist_path.exists():
        print(f"\nWarning: liblist.gam not found at {liblist_path}", file=sys.stderr)
        print("Set gamedll in liblist.gam manually:")
        print(f"  gamedll \"goldsrc\\bin\\{dest_name}\"")
        return

    content = liblist_path.read_text(encoding="utf-8")
    is_windows = "windows" in target
    key_name = "gamedll" if is_windows else "gamedll_linux"

    if is_windows:
        expected_val = f"goldsrc\\bin\\{dest_name}"
    else:
        expected_val = f"goldsrc/bin/{dest_name}"

    expected_line = f'{key_name} "{expected_val}"'

    lines = []
    found_and_enabled = False

    # Check if there is an existing commented or active expected_line in the file
    file_lines = content.split("\n")
    
    # First pass: if expected_line already exists as active or commented, rewrite cleanly
    has_target_entry = any(expected_val in l for l in file_lines)

    if has_target_entry:
        for line in file_lines:
            stripped = line.strip()
            # If this is our target DLL (whether active or commented)
            if expected_val in stripped:
                if not found_and_enabled:
                    lines.append(expected_line)
                    found_and_enabled = True
            # If this is another active gamedll entry, comment it out
            elif (stripped.startswith(f"{key_name} ") or stripped.startswith(f"{key_name}\t")) and not stripped.startswith("//"):
                lines.append(f"// {line}  // Replaced by GoldSrc.rs deploy")
            else:
                lines.append(line)
    else:
        # Target not present in file yet: comment out existing active gamedll and prepend ours
        for line in file_lines:
            stripped = line.strip()
            if (stripped.startswith(f"{key_name} ") or stripped.startswith(f"{key_name}\t")) and not stripped.startswith("//"):
                if not found_and_enabled:
                    lines.append(expected_line)
                    found_and_enabled = True
                lines.append(f"// {line}  // Replaced by GoldSrc.rs deploy")
            else:
                lines.append(line)

        if not found_and_enabled:
            lines.append(expected_line)

    new_content = "\n".join(lines) + "\n"
    if new_content.strip() != content.strip():
        liblist_path.write_text(new_content, encoding="utf-8")
        print(f"Updated {liblist_path} with: {expected_line}")
    else:
        print(f"Standalone backend already set in {liblist_path}")


def deploy_plugin(dll_path: Path, game_path: Path, backend: str = "metamod", target: str = "i686-pc-windows-msvc") -> None:
    """Deploy the plugin backend to the game server."""
    dest_name = get_dest_name(backend, target)

    if backend == "standalone":
        # Standalone lives directly under cstrike/goldsrc/bin/ (no addons/ wrapper)
        mod_dir = game_path / DEFAULT_MOD
        if not mod_dir.exists():
            mod_dir = game_path
        plugin_dir = mod_dir / FRAMEWORK_NAME / "bin"
    else:
        # Metamod lives under cstrike/addons/goldsrc/bin/
        addons_dir = game_path / DEFAULT_MOD / ADDONS_DIR_NAME
        if not addons_dir.exists():
            addons_dir = game_path / ADDONS_DIR_NAME
            if not addons_dir.exists():
                print("Error: Metamod addons directory not found on the server!", file=sys.stderr)
                print(f"  Checked: {game_path / DEFAULT_MOD / ADDONS_DIR_NAME}", file=sys.stderr)
                print(f"  Checked: {game_path / ADDONS_DIR_NAME}", file=sys.stderr)
                print("\n-> To deploy with Metamod, ensure Metamod-r/Metamod is installed on your server.", file=sys.stderr)
                print("-> Alternatively, use Standalone backend without Metamod: python -m scripts deploy --backend standalone", file=sys.stderr)
                sys.exit(1)
        plugin_dir = addons_dir / FRAMEWORK_NAME / "bin"

    plugin_dir.mkdir(parents=True, exist_ok=True)
    dest_path = plugin_dir / dest_name

    try:
        shutil.copy2(dll_path, dest_path)
        print(f"Copied {backend} backend to: {dest_path}")
    except PermissionError:
        # DLL is locked by a running hlds.exe — try to terminate it automatically.
        print(f"\n[WARNING] {dest_path} is locked. Attempting to stop hlds.exe...", file=sys.stderr)
        import subprocess, time
        kill_result = subprocess.run(
            ["taskkill", "/F", "/IM", "hlds.exe"],
            capture_output=True, text=True
        )
        if kill_result.returncode == 0:
            print("  -> hlds.exe terminated. Retrying copy...", file=sys.stderr)
            time.sleep(1)  # give the OS a moment to release file handles
            try:
                shutil.copy2(dll_path, dest_path)
                print(f"Copied {backend} backend to: {dest_path}")
            except PermissionError:
                print(f"\n[CRITICAL ERROR] Still cannot overwrite {dest_path}!", file=sys.stderr)
                sys.exit(1)
        else:
            print(f"\n[CRITICAL ERROR] Cannot overwrite {dest_path} because the file is locked!", file=sys.stderr)
            print(">>> Please STOP/CLOSE the running HLDS server (hlds.exe) first, then run deploy.py again! <<<\n", file=sys.stderr)
            sys.exit(1)

    if backend == "standalone":
        update_liblist_gam(game_path, dest_name, target)
    else:
        # Ensure liblist.gam points to metamod when deploying metamod backend
        restore_liblist_gam_for_metamod(game_path)

        # Update plugins.ini for Metamod
        addons_dir = game_path / DEFAULT_MOD / ADDONS_DIR_NAME
        if not addons_dir.exists():
            addons_dir = game_path / ADDONS_DIR_NAME

        metamod_dir = addons_dir / "metamod"
        plugins_ini = metamod_dir / "plugins.ini"
        prefix = get_platform_prefix(target)
        expected_line = f"{prefix} addons\\goldsrc\\bin\\{dest_name}"

        # If addons/metamod exists but plugins.ini is not created yet, create it automatically
        if not plugins_ini.exists():
            if metamod_dir.exists():
                plugins_ini.write_text(f"; Metamod plugins configuration\n{expected_line}\n", encoding="utf-8")
                print(f"Created {plugins_ini} with: {expected_line}")
                return
            else:
                print(f"\nWarning: metamod directory not found at {metamod_dir}", file=sys.stderr)
                print("You may need to install Metamod-r first.")
                print("Add this line to plugins.ini manually:")
                print(f"  {expected_line}")
                return

        content = plugins_ini.read_text(encoding="utf-8")
        prefix = get_platform_prefix(target)
        expected_line = f"{prefix} addons\\goldsrc\\bin\\{dest_name}"

        lines = []
        updated = False
        for line in content.split("\n"):
            stripped = line.strip()
            if dest_name in stripped and not stripped.startswith(";"):
                if stripped == expected_line:
                    print(f"Plugin already listed in {plugins_ini}")
                    return
                lines.append(expected_line)
                updated = True
            else:
                lines.append(line)

        plugins_ini.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"Updated plugins.ini with new path: {plugins_ini}")

        # Ensure liblist.gam does not point to goldsrc_standalone when deploying metamod
        restore_liblist_gam_for_metamod(game_path)


def restore_liblist_gam_for_metamod(game_path: Path) -> None:
    """Restore liblist.gam to Metamod if it was previously modified by standalone backend."""
    liblist_path = game_path / DEFAULT_MOD / "liblist.gam"
    if not liblist_path.exists():
        liblist_path = game_path / "liblist.gam"

    if not liblist_path.exists():
        return

    content = liblist_path.read_text(encoding="utf-8")
    if "goldsrc_standalone" not in content:
        return

    lines = []
    has_metamod = False
    for line in content.split("\n"):
        stripped = line.strip()
        if "goldsrc_standalone" in stripped:
            continue
        elif "addons" in stripped and "metamod" in stripped:
            # Uncomment metamod gamedll if commented out by deploy
            clean_line = stripped.lstrip("/ \t")
            if clean_line.endswith("// Replaced by GoldSrc.rs deploy"):
                clean_line = clean_line[: -len("// Replaced by GoldSrc.rs deploy")].strip()
            lines.append(clean_line)
            has_metamod = True
        else:
            lines.append(line)

    if not has_metamod:
        lines.append('gamedll "addons\\metamod\\metamod.dll"')

    new_content = "\n".join(lines).strip() + "\n"
    if new_content != content:
        liblist_path.write_text(new_content, encoding="utf-8")
        print(f"Restored Metamod entry in {liblist_path}")


def verify_deploy(
    game_path: Path,
    dll_path: Path,
    wasm_paths: list[Path],
    backend: str = "metamod",
    target: str = "i686-pc-windows-msvc",
) -> bool:
    """Verify that the plugin backend and WASM modules are correctly deployed."""
    dest_name = get_dest_name(backend, target)

    if backend == "standalone":
        mod_dir = game_path / DEFAULT_MOD
        if not mod_dir.exists():
            mod_dir = game_path
        goldsrc_dir = mod_dir / FRAMEWORK_NAME
    else:
        addons_dir = game_path / DEFAULT_MOD / ADDONS_DIR_NAME
        if not addons_dir.exists():
            addons_dir = game_path / ADDONS_DIR_NAME
        goldsrc_dir = addons_dir / FRAMEWORK_NAME

    dest_path = goldsrc_dir / "bin" / dest_name
    wasm_target_dir = goldsrc_dir / "plugins"

    all_ok = True

    # Check 1: DLL exists
    if not dest_path.exists():
        print(f"  [FAIL] Backend DLL not found: {dest_path}")
        all_ok = False
    else:
        # Check 2: Hash matches
        src_hash = hashlib.md5(dll_path.read_bytes()).hexdigest()
        dst_hash = hashlib.md5(dest_path.read_bytes()).hexdigest()
        if src_hash == dst_hash:
            print(f"  [OK]   Backend DLL hash matches ({dst_hash[:8]}...)")
        else:
            print(f"  [FAIL] Backend DLL hash mismatch: src={src_hash[:8]} dst={dst_hash[:8]}")
            all_ok = False

        # Check 2.1: Automated Calling Convention & ABI validation
        if backend == "standalone" and "windows" in target:
            try:
                from crash_analyzer.binary import inspect_function_convention
                from crash_analyzer.cli import DEFAULT_EXPORT_CONTRACT

                mp_dll = game_path / DEFAULT_MOD / "dlls" / "mp.dll"
                for fn in DEFAULT_EXPORT_CONTRACT:
                    p_conv = inspect_function_convention(str(dest_path), fn)
                    if not p_conv:
                        print(f"  [FAIL] Required export '{fn}' is missing from backend DLL!")
                        all_ok = False
                    elif mp_dll.exists():
                        r_conv = inspect_function_convention(str(mp_dll), fn)
                        if r_conv and (p_conv["convention"] != r_conv["convention"] or p_conv["bytes_cleaned"] != r_conv["bytes_cleaned"]):
                            print(f"  [FAIL] Calling convention mismatch on '{fn}': proxy={p_conv['ret_insn']} vs real={r_conv['ret_insn']}")
                            all_ok = False
                if all_ok:
                    print("  [OK]   GoldSrc ABI contract & Calling Conventions verified")
            except ImportError:
                pass

    # Check 3: Registration in config
    if backend == "standalone":
        liblist_path = game_path / DEFAULT_MOD / "liblist.gam"
        if not liblist_path.exists():
            liblist_path = game_path / "liblist.gam"

        if not liblist_path.exists():
            print(f"  [FAIL] liblist.gam not found: {liblist_path}")
            all_ok = False
        else:
            content = liblist_path.read_text(encoding="utf-8")
            is_windows = "windows" in target
            key_name = "gamedll" if is_windows else "gamedll_linux"
            expected_val = f"goldsrc\\bin\\{dest_name}" if is_windows else f"goldsrc/bin/{dest_name}"
            expected_line = f'{key_name} "{expected_val}"'

            first_active_gamedll = None
            for line in content.split("\n"):
                stripped = line.strip()
                if (stripped.startswith(f"{key_name} ") or stripped.startswith(f"{key_name}\t")) and not stripped.startswith("//") and not stripped.startswith(";"):
                    first_active_gamedll = stripped
                    break

            if first_active_gamedll == expected_line:
                print("  [OK]   Standalone backend active in liblist.gam")
            elif first_active_gamedll:
                print(f"  [FAIL] liblist.gam active gamedll mismatch:\n         Expected: {expected_line}\n         Active:   {first_active_gamedll}")
                all_ok = False
            else:
                print(f"  [FAIL] Standalone backend ({expected_line}) not found in liblist.gam")
                all_ok = False
    else:
        addons_dir = game_path / DEFAULT_MOD / ADDONS_DIR_NAME
        if not addons_dir.exists():
            addons_dir = game_path / ADDONS_DIR_NAME
        plugins_ini = addons_dir / "metamod" / "plugins.ini"
        if not plugins_ini.exists():
            print(f"  [FAIL] plugins.ini not found: {plugins_ini}")
            all_ok = False
        else:
            content = plugins_ini.read_text(encoding="utf-8")
            prefix = get_platform_prefix(target)
            expected_line = f"{prefix} addons\\goldsrc\\bin\\{dest_name}"

            found = any(expected_line in line and not line.strip().startswith(";") for line in content.split("\n"))
            if found:
                print("  [OK]   Plugin listed in plugins.ini")
            else:
                print(f"  [FAIL] Plugin not found in plugins.ini (expected: {expected_line})")
                all_ok = False

    # Check 4: WASM plugins hashes
    for wasm_src in wasm_paths:
        wasm_dst = get_plugin_dest_path(wasm_target_dir, wasm_src)

        if not wasm_dst.exists():
            print(f"  [FAIL] WASM plugin not found: {wasm_dst}")
            all_ok = False
        else:
            src_hash = hashlib.md5(wasm_src.read_bytes()).hexdigest()
            dst_hash = hashlib.md5(wasm_dst.read_bytes()).hexdigest()
            if src_hash == dst_hash:
                print(f"  [OK]   WASM {wasm_src.name} hash matches ({dst_hash[:8]}...)")
            else:
                print(f"  [FAIL] WASM {wasm_src.name} hash mismatch: src={src_hash[:8]} dst={dst_hash[:8]}")
                all_ok = False

    return all_ok


def extract_wasm_bundle(wasm_path: Path) -> str | None:
    """Extract bundle name from embedded WASM metadata or filename conventions with strict path traversal validation."""
    try:
        data = wasm_path.read_bytes()
        # Search for bundle = "..." in the embedded metadata string
        marker = b'bundle = "'
        idx = data.find(marker)
        if idx != -1:
            start = idx + len(marker)
            end = data.find(b'"', start)
            if end != -1:
                bundle = data[start:end].decode("utf-8", errors="ignore").strip()
                # Security check: Prevent path traversal (..) and absolute paths
                if ".." in bundle or bundle.startswith("/") or bundle.startswith("\\") or ":" in bundle:
                    print(f"Warning: Rejected dangerous bundle path '{bundle}' in {wasm_path.name}")
                    return None
                # Validate characters
                import re
                if re.match(r'^[a-zA-Z0-9_-]+(/[a-zA-Z0-9_-]+)*$', bundle):
                    return bundle
    except Exception:
        pass

    # Fallback to test_ prefix convention
    if wasm_path.name.startswith("test_"):
        return "test_suite"
    return None


def get_plugin_dest_path(wasm_target_dir: Path, wasm_file: Path) -> Path:
    """Computes destination path respecting safe bundle subfolders."""
    bundle = extract_wasm_bundle(wasm_file)
    if bundle:
        dest_dir = wasm_target_dir / bundle
        dest_dir.mkdir(parents=True, exist_ok=True)
        return dest_dir / wasm_file.name
    return wasm_target_dir / wasm_file.name


def deploy_wasm_plugins(wasm_paths: list[Path], game_path: Path, backend: str = "metamod") -> None:
    """Copy WASM plugins to the server's plugins/ directory."""
    if backend == "standalone":
        mod_dir = game_path / DEFAULT_MOD
        if not mod_dir.exists():
            mod_dir = game_path
        wasm_target_dir = mod_dir / FRAMEWORK_NAME / "plugins"
    else:
        addons_dir = game_path / DEFAULT_MOD / ADDONS_DIR_NAME
        if not addons_dir.exists():
            addons_dir = game_path / ADDONS_DIR_NAME
        wasm_target_dir = addons_dir / FRAMEWORK_NAME / "plugins"

    wasm_target_dir.mkdir(parents=True, exist_ok=True)

    for wasm_file in wasm_paths:
        if wasm_file.exists():
            dest = get_plugin_dest_path(wasm_target_dir, wasm_file)
            shutil.copy2(wasm_file, dest)
            print(f"Copied WASM plugin: {dest}")


def deploy_lang_dictionaries(repo_root: Path, game_path: Path, backend: str = "metamod") -> None:
    """Copy localization dictionaries (resources/lang, examples/demo_lang) to data/lang/ directory."""
    if backend == "standalone":
        mod_dir = game_path / DEFAULT_MOD
        if not mod_dir.exists():
            mod_dir = game_path
        lang_target_dir = mod_dir / FRAMEWORK_NAME / "data" / "lang"
    else:
        addons_dir = game_path / DEFAULT_MOD / ADDONS_DIR_NAME
        if not addons_dir.exists():
            addons_dir = game_path / ADDONS_DIR_NAME
        lang_target_dir = addons_dir / FRAMEWORK_NAME / "data" / "lang"

    lang_target_dir.mkdir(parents=True, exist_ok=True)

    sources = [
        repo_root / "resources" / "lang",
        repo_root / "examples" / "demo_lang",
    ]

    for src_dir in sources:
        if src_dir.exists() and src_dir.is_dir():
            for item in src_dir.iterdir():
                dest = lang_target_dir / item.name
                if item.is_file():
                    shutil.copy2(item, dest)
                    print(f"Copied localization file: {dest}")
                elif item.is_dir():
                    if dest.exists():
                        shutil.rmtree(dest)
                    shutil.copytree(item, dest)
                    print(f"Copied localization directory: {dest}")


def resolve_game_path(cli_path: str | None, repo_root: Path) -> Path:
    """Resolve HLDS server directory path from CLI, environment, or local config."""
    # 1. Explicit CLI argument
    if cli_path:
        path = Path(cli_path)
        if path.exists():
            return path
        print(f"Error: Path provided via --path does not exist: {cli_path}", file=sys.stderr)
        sys.exit(1)

    # 2. Environment variable
    env_path = os.environ.get("GOLDSRC_SERVER_DIR") or os.environ.get("HLDS_DIR")
    if env_path:
        path = Path(env_path)
        if path.exists():
            print(f"Using server path from environment variable: {path}")
            return path

    # 3. Unified local config (.goldsrc.local.toml / .goldsrc.toml / goldsrc.local.toml)
    try:
        import tomllib
    except ImportError:
        try:
            import tomli as tomllib
        except ImportError:
            tomllib = None

    if tomllib:
        for cfg_name in [".goldsrc.local.toml", ".goldsrc.toml", "goldsrc.local.toml", "deploy.local.toml"]:
            local_config = repo_root / cfg_name
            if local_config.exists():
                try:
                    data = tomllib.loads(local_config.read_text(encoding="utf-8"))
                    server_path = (
                        data.get("deploy", {}).get("server_path")
                        if "deploy" in data
                        else data.get("server_path")
                    )
                    if server_path:
                        path = Path(server_path)
                        if path.exists():
                            print(f"Using server path from {local_config.name}: {path}")
                            return path
                        else:
                            print(f"Warning: {local_config.name} specifies server_path = \"{server_path}\", but that directory does not exist!", file=sys.stderr)
                except Exception:
                    pass

    # 4. Error if no valid path resolved
    print("Error: No game server path specified!", file=sys.stderr)
    print("\nPlease provide the server path using one of the following methods:", file=sys.stderr)
    print('  1. Pass --path argument: python -m scripts deploy --path "C:\\path\\to\\hlds"', file=sys.stderr)
    print('  2. Set environment variable: set GOLDSRC_SERVER_DIR="C:\\path\\to\\hlds"', file=sys.stderr)
    print('  3. In .goldsrc.local.toml under [deploy]: server_path = "C:\\\\path\\\\to\\\\hlds"', file=sys.stderr)
    sys.exit(1)


def main(argv=None):
    from .build import resolve_default_backend

    default_backend = resolve_default_backend()

    parser = argparse.ArgumentParser(description="Deploy GoldSrc.rs backend and WASM modules")
    parser.add_argument(
        "--path",
        type=str,
        default=None,
        help="Path to the game/server directory (or set GOLDSRC_SERVER_DIR env var / deploy.local.toml)",
    )
    parser.add_argument(
        "--backend",
        choices=["metamod", "standalone"],
        default=default_backend,
        help=f"Backend to deploy (metamod or standalone, default: {default_backend} from .goldsrc.local.toml)",
    )
    parser.add_argument(
        "--no-build",
        action="store_true",
        help="Skip building (use existing DLL & WASM binaries)",
    )
    parser.add_argument(
        "--target",
        type=str,
        default="i686-pc-windows-msvc",
        help="Build target for host DLL (default: i686-pc-windows-msvc)",
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="Verify deployment without deploying",
    )
    args = parser.parse_args(argv)

    import time
    t_deploy_total = time.perf_counter()
    repo_root = Path(__file__).resolve().parent.parent.parent
    game_path = resolve_game_path(args.path, repo_root)
    dest_name = get_dest_name(args.backend, args.target)

    dll_path = repo_root / "target" / args.target / "release" / dest_name
    wasm_dir = repo_root / "target" / "wasm32-unknown-unknown" / "release"
    wasm_plugins = [p for p in wasm_dir.glob("*.wasm") if p.is_file()]

    if args.verify:
        print(f"Verifying {args.backend} deployment...")
        if verify_deploy(game_path, dll_path, wasm_plugins, args.backend, args.target):
            print("\nAll checks passed!")
        else:
            print("\nSome checks failed!")
            sys.exit(1)
        return

    t_build_start = time.perf_counter()
    if args.no_build:
        if not dll_path.exists():
            print(f"Error: Library not found at {dll_path}", file=sys.stderr)
            sys.exit(1)
        print(f"Using existing library: {dll_path}")
        build_time = 0.0
    else:
        dll_path = build_plugin(backend=args.backend, target=args.target, release=True)
        wasm_plugins = build_wasm_plugins(release=True)
        build_time = time.perf_counter() - t_build_start

    t_copy_start = time.perf_counter()
    deploy_plugin(dll_path, game_path, backend=args.backend, target=args.target)
    deploy_wasm_plugins(wasm_plugins, game_path, backend=args.backend)
    deploy_lang_dictionaries(repo_root, game_path, backend=args.backend)
    copy_time = time.perf_counter() - t_copy_start

    print(f"\nVerifying {args.backend} deployment...")
    t_verify_start = time.perf_counter()
    verified = verify_deploy(game_path, dll_path, wasm_plugins, args.backend, args.target)
    verify_time = time.perf_counter() - t_verify_start

    if verified:
        total_time = time.perf_counter() - t_deploy_total
        print(f"\n========================================")
        print(f"       Deployment Time Breakdown        ")
        print(f"========================================")
        print(f"  • Build & Optimization : {build_time:.2f}s")
        print(f"  • Copy & Registration  : {copy_time:.2f}s")
        print(f"  • Post-Deploy Verify   : {verify_time:.2f}s")
        print(f"  --------------------------------------")
        print(f"  • Total Elapsed Time   : {total_time:.2f}s")
        print(f"========================================\n")
        print("Deployment verified successfully!")
    else:
        print("\nDeployment verification failed!")
        sys.exit(1)


if __name__ == "__main__":
    main()
