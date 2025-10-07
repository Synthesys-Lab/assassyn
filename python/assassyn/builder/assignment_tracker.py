"""
Assignment Tracker for the Assassyn Naming System.

Tracks unnamed values created during expression evaluation and assigns names when
an assignment happens.
"""

from __future__ import annotations
from dataclasses import dataclass
from typing import List, Any

from .unique_name import UniqueNameCache


@dataclass
class PendingValue:
    """Represents a value waiting to be named."""
    value: Any


class AssignmentTracker:
    """
    Tracks unnamed values created during expression evaluation.
    When an assignment happens, assigns names to all pending values.
    """

    def __init__(self):
        self._pending_stack: List[PendingValue] = []
        self._name_cache = UniqueNameCache()
        self._enabled = True

    def push_value(self, value: Any):
        """Push a newly created value onto the pending stack."""
        if not self._enabled:
            return

        self._pending_stack.append(PendingValue(value))

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
            pending = self._pending_stack.pop(0)

            # Generate appropriate name based on type
            name = self._generate_name(pending.value, assigned_name)

            # Assign the name to the value if it exposes a name attribute.
            value = pending.value
            value_dict = getattr(value, '__dict__', None)

            if value_dict is not None and '_name' in value_dict:
                value._name = name  # pylint: disable=protected-access
            elif value_dict is not None and 'name' in value_dict:
                value.name = name
            else:
                # Best effort: try setting common attributes without triggering __getattr__
                for attr in ('_name', 'name'):
                    try:
                        setattr(value, attr, name)
                        break
                    except (AttributeError, AssertionError, TypeError):
                        continue

            named_values.append((name, pending.value))

        return named_values

    def _generate_name(self, value: Any, assigned_name: str) -> str:
        """Generate an appropriate name for a value."""
        # Use assigned name if available, otherwise generate from type prefix
        base_name = assigned_name or self._get_type_prefix(value)
        return self._name_cache.get_unique_name(base_name)

    def _get_type_prefix(self, value: Any) -> str:  # pylint: disable=unused-argument
        """Get a type-appropriate prefix for naming."""
        # This will be implemented by TypeOrientedNamer
        return "val"

    def disable(self):
        """Temporarily disable tracking."""
        self._enabled = False

    def enable(self):
        """Re-enable tracking."""
        self._enabled = True
