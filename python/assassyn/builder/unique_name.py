"""
Unique Name Cache

This module provides a cache for unique name with a given prefix.
"""

class UniqueNameCache:  # pylint: disable=too-few-public-methods
    """A cache for generating unique names with given prefixes."""

    def __init__(self):
        """Initialize a UniqueNameCache."""
        self._cache = {}

    def get_unique_name(self, prefix: str) -> str:
        """
        Get a unique name with the given prefix.

        Args:
            prefix: The prefix for the unique name.

        Returns:
            A unique name. If the prefix hasn't been used, returns the prefix itself.
            Otherwise, appends a number to make it unique.
        """
        if prefix not in self._cache:
            self._cache[prefix] = 0
            return prefix

        self._cache[prefix] += 1
        return f"{prefix}_{self._cache[prefix]}"
