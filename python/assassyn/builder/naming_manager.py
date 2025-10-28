"""
Naming Manager for the Assassyn Naming System.

Central coordinator for the naming system, integrating type-based naming with
the AST rewriting hooks.
"""

from __future__ import annotations
from typing import Optional, Any

from .type_oriented_namer import TypeOrientedNamer
from .unique_name import UniqueNameCache


class NamingManager:
    """
    Manages the overall naming system for IR values.
    Coordinates between type-based naming and AST rewriting.
    """

    def __init__(self):
        self._namer = TypeOrientedNamer()
        self._module_name_cache = UniqueNameCache()

    def push_value(self, value: Any):
        """Track a newly created IR value."""
        # Always name Expr objects for better IR readability
        # Import Expr here to check instanceof
        try:
            # pylint: disable=import-outside-toplevel,cyclic-import
            from assassyn.ir.expr import Expr
            if isinstance(value, Expr):
                # Immediately name the value if it doesn't have a name yet
                if value.name is None:
                    type_based_name = self._namer.name_value(value)
                    self._apply_name(value, type_based_name)
        except (ImportError, AttributeError):
            # Silently fail if we can't name it
            pass

    def process_assignment(self, name: str, value: Any) -> Any:
        """
        Process an assignment and name the assigned value.
        Called by the rewritten assignment hook.
        """
        # Name the assigned value
        final_name = self._namer.name_value(value, name)
        self._apply_name(value, final_name)

        return value

    def _apply_name(self, value: Any, name: str):
        """Apply a name to a value."""
        try:
            setattr(value, 'name', name)
        except (AttributeError, TypeError):
            # Some Python builtins cannot be annotated - ignore silently
            pass

    def assign_name(self, value: Any, hint: Optional[str] = None) -> str:
        """
        Public helper to assign a semantic name to any value-like object.

        Useful for non-expression objects (arrays, modules, etc.) that still
        participate in textual IR dumps.
        """
        name = self._namer.name_value(value, hint)
        self._apply_name(value, name)
        return name

    def get_module_name(self, base_name: str) -> str:
        """
        Get a unique module name based on the given base name.

        The name is capitalized and made unique using a counter.
        Used by builder utilities that synthesize modules programmatically.
        """
        capitalized = base_name.capitalize()
        return self._module_name_cache.get_unique_name(capitalized)

    def get_context_prefix(self) -> Optional[str]:
        """
        Get the current naming context prefix from the active module.
        Returns the module instance's name if inside a module, None otherwise.
        """
        # pylint: disable=import-outside-toplevel,cyclic-import
        from . import Singleton

        try:
            builder = Singleton.peek_builder()
            module = builder.current_module
        except RuntimeError:
            return None

        module_name = getattr(module, 'name', None)
        return module_name
