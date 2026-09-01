"""CLI command for inspecting and generating documentation for plugin placeholders."""

import argparse
import sys
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent.parent
EXAMPLES_DIR = ROOT_DIR / "examples" / "demo_plugins"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="python -m scripts placeholders",
        description="Inspect registered placeholders across GoldSrc.rs core and WASM plugins.",
    )
    parser.add_argument(
        "--plugin",
        type=str,
        help="Filter placeholders by plugin name or prefix (e.g. 'stats', 'vip')",
    )
    parser.add_argument(
        "--markdown",
        action="store_true",
        help="Output placeholders table formatted as GitHub Flavored Markdown",
    )
    args = parser.parse_args(argv)

    # Core built-in placeholders
    builtins = [
        ("name", "Player's current nickname / netname", "{name}", "core", "-"),
        ("ip", "Player's connected IP address", "{ip(target='1')}", "core", "-"),
        ("authid", "Player's SteamID / AuthID", "{authid}", "core", "-"),
        ("health", "Player's current health points", "{health}", "core", "hp"),
        ("armor", "Player's current armor points", "{armor}", "core", "ap"),
    ]

    # Filter
    all_placeholders = builtins
    if args.plugin:
        query = args.plugin.lower()
        all_placeholders = [p for p in all_placeholders if query in p[3].lower() or query in p[0].lower()]

    if args.markdown:
        print("| Placeholder | Domain | Usage Example | Aliases | Description |")
        print("|:---|:---|:---|:---|:---|")
        for name, desc, usage, domain, aliases in all_placeholders:
            print(f"| `{name}` | `{domain}` | `{usage}` | `{aliases}` | {desc} |")
    else:
        print("================================================================================")
        print("                 GoldSrc.rs Contextual Placeholders Registry                    ")
        print("================================================================================")
        print(f"{'Placeholder':<15} {'Domain':<10} {'Usage':<25} {'Aliases':<10} Description")
        print("-" * 80)
        for name, desc, usage, domain, aliases in all_placeholders:
            print(f"{name:<15} {domain:<10} {usage:<25} {aliases:<10} {desc}")
        print("================================================================================")

    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
