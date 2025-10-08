"""Assassyn's python frontend."""

from . import frontend
from . import utils
from . import backend
from . import ir
from . import ramulator2
from . import builder as _builder

# Mirror builder's public surface so callers can continue importing from
# `assassyn` directly without repeating the export list here.
__all__ = tuple(_builder.__all__)
globals().update({name: getattr(_builder, name) for name in __all__})
