"""
Type-Oriented Namer for the Assassyn Naming System.

Generates semantically meaningful names based on IR node types and operations.
"""

from __future__ import annotations
from typing import Optional, Any

from .unique_name import UniqueNameCache


class TypeOrientedNamer:
    """Generates appropriate names for IR nodes based on their type."""

    def __init__(self):
        self._cache = UniqueNameCache()

        # Binary operation prefixes
        self._binary_ops = {
            200: 'add', 201: 'sub', 202: 'mul', 203: 'div', 204: 'mod_op',
            206: 'and_op', 207: 'or_op', 208: 'xor',
            209: 'lt', 210: 'gt', 211: 'le', 212: 'ge', 213: 'eq', 216: 'neq',
            214: 'shl', 215: 'shr'
        }

        # Unary operation prefixes
        self._unary_ops = {
            100: 'neg', 101: 'not_op'
        }

        # Class-based prefixes
        self._class_prefixes = {
            'ArrayRead': 'arr_rd',
            'ArrayWrite': 'arr_wr',
            'FIFOPop': 'pop',
            'FIFOPush': 'push',
            'Bind': 'bind',
            'AsyncCall': 'call',
            'Concat': 'concat',
            'Select': 'select',
            'Select1Hot': 'sel1h',
            'Slice': 'slice',
            'Cast': 'cast',
            'WireAssign': 'wire_assign',
            'WireRead': 'wire_rd'
        }

    def get_prefix_for_type(self, node: Any) -> str:  # pylint: disable=too-many-return-statements
        """Get the naming prefix for a given node type."""
        class_name = node.__class__.__name__

        # Check class-based prefixes first
        if class_name in self._class_prefixes:
            return self._class_prefixes[class_name]

        # Check for operation codes
        if hasattr(node, 'opcode'):
            opcode = node.opcode

            # Binary operations
            if opcode in self._binary_ops:
                return self._binary_ops[opcode]

            # Unary operations
            if opcode in self._unary_ops:
                return self._unary_ops[opcode]

        # Check for dtype-based naming
        if hasattr(node, 'dtype'):
            dtype = node.dtype
            dtype_name = dtype.__class__.__name__

            if dtype_name == 'UInt':
                return f'u{dtype.bits}'
            if dtype_name == 'Int':
                return f'i{dtype.bits}'
            if dtype_name == 'Bits':
                return f'b{dtype.bits}'
            if dtype_name == 'Float':
                return 'f32'

        # Default fallback
        return 'val'

    def name_value(self, value: Any, hint: Optional[str] = None) -> str:
        """Generate a unique name for a value based on its type."""
        # Clean the hint to be a valid identifier
        if hint:
            hint = hint.replace('-', '_').replace(' ', '_')
            # Use the hint directly (cache will handle uniqueness)
            return self._cache.get_unique_name(hint)

        # Get type-based prefix if no hint
        prefix = self.get_prefix_for_type(value)
        return self._cache.get_unique_name(prefix)

    def reset(self):
        """Reset the internal name cache."""
        self._cache = UniqueNameCache()
