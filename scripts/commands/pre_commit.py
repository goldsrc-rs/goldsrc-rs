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

    import time
    t_start = time.perf_counter()

    # 1. Format check / auto-format
    t_fmt = time.perf_counter()
    print("\n[1/4] Running cargo fmt...")
    fmt_res = subprocess.run(
        ["cargo", "fmt", "--all"],
        cwd=repo_root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if fmt_res.returncode != 0:
        print(f"cargo fmt failed:\n{fmt_res.stderr}", file=sys.stderr)
        return 1
    # Stage any formatting changes
    subprocess.run(["git", "add", "-u"], cwd=repo_root, capture_output=True)
    fmt_time = time.perf_counter() - t_fmt
    print(f"  -> Format OK ({fmt_time:.2f}s, staged)")

    # 2. Linter check (Clippy)
    t_clippy = time.perf_counter()
    print("\n[2/4] Running cargo clippy...")
    clippy_cmd = ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"]
    clippy_res = subprocess.run(clippy_cmd, cwd=repo_root, text=True, env=env)
    if clippy_res.returncode != 0:
        print("cargo clippy reported errors/warnings!", file=sys.stderr)
        return 1
    clippy_time = time.perf_counter() - t_clippy
    print(f"  -> Clippy OK ({clippy_time:.2f}s)")

    # 3. Unit & Integration Tests
    t_test = time.perf_counter()
    print("\n[3/4] Running cargo test (workspace)...")
    test_cmd = ["cargo", "test", "--workspace"]
    test_res = subprocess.run(test_cmd, cwd=repo_root, text=True, env=env)
    if test_res.returncode != 0:
        print("cargo test failed!", file=sys.stderr)
        return 1
    test_time = time.perf_counter() - t_test
    print(f"  -> Tests OK ({test_time:.2f}s)")

    # 4. WASM Plugins Build Check
    t_wasm = time.perf_counter()
    print("\n[4/4] Checking WASM plugins compilation...")
    wasm_cmd = [
        "cargo",
        "check",
        "--target",
        "wasm32-unknown-unknown",
        "-p", "admin_system",
        "-p", "test_hud",
        "-p", "test_menu",
        "-p", "test_ecs",
        "-p", "vip_core",
    ]
    wasm_res = subprocess.run(wasm_cmd, cwd=repo_root, text=True, env=env)
    if wasm_res.returncode != 0:
        print("WASM plugins check failed!", file=sys.stderr)
        return 1
    wasm_time = time.perf_counter() - t_wasm
    print(f"  -> WASM plugins OK ({wasm_time:.2f}s)")

    total_time = time.perf_counter() - t_start
    print(f"\n========================================")
    print(f"       Pre-Commit Time Breakdown        ")
    print(f"========================================")
    print(f"  • Cargo fmt     : {fmt_time:.2f}s")
    print(f"  • Cargo clippy  : {clippy_time:.2f}s")
    print(f"  • Cargo test    : {test_time:.2f}s")
    print(f"  • WASM check    : {wasm_time:.2f}s")
    print(f"  --------------------------------------")
    print(f"  • Total Time    : {total_time:.2f}s")
    print(f"========================================")
    print("\n[SUCCESS] All pre-commit checks passed successfully!\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
