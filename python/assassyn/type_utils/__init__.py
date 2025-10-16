"""Utility functions and decorators for Assassyn."""

# Import new type enforcement utilities
from .enforce_type import enforce_type, validate_arguments, check_type

# Import existing utilities from the main utils module
# Note: We avoid importing from ..utils to prevent circular imports
# Instead, users should import directly from assassyn.utils for existing functions
# and from assassyn.utils.enforce_type for type enforcement

__all__ = [
    # New type enforcement utilities
    'enforce_type', 'validate_arguments', 'check_type'
]
