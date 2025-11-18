"""
Type-Oriented Namer for the Assassyn Naming System.

Generates semantically meaningful names based on IR node types and operations.
"""

from __future__ import annotations
import re
from typing import Optional, Any

from .unique_name import UniqueNameCache
from ..utils import unwrap_operand


class TypeOrientedNamer:
    """Generates appropriate names for IR nodes based on their type."""

    def __init__(self):
        self._cache = UniqueNameCache()

        # Import classes locally to avoid circular imports
        # pylint: disable=import-outside-toplevel
        from ..ir.expr.arith import BinaryOp, UnaryOp
        from ..ir.expr.array import ArrayRead, ArrayWrite
        from ..ir.expr.call import FIFOPush, Bind, AsyncCall
        from ..ir.expr.expr import FIFOPop, Cast, Concat, Select, Select1Hot
        from ..ir.expr.intrinsic import PureIntrinsic
        from ..ir.array import Slice

        # Unified naming strategies dictionary
        self._naming_strategies = {
            BinaryOp: self._binary_op_strategy,
            UnaryOp: self._unary_op_strategy,
            PureIntrinsic: self._pure_intrinsic_strategy,
            ArrayRead: lambda n: self._combine_parts(self._entity_name(n.array), 'rd') or 'rd',
            ArrayWrite: lambda n: self._combine_parts(self._entity_name(n.array), 'wt') or 'wt',
            FIFOPop: lambda n: self._entity_name(n.fifo) or 'pop',
            FIFOPush: lambda n: self._combine_parts(self._entity_name(n.fifo), 'push') or 'push',
            Bind: lambda n: 'bind',
            AsyncCall: lambda n: 'call',
            Cast: lambda n: self._combine_parts(self._describe_operand(n.x), 'cast') or 'cast',
            Slice: lambda n: self._combine_parts(self._describe_operand(n.x), 'slice') or 'slice',
            Concat: lambda n: self._combine_parts(
                self._describe_operand(n.msb), 'cat', self._describe_operand(n.lsb)
            ) or 'concat',
            Select: lambda n: self._combine_parts(self._describe_operand(n.cond), 'mux') or 'mux',
            Select1Hot: lambda n: self._combine_parts(
                self._describe_operand(n.cond), 'mux'
            ) or 'mux',
        }

    @staticmethod
    def _sanitize(text: str) -> str:
        """Sanitize text into a valid identifier-like token."""
        return re.sub(r'[^0-9a-zA-Z_]+', '_', text) or 'val'

    @staticmethod
    def _symbol_to_name():
        """Convert operator symbols to descriptive names."""
        return {
            '+': 'add', '-': 'sub', '*': 'mul', '/': 'div', '%': 'mod',
            '&': 'and_', '|': 'or_', '^': 'xor_',
            '<': 'lt', '>': 'gt', '<=': 'le', '>=': 'ge', '==': 'eq', '!=': 'neq',
            '<<': 'shl', '>>': 'shr',
            '!': 'not_',
        }


    def _entity_name(self, entity: Any) -> Optional[str]:
        """Extract a meaningful name from an entity."""
        if entity is None:
            return None

        entity = unwrap_operand(entity)

        # Check for name attribute
        name_attr = getattr(entity, 'name', None)
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

    def _describe_operand(self, operand: Any) -> Optional[str]:
        """Provide a descriptive token for an operand."""
        name = self._entity_name(operand)
        if not name:
            return None
        return self._head_token_segment(name)

    def _combine_parts(self, *parts: Optional[str]) -> Optional[str]:
        """Combine multiple name parts into a sanitized identifier."""
        # Filter out None/empty parts - callers already provide sanitized/segmented inputs
        tokens = [part for part in parts if isinstance(part, str) and part]
        if not tokens:
            return None

        # Remove duplicate adjacent tokens for clarity
        deduped = [tokens[0]]
        for token in tokens[1:]:
            if deduped[-1] != token:
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

    def _binary_op_strategy(self, node: Any) -> str:
        """Strategy for binary operations using OPERATORS dictionary."""
        # pylint: disable=import-outside-toplevel
        from ..ir.expr.arith import BinaryOp

        symbol = BinaryOp.OPERATORS.get(node.opcode)
        if symbol:
            op_name = self._symbol_to_name().get(symbol, 'bin')
            lhs_desc = self._describe_operand(node.lhs)
            rhs_desc = self._describe_operand(node.rhs)
            res = self._combine_parts(lhs_desc, op_name, rhs_desc) or op_name
            return res
        return 'bin'

    def _unary_op_strategy(self, node: Any) -> str:
        """Strategy for unary operations using OPERATORS dictionary."""
        # pylint: disable=import-outside-toplevel
        from ..ir.expr.arith import UnaryOp

        symbol = UnaryOp.OPERATORS.get(node.opcode)
        if symbol:
            op_name = self._symbol_to_name().get(symbol, 'unary')
            operand_desc = self._describe_operand(node.x)
            return self._combine_parts(op_name, operand_desc) or op_name
        return 'unary'

    def _pure_intrinsic_strategy(self, node: Any) -> str:
        """Strategy for pure intrinsics using OPERATORS dictionary."""
        # pylint: disable=import-outside-toplevel
        from ..ir.expr.intrinsic import PureIntrinsic

        op_suffix = PureIntrinsic.OPERATORS.get(node.opcode)
        args = node.args or ()

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

    def get_prefix_for_type(self, node: Any) -> str:
        """Get the naming prefix for a given node type using strategy pattern."""
        # Check ModuleBase via MRO
        mro_names = {base.__name__ for base in node.__class__.__mro__}
        if 'ModuleBase' in mro_names:
            return self._module_prefix(node)

        # Try class-based strategy
        node_class = node.__class__
        if node_class in self._naming_strategies:
            return self._naming_strategies[node_class](node)

        # Fallback to name attribute or 'val'
        name_attr = getattr(node, 'name', None)
        if isinstance(name_attr, str):
            return self._sanitize(name_attr)

        return 'val'


    def name_value(self, value: Any, hint: Optional[str] = None) -> str:
        """Generate a unique name for a value based on its type."""
        # Clean the hint to be a valid identifier
        if hint:
            return self._cache.get_unique_name(self._sanitize(hint))

        # Get type-based prefix if no hint
        prefix = self._sanitize(self.get_prefix_for_type(value))
        return self._cache.get_unique_name(prefix)
