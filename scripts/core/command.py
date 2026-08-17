"""Core abstractions for the GoldSrc.rs CLI dispatcher."""

import sys
from typing import Callable


class Command:
    """Represents a registered CLI command with lazy-loading support."""

    def __init__(
        self,
        name: str,
        description: str,
        loader: Callable[[], Callable[[list[str]], None]],
        aliases: tuple[str, ...] = (),
        group: str = "Core Commands",
    ):
        self.name = name
        self.description = description
        self.loader = loader
        self.aliases = aliases
        self.group = group

    def run(self, argv: list[str], invoked_as: str = "") -> None:
        """Execute the command handler."""
        handler = self.loader()
        if self.name == "analyze" and invoked_as in ("dump", "module", "abi"):
            sys.argv = ["crash-analyzer", invoked_as] + argv
            handler()
        elif self.name == "analyze":
            sys.argv = ["crash-analyzer"] + argv
            handler()
        else:
            handler(argv)
