#!/usr/bin/env python3
"""GoldSrc.rs — Unified Project Automation CLI Entry Point."""

import sys
from pathlib import Path

# Ensure the scripts directory is in sys.path
SCRIPTS_DIR = Path(__file__).resolve().parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from core.registry import CommandRegistry


def main():
    registry = CommandRegistry()
    registry.dispatch(sys.argv[1:])


if __name__ == "__main__":
    main()
