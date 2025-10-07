"""
Naming Manager for the Assassyn Naming System.

Central coordinator for the entire naming system, managing assignment tracking,
type-based naming, and AST rewriting.
"""

from __future__ import annotations
from typing import Optional, Any, List

from .assignment_tracker import AssignmentTracker
from .type_oriented_namer import TypeOrientedNamer
from .unique_name import UniqueNameCache


class NamingManager:
    """
    Manages the overall naming system for IR values.
    Coordinates between assignment tracking, type-based naming, and AST rewriting.
    """

    def __init__(self):
        self._tracker = AssignmentTracker()
        self._namer = TypeOrientedNamer()
        self._pending_values: List[Any] = []
        self._assignment_hook_enabled = False
        self._module_name_cache = UniqueNameCache()

    def push_value(self, value: Any):
        """Track a newly created IR value."""
        if self._assignment_hook_enabled:
            self._pending_values.append(value)

        # Always name Expr objects for better IR readability
        # Import Expr here to check instanceof
        try:
            # pylint: disable=import-outside-toplevel,cyclic-import
            from assassyn.ir.expr import Expr
            if isinstance(value, Expr):
                # Immediately name the value based on its type if it doesn't have a name yet
                # Use __dict__ to avoid triggering __getattr__ on some Expr types
                attr_name = '__assassyn_semantic_name__'
                if attr_name not in getattr(value, '__dict__', {}) or \
                        value.__dict__.get(attr_name) is None:
                    type_based_name = self._namer.name_value(value)
                    self._apply_name(value, type_based_name)
        except (ImportError, AttributeError):
            # Silently fail if we can't name it
            pass

        self._tracker.push_value(value)

    def process_assignment(self, name: str, value: Any) -> Any:
        """
        Process an assignment, naming all pending values.
        Called by the rewritten assignment hook.
        """
        # Process all pending values
        if self._pending_values:
            # The last value is the one being assigned
            for pending in self._pending_values[:-1]:
                # Generate intermediate names
                intermediate_name = self._namer.name_value(pending)
                self._apply_name(pending, intermediate_name)

            # Clear pending list
            self._pending_values.clear()

        # Name the assigned value
        final_name = self._namer.name_value(value, name)
        self._apply_name(value, final_name)

        # Clear the tracker
        self._tracker.clear_and_assign(name)

        return value

    def _apply_name(self, value: Any, name: str):
        """Apply a name to a value."""
        # Apply via a special attribute to avoid conflicts with existing _name usage
        try:
            setattr(value, '__assassyn_semantic_name__', name)
        except (AttributeError, TypeError):
            # Some Python builtins (e.g. list) cannot be annotated - ignore silently
            pass

    def reset(self):
        """Reset all naming state."""
        self._tracker = AssignmentTracker()
        self._namer.reset()
        self._pending_values.clear()

    def enable_assignment_hook(self):
        """Enable the assignment rewriting hook."""
        self._assignment_hook_enabled = True

    def disable_assignment_hook(self):
        """Disable the assignment rewriting hook."""
        self._assignment_hook_enabled = False

    def get_module_name(self, base_name: str) -> str:
        """
        Get a unique module name based on the given base name.

        The name is capitalized and made unique using a counter.
        Used by the experimental frontend factory functions.
        """
        capitalized = base_name.capitalize()
        return self._module_name_cache.get_unique_name(capitalized)


# ============================================================================
# Global Integration Functions
# ============================================================================

_global_naming_manager: Optional[NamingManager] = None


def get_naming_manager() -> Optional[NamingManager]:
    """Get the current global naming manager."""
    return _global_naming_manager


def set_naming_manager(manager: Optional[NamingManager]):
    """Set the global naming manager."""
    global _global_naming_manager  # pylint: disable=global-statement
    _global_naming_manager = manager


def assassyn_assignment_hook(name: str, value: Any) -> Any:
    """
    Hook called by rewritten assignments to process naming.
    """
    manager = get_naming_manager()
    if manager:
        return manager.process_assignment(name, value)
    return value


def name_ir_node(node: Any, hint: Optional[str] = None,
                 namer: Optional[TypeOrientedNamer] = None) -> str:
    """Convenience function for direct node naming."""
    if namer is None:
        namer = TypeOrientedNamer()
    return namer.name_value(node, hint)
