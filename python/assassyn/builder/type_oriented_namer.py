"""
Type-Oriented Namer for the Assassyn Naming System.

Generates semantically meaningful names based on IR node types and operations.
"""

from __future__ import annotations
import re
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
            'ArrayRead': 'rd',
            'ArrayWrite': 'wt',
            'Array': 'arr',
            'FIFOPop': 'pop',
            'FIFOPush': 'push',
            'Bind': 'bind',
            'AsyncCall': 'call',
            'Concat': 'concat',
            'Select': 'select',
            'Select1Hot': 'sel1h',
            'Slice': 'slice',
            'Cast': 'cast'
        }

    @staticmethod
    def _sanitize(text: str) -> str:
        """Sanitize text into a valid identifier-like token."""
        return re.sub(r'[^0-9a-zA-Z_]+', '_', text).strip('_') or 'val'

    @staticmethod
    def _safe_getattr(node: Any, attr: str) -> Optional[Any]:
        """Safely fetch an attribute without triggering __getattr__ side-effects."""
        try:
            return object.__getattribute__(node, attr)
        except (AttributeError, TypeError):
            return None

    def _entity_name(self, entity: Any) -> Optional[str]:
        """Extract a meaningful name from an entity."""
        if entity is None:
            return None

        try:
            from assassyn.utils import unwrap_operand  # pylint: disable=import-outside-toplevel
            entity = unwrap_operand(entity)
        except ImportError:
            pass

        semantic = None
        entity_dict = getattr(entity, '__dict__', None)
        if isinstance(entity_dict, dict):
            semantic = entity_dict.get('__assassyn_semantic_name__')
        if not semantic and hasattr(entity, '__slots__'):
            try:
                semantic = object.__getattribute__(entity, '__assassyn_semantic_name__')
            except AttributeError:
                semantic = None

        if isinstance(semantic, str) and semantic:
            return self._sanitize(semantic)

        name_attr = self._safe_getattr(entity, 'name')
        if isinstance(name_attr, str) and name_attr:
            return self._sanitize(name_attr)

        return None

    def _module_prefix(self, node: Any) -> str:
        """Generate a prefix for Module-like objects."""
        class_name = node.__class__.__name__
        base = self._sanitize(class_name)
        if base in {'module', 'modulebase'}:
            base = 'module'
        return f'{base}_inst'

    def get_prefix_for_type(self, node: Any) -> str:  # pylint: disable=too-many-return-statements,too-many-branches,too-many-locals
        """Get the naming prefix for a given node type."""
        class_name = node.__class__.__name__
        mro_names = {base.__name__ for base in node.__class__.__mro__}

        if 'ModuleBase' in mro_names:
            return self._module_prefix(node)

        if 'FIFOPop' in mro_names:
            fifo_name = self._entity_name(self._safe_getattr(node, 'fifo'))
            if fifo_name:
                return fifo_name

        if 'FIFOPush' in mro_names:
            fifo_name = self._entity_name(self._safe_getattr(node, 'fifo'))
            if fifo_name:
                return f'push_{fifo_name}'

        if class_name == 'PureIntrinsic':
            opcode = self._safe_getattr(node, 'opcode')
            ops_map = getattr(node, 'OPERATORS', {})
            op_suffix = ops_map.get(opcode)
            args = self._safe_getattr(node, 'args') or []
            base_name = None
            if args:
                base_name = self._entity_name(args[0])
            if base_name and op_suffix:
                return self._sanitize(f'{base_name}_{op_suffix}')
            if op_suffix:
                return self._sanitize(op_suffix)

        # Check class-based prefixes first
        if class_name in self._class_prefixes:
            prefix = self._class_prefixes[class_name]
            if class_name in ('ArrayRead', 'ArrayWrite'):
                array = getattr(node, 'array', None)
                if array is not None:
                    array_name = getattr(array, '__assassyn_semantic_name__', None) \
                        or getattr(array, 'name', None)
                    if array_name:
                        prefix = f'{prefix}_{self._sanitize(str(array_name))}'
            return prefix

        if hasattr(node, 'opcode'):
            opcode = node.opcode
            if opcode in self._binary_ops:
                return self._binary_ops[opcode]
            if opcode in self._unary_ops:
                return self._unary_ops[opcode]

        name_attr = self._safe_getattr(node, 'name')
        if isinstance(name_attr, str):
            return self._sanitize(name_attr)

        # Default fallback
        return 'val'


    def name_value(self, value: Any, hint: Optional[str] = None) -> str:
        """Generate a unique name for a value based on its type."""
        # Clean the hint to be a valid identifier
        if hint:
            hint = self._sanitize(hint)
            return self._cache.get_unique_name(hint)

        # Get type-based prefix if no hint
        prefix = self.get_prefix_for_type(value)
        prefix = self._sanitize(prefix)
        return self._cache.get_unique_name(prefix)
