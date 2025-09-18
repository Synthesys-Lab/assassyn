"""
Optimized AST naming generator with clearer, more concise naming rules.
Fixed to ensure global name uniqueness across the entire system.
"""
import ast
import logging
import typing
import hashlib
from dataclasses import dataclass, field
from collections import OrderedDict

log = logging.getLogger(__name__)

@dataclass
class NamingContext:
    """Context information for naming decisions"""
    ast_node: ast.AST
    target_names: typing.List[str]
    lineno: int
    generated_names: typing.Set[str] = field(default_factory=set)

class NamingStrategy:
    """Optimized naming strategy with semantic awareness"""

    # Common patterns for hardware/CPU operations
    SEMANTIC_PATTERNS = {
        # Register bypass patterns
        ('exec_bypass_reg', 'mem_bypass_reg'): 'bypass_check',
        ('rs1', 'rs2', 'rd'): 'reg',
        ('on_write', 'RShift'): 'write_mask',

        # ALU patterns
        ('ALU_', 'result'): 'alu_out',
        ('signals', 'alu'): 'alu_op',
        ('signals', 'cond'): 'branch_cond',

        # Memory patterns
        ('memory', 'read'): 'mem_rd',
        ('memory', 'write'): 'mem_wr',

        # PC patterns
        ('fetch_addr', 'Add', '4'): 'pc_next',
        ('is_offset_br', 'is_pc_calc'): 'branch_type',
    }

    # Shortened operation names
    OP_ABBREVIATIONS = {
        ast.Add: 'add', ast.Sub: 'sub', ast.Mult: 'mul',
        ast.Div: 'div', ast.Mod: 'mod', ast.LShift: 'shl',
        ast.RShift: 'shr', ast.BitAnd: 'and', ast.BitOr: 'or',
        ast.BitXor: 'xor', ast.Invert: 'not', ast.Not: 'not',
        ast.Eq: 'eq', ast.NotEq: 'ne', ast.Lt: 'lt',
        ast.LtE: 'le', ast.Gt: 'gt', ast.GtE: 'ge'
    }

    # Class-level global names tracker - shared across all instances
    _global_seen_names = set()

    def __init__(self):
        self.collected_names = []
        self.name_cache = OrderedDict()
        self.context_stack = []  # Track nested context
        # Reference the class-level global names
        self.global_seen_names = NamingStrategy._global_seen_names

    def get_unique_name(self, base_name: str, max_length: int = 30) -> str:
        """Generate unique name with length limit using global tracking"""
        # Truncate if too long
        if len(base_name) > max_length:
            base_name = self._abbreviate_name(base_name)

        # Check against global names
        if base_name not in self.global_seen_names:
            self.global_seen_names.add(base_name)
            return base_name

        # Add numeric suffix for uniqueness
        for i in range(2, 100):
            candidate = f"{base_name}_{i}"  # Use underscore for clarity
            if candidate not in self.global_seen_names:
                self.global_seen_names.add(candidate)
                return candidate

        hash_suffix = hashlib.md5(f"{base_name}{len(self.global_seen_names)}" \
                                  .encode()).hexdigest()[:6]
        candidate = f"{base_name}_{hash_suffix}"
        self.global_seen_names.add(candidate)
        return candidate

    def _abbreviate_name(self, name: str) -> str:
        """Abbreviate long names intelligently"""
        # Common abbreviations
        abbreviations = {
            'register': 'reg', 'memory': 'mem', 'execute': 'exec',
            'bypass': 'byp', 'condition': 'cond', 'address': 'addr',
            'instruction': 'inst', 'immediate': 'imm', 'result': 'res',
            'valid': 'vld', 'select': 'sel', 'bitcast': 'cast'
        }

        for full, abbr in abbreviations.items():
            name = name.replace(full, abbr)

        # Remove redundant underscores
        name = '_'.join(filter(None, name.split('_')))

        # If still too long, take first and last parts
        if len(name) > 25:
            parts = name.split('_')
            if len(parts) > 2:
                name = f"{parts[0]}_{parts[-1]}"

        return name

    def _detect_semantic_pattern(self, node: ast.AST) -> typing.Optional[str]:
        """Detect semantic patterns for better naming"""
        node_str = ast.dump(node)

        # Check for known patterns
        for pattern_keys, pattern_name in self.SEMANTIC_PATTERNS.items():
            if all(key in node_str for key in pattern_keys):
                return pattern_name

        return None

    def _extract_name_from_node(self, node: ast.AST, depth: int = 0) -> str:
        """Extract name with depth limit to prevent overly long names"""
        if depth > 2:  # Limit recursion depth
            return "val"

        # Check cache first
        node_repr = ast.dump(node)
        if node_repr in self.name_cache:
            return self.name_cache[node_repr]

        # Check for semantic patterns
        semantic_name = self._detect_semantic_pattern(node)
        if semantic_name:
            self.name_cache[node_repr] = semantic_name
            return semantic_name

        name = self._extract_name_impl(node, depth)
        self.name_cache[node_repr] = name
        return name

    #pylint: disable = too-many-return-statements,too-many-branches
    def _extract_name_impl(self, node: ast.AST, depth: int) -> str:
        """Implementation of name extraction"""
        if isinstance(node, ast.Name):
            return node.id

        if isinstance(node, ast.Constant):
            value = node.value
            if isinstance(value, int):
                if value in [0, 1, 4, 8, 16, 32]:
                    return str(value)
                return "val"
            if isinstance(value, str) and len(value) < 20:
                return value.replace(' ', '_')[:15]
            return "const"

        if isinstance(node, ast.Attribute):
            if depth > 0:
                return node.attr
            base = self._extract_name_from_node(node.value, depth + 1)
            if base in ['self', 'this']:
                return node.attr
            return f"{base}_{node.attr}" if len(base) < 10 else node.attr

        if isinstance(node, ast.Subscript):
            base = self._extract_name_from_node(node.value, depth + 1)
            if isinstance(node.slice, ast.Constant):
                # Don't use brackets in names as they can cause issues
                return f"{base}_{node.slice.value}"[:20]
            if isinstance(node.slice, ast.Slice):
                return f"{base}_slice"
            return f"{base}_idx"

        if isinstance(node, ast.BinOp):
            if depth > 1:
                return self.OP_ABBREVIATIONS.get(type(node.op), "op")
            left = self._extract_name_from_node(node.left, depth + 1)
            op = self.OP_ABBREVIATIONS.get(type(node.op), "op")
            # For simple ops, just use the operation name
            if len(left) > 10:
                return op
            return f"{left}_{op}"

        if isinstance(node, ast.UnaryOp):
            op = self.OP_ABBREVIATIONS.get(type(node.op), "unary")
            if depth > 0:
                return op
            operand = self._extract_name_from_node(node.operand, depth + 1)
            return f"{op}_{operand}"[:20]

        if isinstance(node, ast.Compare):
            left = self._extract_name_from_node(node.left, depth + 1)
            if node.ops:
                op = self.OP_ABBREVIATIONS.get(type(node.ops[0]), "cmp")
                return f"{left}_{op}"[:20]
            return f"cmp_{left}"[:15]

        if isinstance(node, ast.Call):
            if isinstance(node.func, ast.Attribute):
                method = node.func.attr
                if method in ['select', 'select1hot']:
                    return "sel"
                if method == 'bitcast':
                    return "cast"
                if method == 'log':
                    return "log"
                return method[:10]
            if isinstance(node.func, ast.Name):
                return node.func.id[:10]
            return "call"

        return "val"

    def generate_names(self, context: NamingContext) -> typing.List[str]:
        """Generate optimized names for assignment patterns"""
        self.collected_names = []

        try:
            node = context.ast_node
            if isinstance(node, ast.Assign):
                self._process_assignment(node)
            elif isinstance(node, ast.Expr):
                self._process_expression(node.value)
        except (AttributeError, TypeError, KeyError, IndexError) as e:
            log.debug("Error generating names for node type %s: %s",
                     type(context.ast_node).__name__, e)
            self.collected_names.append(self.get_unique_name("expr"))
        return self.collected_names

    #pylint: disable=too-many-branches
    def _process_assignment(self, assign: ast.Assign):
        """Process assignment with optimized naming"""
        target = assign.targets[0]
        value = assign.value

        # Get target name
        if isinstance(target, ast.Name):
            target_name = target.id
        elif isinstance(target, ast.Subscript):
            base = self._extract_name_from_node(target.value, 0)
            if isinstance(target.slice, ast.Constant):
                target_name = f"{base}{target.slice.value}"
            else:
                target_name = f"{base}_elem"
        elif isinstance(target, ast.Attribute):
            target_name = target.attr
        elif isinstance(target, ast.Tuple):
            # Handle tuple unpacking
            for _, elt in enumerate(target.elts):
                if isinstance(elt, ast.Name):
                    self.collected_names.append(self.get_unique_name(elt.id))
            return
        else:
            target_name = "var"

        # Generate name based on value
        if isinstance(value, ast.Call) and isinstance(value.func, ast.Attribute):
            method = value.func.attr
            if method in ['select', 'select1hot']:
                self.collected_names.append(self.get_unique_name(f"{target_name}_sel"))
            elif method == 'bitcast':
                self.collected_names.append(self.get_unique_name(f"{target_name}_cast"))
            else:
                self.collected_names.append(self.get_unique_name(target_name))
        else:
            # Use target name directly for simple assignments
            self.collected_names.append(self.get_unique_name(target_name))

    def _process_expression(self, node: ast.AST):
        """Process standalone expression"""
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
            if node.func.id == 'log':
                self.collected_names.append(self.get_unique_name("log_stmt"))
                return

        base = self._extract_name_from_node(node, 0)
        self.collected_names.append(self.get_unique_name(base))

    def reset(self):
        """Reset strategy state (but NOT global names)"""
        self.collected_names = []
        # Don't clear global_seen_names!
        self.name_cache.clear()
        self.context_stack = []

    @classmethod
    def clear_global_names(cls):
        """Clear global names - call this only when starting a new system"""
        cls._global_seen_names.clear()


