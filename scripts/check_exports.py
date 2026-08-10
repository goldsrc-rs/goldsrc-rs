"""Check DLL exports using dumpbin."""

import subprocess
import sys
from pathlib import Path


def find_dumpbin() -> Path:
    """Find dumpbin.exe from MSVC installation."""
    vs_path = Path(r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC")
    if not vs_path.exists():
        return Path("dumpbin")  # Fallback to PATH

    # Find the latest version
    versions = sorted(vs_path.iterdir(), key=lambda p: p.name, reverse=True)
    for version in versions:
        dumpbin = version / "bin" / "HostX64" / "x86" / "dumpbin.exe"
        if dumpbin.exists():
            return dumpbin

    return Path("dumpbin")  # Fallback to PATH


def check_exports(dll_path: Path) -> list[str]:
    """Return list of exported function names from a PE DLL using dumpbin."""
    dumpbin = find_dumpbin()
    result = subprocess.run(
        [str(dumpbin), "/exports", str(dll_path)],
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        print(f"dumpbin failed: {result.stderr}", file=sys.stderr)
        return []

    exports = []
    in_exports = False

    for line in result.stdout.split("\n"):
        line = line.strip()

        # Look for the table header
        if "ordinal hint RVA" in line:
            in_exports = True
            continue

        if in_exports and line:
            parts = line.split()
            if len(parts) >= 4 and parts[0].isdigit():
                # Format: ordinal hint RVA name [= decorated]
                name = parts[3]
                if "=" in line:
                    # Has decorated name: "GiveFnptrsToDll = _GiveFnptrsToDll@8"
                    decorated = line.split("=")[1].strip()
                    exports.append(f"{name} = {decorated}")
                else:
                    exports.append(name)

    return exports


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python check_exports.py <path-to-dll>")
        sys.exit(1)

    dll_path = Path(sys.argv[1])
    print(f"Checking: {dll_path.name}")
    exports = check_exports(dll_path)

    if not exports:
        print("  NO EXPORTS FOUND")
    else:
        print(f"  {len(exports)} exports:")
        for name in exports:
            print(f"    - {name}")
