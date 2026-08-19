"""Core CLI engine package for GoldSrc.rs."""

from .command import Command
from .registry import CommandRegistry

__all__ = ["Command", "CommandRegistry"]
