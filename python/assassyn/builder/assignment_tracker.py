"""
Assignment Tracker for the Assassyn Naming System.

Tracks unnamed values created during expression evaluation and assigns names when
an assignment happens.
"""

from __future__ import annotations
from typing import List, Any

from .unique_name import UniqueNameCache


class AssignmentTracker:
    """
    Tracks unnamed values created during expression evaluation.
    When an assignment happens, assigns names to all pending values.
    """

    def __init__(self):
        self._pending_stack: List[Any] = []
        self._name_cache = UniqueNameCache()
        self._enabled = True

    def push_value(self, value: Any):
        """Push a newly created value onto the pending stack."""
        if not self._enabled:
            return

        self._pending_stack.append(value)

    def clear_and_assign(self, assigned_name: str) -> List[Any]:
        """
        Clear the pending stack and assign names to all values.
        Returns list of named values.
        """
        if not self._enabled:
            return []

        named_values = []

        # Process all pending values
        while self._pending_stack:
            value = self._pending_stack.pop(0)
            name = self._generate_name(value, assigned_name)
            named_values.append((name, value))

        return named_values

    def _generate_name(self, value: Any, assigned_name: str) -> str:
        """Generate an appropriate name for a value."""
        # Use assigned name if available, otherwise generate from type prefix
        del value  # Value unused: kept for future enhancements
        base_name = assigned_name
        return self._name_cache.get_unique_name(base_name)

    def disable(self):
        """Temporarily disable tracking."""
        self._enabled = False

    def enable(self):
        """Re-enable tracking."""
        self._enabled = True
