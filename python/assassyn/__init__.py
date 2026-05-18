"""Assassyn's Python frontend package."""

from importlib import import_module
from types import ModuleType

from . import backend
from . import builder
from . import frontend
from . import ir
from . import utils

# Optional integrations stay out of __all__ so wildcard imports do not load C
# wrappers or require external simulator libraries.
__all__ = [
    "backend",
    "builder",
    "frontend",
    "ir",
    "utils",
]


def __getattr__(name: str) -> ModuleType:
    """Load optional top-level integrations without affecting base imports."""
    if name == "ramulator2":
        module = import_module(f"{__name__}.ramulator2")
        globals()[name] = module
        return module
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
