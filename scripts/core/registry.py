"""CommandRegistry and dynamic discovery engine for GoldSrc.rs CLI."""

import sys
from importlib.metadata import entry_points
from typing import Callable

from .command import Command


class CommandRegistry:
    """Dynamic registry for core commands and external plugin tools."""

    def __init__(self):
        self._commands: dict[str, Command] = {}
        self._alias_map: dict[str, str] = {}
        self._register_builtins()
        self._discover_entrypoint_plugins()

    def register(
        self,
        name: str,
        description: str,
        loader: Callable[[], Callable],
        aliases: tuple[str, ...] = (),
        group: str = "Core Commands",
    ) -> None:
        cmd = Command(name, description, loader, aliases, group)
        self._commands[name] = cmd
        for alias in aliases:
            self._alias_map[alias] = name

    def _register_builtins(self) -> None:
        """Register project-local automation scripts from commands package."""
        self.register(
            name="setup",
            description="Clone reference SDKs, detect system tools, and configure build environment",
            loader=lambda: __import__("commands.setup", fromlist=["main"]).main,
            group="Environment & SDKs",
        )
        self.register(
            name="build",
            description="Build backend host DLL (Metamod/Standalone) and/or WASM plugins",
            loader=lambda: __import__("commands.build", fromlist=["main"]).main,
            group="Build & Compilation",
        )
        self.register(
            name="deploy",
            description="Build and deploy plugins to dedicated server with registration in configs",
            loader=lambda: __import__("commands.deploy", fromlist=["main"]).main,
            group="Deployment & Testing",
        )
        self.register(
            name="verify",
            description="Verify server deployment, hash matches, and ABI calling conventions",
            loader=lambda: lambda argv: __import__("commands.deploy", fromlist=["main"]).main(["--verify"] + argv),
            group="Deployment & Testing",
        )
        self.register(
            name="pre-commit",
            description="Run format, linter, workspace tests, and WASM compilation checks",
            loader=lambda: __import__("commands.pre_commit", fromlist=["main"]).main,
            aliases=("check", "lint", "test"),
            group="Diagnostics & Tools",
        )
        self.register(
            name="exports",
            description="Inspect PE DLL export table via dumpbin",
            loader=lambda: __import__("commands.check_exports", fromlist=["main"]).main,
            aliases=("check-exports",),
            group="Diagnostics & Tools",
        )
        self.register(
            name="analyze",
            description="Inspect crashes (.mdmp / core), PE/ELF modules, and ABI (via crash-analyzer)",
            loader=self._load_crash_analyzer,
            aliases=("crash", "crash-analyzer", "dump", "module", "abi"),
            group="Diagnostics & Tools",
        )
        self.register(
            name="logo",
            description="Generate vector (SVG) and raster (PNG) brand logos in various styles",
            loader=lambda: __import__("commands.logo", fromlist=["main"]).main,
            aliases=("generate-logo", "branding"),
            group="Assets & Branding",
        )

    def _load_crash_analyzer(self) -> Callable:
        try:
            return __import__("crash_analyzer.cli", fromlist=["main"]).main
        except ImportError:
            print("Error: crash-analyzer is not installed in the current environment.", file=sys.stderr)
            print("To install it, run: python -m scripts setup", file=sys.stderr)
            sys.exit(1)

    def _discover_entrypoint_plugins(self) -> None:
        """Auto-discover third-party tools exposing the 'goldsrc.cli' entrypoint group."""
        try:
            if sys.version_info >= (3, 10):
                eps = entry_points(group="goldsrc.cli")
            else:
                eps = entry_points().get("goldsrc.cli", [])

            for ep in eps:
                if ep.name not in self._commands and ep.name not in self._alias_map:
                    self.register(
                        name=ep.name,
                        description=f"Plugin tool ({ep.value})",
                        loader=ep.load,
                        group="Installed Plugins",
                    )
        except Exception:
            pass

    def get_command(self, name: str) -> tuple[Command | None, str]:
        canonical = self._alias_map.get(name, name)
        return self._commands.get(canonical), name

    def print_help(self) -> None:
        print("GoldSrc.rs — Unified Project Automation CLI\n")
        print("Usage:")
        print("  python -m scripts <command> [options]\n")

        # Group commands for clean formatting
        groups: dict[str, list[Command]] = {}
        for cmd in self._commands.values():
            groups.setdefault(cmd.group, []).append(cmd)

        for group_name, cmds in groups.items():
            print(f"{group_name}:")
            for cmd in cmds:
                alias_str = f" (aliases: {', '.join(cmd.aliases)})" if cmd.aliases else ""
                print(f"  {cmd.name:<14} {cmd.description}{alias_str}")
            print()

        print("Run 'python -m scripts <command> --help' for detailed options on a specific command.")

    def dispatch(self, argv: list[str]) -> None:
        if not argv or argv[0] in ("-h", "--help", "help"):
            self.print_help()
            sys.exit(0)

        cmd_name = argv[0].lower()
        cmd, invoked_as = self.get_command(cmd_name)

        if cmd is None:
            print(f"Error: Unknown command '{cmd_name}'\n", file=sys.stderr)
            self.print_help()
            sys.exit(1)

        cmd.run(argv[1:], invoked_as=invoked_as)