class NamingManager:
    """Optimized naming manager with global name tracking"""

    def __init__(self):
        self.strategy = NamingStrategy()
        self.line_name_cache = {}
        self.generated_names_global = set()  # Track all generated names

    def generate_source_names(self, lineno: int, target_ast_node: ast.AST) -> typing.List[str]:
        """Generate optimized source names with caching"""
        # Use simplified cache key
        cache_key = (lineno, id(target_ast_node))
        if cache_key in self.line_name_cache:
            return self.line_name_cache[cache_key]

        context = NamingContext(
            ast_node=target_ast_node,
            target_names=self._extract_target_names(target_ast_node),
            lineno=lineno
        )

        names = self.strategy.generate_names(context)

        # Filter out overly generic names
        filtered_names = []
        for name in names:
            if name not in ['val', 'expr', 'const'] or len(names) == 1:
                filtered_names.append(name)
                self.generated_names_global.add(name)

        self.line_name_cache[cache_key] = filtered_names
        return filtered_names

    def _extract_target_names(self, ast_node: ast.AST) -> list:
        """Extract target variable names"""
        if isinstance(ast_node, ast.Expr):
            return []

        target_names = []
        if hasattr(ast_node, 'targets') and ast_node.targets:
            target = ast_node.targets[0]
            if isinstance(target, ast.Name):
                target_names = [target.id]
            elif isinstance(target, ast.Tuple):
                target_names = [elt.id for elt in target.elts if isinstance(elt, ast.Name)]

        return target_names

    def reset(self):
        """Reset manager state"""
        self.strategy.reset()
        self.line_name_cache.clear()
        # Don't clear generated_names_global unless explicitly needed

    def clear_all(self):
        """Completely reset including global names - use when starting new system"""
        self.reset()
        self.generated_names_global.clear()
        NamingStrategy.clear_global_names()
