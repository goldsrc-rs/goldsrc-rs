"""Git pre-commit hook implementation for GoldSrc.rs."""

import os
import subprocess
import sys
from pathlib import Path


def get_repo_root() -> Path:
    return Path(__file__).resolve().parent.parent.parent


def main(argv=None):
    repo_root = get_repo_root()
    env = dict(os.environ)
    # Limit cargo parallel linking jobs to prevent OOM on MSVC (LNK1102)
    env.setdefault("CARGO_BUILD_JOBS", "2")

    print("========================================")
    print("      GoldSrc.rs Pre-Commit Checks      ")
    print("========================================")

    # 1. Format check / auto-format
    print("\n[1/4] Running cargo fmt...")
    fmt_res = subprocess.run(["cargo", "fmt", "--all"], cwd=repo_root, capture_output=True, text=True)
    if fmt_res.returncode != 0:
        print(f"cargo fmt failed:\n{fmt_res.stderr}", file=sys.stderr)
        return 1
    # Stage any formatting changes
    subprocess.run(["git", "add", "-u"], cwd=repo_root, capture_output=True)
    print("  -> Format OK (staged)")

    # 2. Linter check (Clippy)
    print("\n[2/4] Running cargo clippy...")
    clippy_cmd = ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"]
    clippy_res = subprocess.run(clippy_cmd, cwd=repo_root, text=True, env=env)
    if clippy_res.returncode != 0:
        print("cargo clippy reported errors/warnings!", file=sys.stderr)
        return 1
    print("  -> Clippy OK")

    # 3. Unit & Integration Tests
    print("\n[3/4] Running cargo test (workspace)...")
    test_cmd = ["cargo", "test", "--workspace"]
    test_res = subprocess.run(test_cmd, cwd=repo_root, text=True, env=env)
    if test_res.returncode != 0:
        print("cargo test failed!", file=sys.stderr)
        return 1
    print("  -> Tests OK")

    # 4. WASM Plugins Build Check
    print("\n[4/4] Checking WASM plugins compilation...")
    wasm_cmd = [
        "cargo",
        "check",
        "--target",
        "wasm32-unknown-unknown",
        "-p", "admin_system",
        "-p", "test_suite",
        "-p", "vip_core",
        "-p", "vip_menu",
    ]
    wasm_res = subprocess.run(wasm_cmd, cwd=repo_root, text=True, env=env)
    if wasm_res.returncode != 0:
        print("WASM plugins check failed!", file=sys.stderr)
        return 1
    print("  -> WASM plugins OK")

    print("\n[SUCCESS] All pre-commit checks passed successfully!\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
