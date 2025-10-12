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
            200: 'add', 201: 'sub', 202: 'mul', 203: 'div', 204: 'mod',
            206: 'and', 207: 'or', 208: 'xor',
            209: 'lt', 210: 'gt', 211: 'le', 212: 'ge', 213: 'eq', 216: 'neq',
            214: 'shl', 215: 'shr'
        }

        # Unary operation prefixes
        self._unary_ops = {
            100: 'neg', 101: 'not'
        }

        # Class-based prefixes
        self._class_prefixes = {
            'ArrayRead': 'rd',
            'ArrayWrite': 'wt',
            'Array': '',
            'FIFOPop': 'pop',
            'FIFOPush': 'push',
            'Bind': 'bind',
            'AsyncCall': 'call',
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

        entity = self._unwrap_operand(entity)

        semantic = self._safe_getattr(entity, '__assassyn_semantic_name__')
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
        return f'{base}Instance'

    def _unwrap_operand(self, entity: Any) -> Any:
        """Unwrap Operand wrappers when available."""
        if entity is None:
            return None
        try:
            from assassyn.utils import unwrap_operand  # pylint: disable=import-outside-toplevel
            return unwrap_operand(entity)
        except ImportError:
            return entity

    def _describe_operand(self, operand: Any) -> Optional[str]:
        """Provide a descriptive token for an operand."""
        name = self._entity_name(operand)
        if not name:
            return None
        return self._head_token_segment(name)

    def _combine_parts(self, *parts: Optional[str]) -> Optional[str]:
        """Combine multiple name parts into a sanitized identifier."""
        tokens = []
        for part in parts:
            if isinstance(part, str) and part:
                segment = self._head_token_segment(self._sanitize(part))
                if segment:
                    tokens.append(segment)
        if not tokens:
            return None
        # Remove duplicate adjacent tokens for clarity
        deduped = []
        for token in tokens:
            if not deduped or deduped[-1] != token:
                deduped.append(token)
        combined = '_'.join(deduped)
        # Keep identifiers reasonably sized to avoid unreadable dumps
        return combined[:25]

    @staticmethod
    def _head_token_segment(token: str) -> str:
        """Keep at most the first two underscore-separated segments for brevity."""
        if not token:
            return token
        return '_'.join(token.split('_')[:2])

    def get_prefix_for_type(self, node: Any) -> str:  # pylint: disable=too-many-return-statements,too-many-branches,too-many-locals
        """Get the naming prefix for a given node type."""
        class_name = node.__class__.__name__
        mro_names = {base.__name__ for base in node.__class__.__mro__}

        if 'ModuleBase' in mro_names:
            return self._module_prefix(node)

        if class_name == 'PureIntrinsic':
            opcode = self._safe_getattr(node, 'opcode')
            op_suffix = getattr(node, 'OPERATORS', {}).get(opcode)
            args = self._safe_getattr(node, 'args') or ()

            # For FIFO operations (peek, valid), use just the port/fifo name
            if op_suffix and args:
                base_name = self._entity_name(args[0])
                if base_name:
                    # For FIFO_PEEK (303), use the base name with suffix for clarity
                    if op_suffix in ('peek', 'valid'):
                        return (
                            self._combine_parts(base_name, op_suffix) or
                            f'{base_name}_{op_suffix}'
                        )
                    return self._combine_parts(base_name, op_suffix) or f'{base_name}_{op_suffix}'

            if op_suffix:
                return self._sanitize(str(op_suffix))

            return 'intrinsic'

        if class_name == 'Cast':
            source_desc = self._describe_operand(self._safe_getattr(node, 'x'))
            return self._combine_parts(source_desc, 'cast') or 'cast'

        # Check class-based prefixes first
        if class_name in self._class_prefixes:
            prefix = self._class_prefixes[class_name]
            if class_name in ('ArrayRead', 'ArrayWrite'):
                array_name = self._entity_name(self._safe_getattr(node, 'array'))
                if array_name:
                    return self._combine_parts(array_name, prefix) or prefix
            elif class_name == 'FIFOPop':
                fifo_name = self._entity_name(self._safe_getattr(node, 'fifo'))
                if fifo_name:
                    return fifo_name
            elif class_name == 'FIFOPush':
                fifo_name = self._entity_name(self._safe_getattr(node, 'fifo'))
                if fifo_name:
                    return self._combine_parts(fifo_name, prefix) or prefix
            return prefix

        if 'BinaryOp' in mro_names:
            lhs_desc = self._describe_operand(self._safe_getattr(node, 'lhs'))
            rhs_desc = self._describe_operand(self._safe_getattr(node, 'rhs'))
            op_token = self._binary_ops.get(self._safe_getattr(node, 'opcode'), 'bin')
            return self._combine_parts(lhs_desc, op_token, rhs_desc) or op_token

        if 'UnaryOp' in mro_names:
            operand_desc = self._describe_operand(self._safe_getattr(node, 'x'))
            op_token = self._unary_ops.get(self._safe_getattr(node, 'opcode'), 'unary')
            return self._combine_parts(op_token, operand_desc) or op_token

        if class_name == 'Slice':
            source_node = self._safe_getattr(node, 'x')
            source_desc = self._describe_operand(source_node)
            return self._combine_parts(source_desc, 'slice') or 'slice'

        if class_name == 'Concat':
            msb_desc = self._describe_operand(self._safe_getattr(node, 'msb'))
            lsb_desc = self._describe_operand(self._safe_getattr(node, 'lsb'))
            return self._combine_parts(msb_desc, 'cat', lsb_desc) or 'concat'

        if class_name in ('Select', 'Select1Hot'):
            cond_desc = self._describe_operand(self._safe_getattr(node, 'cond'))
            return self._combine_parts(cond_desc, 'mux') or 'mux'

        name_attr = self._safe_getattr(node, 'name')
        if isinstance(name_attr, str):
            return self._sanitize(name_attr)

        # Default fallback
        return 'val'


    def name_value(self, value: Any, hint: Optional[str] = None) -> str:
        """Generate a unique name for a value based on its type."""
        # Clean the hint to be a valid identifier
        if hint:
            return self._cache.get_unique_name(self._sanitize(hint))

        # Get type-based prefix if no hint
        prefix = self._sanitize(self.get_prefix_for_type(value))
        return self._cache.get_unique_name(prefix)
