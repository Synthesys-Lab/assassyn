"""Assassyn's python frontend."""

from . import frontend
from . import utils
from . import backend
from . import ir
from . import builder as _builder

# Keep heavyweight optional integrations lazy.  Importing the base package should
# not require the Ramulator2 C wrapper when a design only uses the core frontend
# and backend.
