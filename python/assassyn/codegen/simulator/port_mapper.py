"""Port index mapping for Vec-based array writes.

This module manages port index assignment for arrays during code generation,
enabling compile-time port calculation for optimal performance.
"""

from collections import defaultdict


class PortIndexManager:
    """Manages port index assignment for arrays during code generation."""

    def __init__(self):
        # Map: (array_name, module_name) -> port_index
        self.port_map = {}
        # Map: array_name -> next available index
        self.next_index = defaultdict(int)
        # Map: array_name -> total port count
        self.port_counts = defaultdict(int)

    def get_or_assign_port(self, array_name: str, module_name: str) -> int:
        """Get or assign a port index for a module writing to an array.

        Args:
            array_name: Name of the array being written to
            module_name: Name of the module performing the write

        Returns:
            Port index (0, 1, 2, ...) for this array-module combination
        """
        key = (array_name, module_name)

        if key not in self.port_map:
            # Assign new port index
            idx = self.next_index[array_name]
            self.port_map[key] = idx
            self.next_index[array_name] += 1
            self.port_counts[array_name] = self.next_index[array_name]

        return self.port_map[key]

    def get_port_count(self, array_name: str) -> int:
        """Get the total number of ports needed for an array.

        Args:
            array_name: Name of the array

        Returns:
            Total number of ports needed (minimum 1)
        """
        # Minimum 1 port, even if no writes detected
        return max(1, self.port_counts[array_name])


# Global singleton for the current compilation
# pylint: disable=invalid-name
_port_manager = None


def get_port_manager():
    """Get the global port manager instance.

    Returns:
        The global PortIndexManager instance
    """
    global _port_manager  # pylint: disable=global-statement
    if _port_manager is None:
        _port_manager = PortIndexManager()
    return _port_manager


def reset_port_manager():
    """Reset the port manager (useful for tests and new compilations)."""
    global _port_manager  # pylint: disable=global-statement
    _port_manager = PortIndexManager()
