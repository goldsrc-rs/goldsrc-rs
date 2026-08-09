#!/usr/bin/env python3
"""GoldSrc.rs — Pre-commit Script

Checks for unformatted files and runs cargo fmt.

Run this before pushing:
    python3 scripts/pre-commit.py
"""

import argparse
import subprocess
import sys
from pathlib import Path
