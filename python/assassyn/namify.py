"""
generating descriptive names from Abstract Syntax Trees.
"""
import ast
import logging
import typing
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
    """recursive strategy with deduplication"""

    def __init__(self):
        self.collected_names = []
        self.temp_counter = 0
        self.name_cache = OrderedDict()
        self.seen_names = set()

    def _get_op_symbol(self, op_node: ast.operator) -> str:
        """Convert an AST operator to a descriptive string."""
        op_map = {
            ast.Add: "add", ast.Sub: "sub", ast.Mult: "mul",
            ast.Div: "div", ast.Mod: "mod", ast.LShift: "shl",
            ast.RShift: "shr", ast.BitAnd: "and", ast.BitOr: "or",
            ast.BitXor: "xor",
        }
        return op_map.get(type(op_node), "op")

    def _get_unique_name(self, base_name: str) -> str:
        """Ensure name is unique by adding suffix if needed"""
        if base_name not in self.seen_names:
            self.seen_names.add(base_name)
            return base_name

        counter = 2
        while f"{base_name}_{counter}" in self.seen_names:
            counter += 1
        unique_name = f"{base_name}_{counter}"
        self.seen_names.add(unique_name)
        return unique_name

    def _extract_simple_name(self, node: ast.AST) -> str:
        """Extract name from simple node types"""
        if isinstance(node, ast.Name):
            return node.id
        if isinstance(node, ast.Constant):
            return str(node.value)
        return ""

    #pylint: disable = too-many-return-statements
    def _extract_complex_name(self, node: ast.AST) -> str:
        """Extract name from complex node types"""
        if isinstance(node, ast.Attribute):
            base = self._extract_base_name_from_complex(node.value)
            return f"{base}_{node.attr}" if base else node.attr

        if isinstance(node, ast.Call):
            if isinstance(node.func, ast.Attribute):
                base = self._extract_base_name_from_complex(node.func.value)
                method = node.func.attr
                return f"{base}_{method}" if base else method
            if isinstance(node.func, ast.Name):
                return node.func.id
            return "call"

        if isinstance(node, ast.Subscript):
            base = self._extract_base_name_from_complex(node.value)
            index = self._extract_index_name(node.slice)
            return f"{base}_{index}" if base else f"item_{index}"

        if isinstance(node, ast.BinOp):
            left = self._extract_base_name_from_complex(node.left)
            op = self._get_op_symbol(node.op)
            right = self._extract_base_name_from_complex(node.right)
            return f"{left}_{op}_{right}"

        if isinstance(node, ast.UnaryOp):
            operand = self._extract_base_name_from_complex(node.operand)
            return f"not_{operand}" if isinstance(node.op, ast.Invert) else f"unary_{operand}"

        if isinstance(node, ast.Compare):
            left = self._extract_base_name_from_complex(node.left)
            return f"cmp_{left}"

        return "expr"

    def _extract_index_name(self, slice_node: ast.AST) -> str:
        """Extract index name from slice node"""
        if isinstance(slice_node, ast.Name):
            return slice_node.id
        if isinstance(slice_node, ast.Constant):
            return str(slice_node.value)
        if isinstance(slice_node, ast.Slice):
            lower = slice_node.lower.value if slice_node.lower else 0
            upper = slice_node.upper.value if slice_node.upper else "end"
            return f"{lower}_to_{upper}"
        return "idx"

    def _extract_base_name_from_complex(self, node: ast.AST) -> str:
        """Extract a base name from complex nested structures"""
        node_repr = ast.dump(node)
        if node_repr in self.name_cache:
            return self.name_cache[node_repr]

        try:
            # Try simple extraction first
            simple_name = self._extract_simple_name(node)
            if simple_name:
                self.name_cache[node_repr] = simple_name
                return simple_name

            # Then try complex extraction
            complex_name = self._extract_complex_name(node)
            self.name_cache[node_repr] = complex_name
            return complex_name

        except (AttributeError, IndexError, TypeError) as e:
            log.debug("Could not extract base name from %s: %s", type(node).__name__, e)
            return "expr"

    def generate_names(self, context: NamingContext) -> typing.List[str]:
        """Generate names for any assignment pattern with deduplication"""
        self.collected_names = []
        self.temp_counter = 0

        try:
            node = context.ast_node
            if isinstance(node, ast.Assign):
                self._process_assignment(node)
            elif isinstance(node, ast.Expr):
                self._process_expression(node.value)
        except (AttributeError, TypeError, ValueError) as e:
            log.warning("Error generating names for node: %s", e)
            fallback_name = self._get_unique_name("expr")
            self.collected_names.append(fallback_name)

        return self.collected_names

    def _process_assignment(self, assign: ast.Assign):
        """Process assignment statements"""
        target = assign.targets[0]
        value = assign.value
        target_name = self._extract_target_name(target)
        self._process_value(value, target_name, target)

    #pylint: disable = too-many-return-statements
    def _extract_target_name(self, target: ast.AST) -> typing.Optional[str]:
        """Extract name from assignment target"""
        if isinstance(target, ast.Name):
            return target.id
        if isinstance(target, ast.Attribute):
            base = self._extract_base_name_from_complex(target.value)
            return f"{base}_{target.attr}"
        if isinstance(target, ast.Subscript):
            base = self._extract_base_name_from_complex(target.value)
            if isinstance(target.slice, ast.Constant):
                return f"array_{base}_{target.slice.value}"
            if isinstance(target.slice, ast.Name):
                return f"array_{base}_{target.slice.id}"
            return f"array_{base}"
        if isinstance(target, ast.Tuple):
            return None  # Handled specially
        return "target"

    def _process_expression(self, node: ast.AST):
        """Process standalone expression"""
        base = self._extract_base_name_from_complex(node)
        unique_name = self._get_unique_name(base)
        self.collected_names.append(unique_name)

    def _process_value(self, node: ast.AST, target_name: str = None, target: ast.AST = None):
        """Process value expression with enhanced handling"""
        if isinstance(node, ast.BinOp):
            self._process_binop_enhanced(node, target_name)
        elif isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute):
            self._process_method_call(node, target_name, target)
        else:
            self._process_default(node, target_name)

    def _process_method_call(self, node: ast.Call, target_name: str, target: ast.AST):
        """Process method calls"""
        method = node.func.attr

        if method in {'select', 'select1hot'}:  # Using set as suggested
            self._process_select_enhanced(node, target_name)
        elif method == 'pop_all_ports':
            if isinstance(target, ast.Tuple):
                self._process_pop_all_ports(target)
            else:
                name = self._get_unique_name(target_name or "pop_result")
                self.collected_names.append(name)
        else:
            base = self._extract_base_name_from_complex(node.func.value)
            name = self._get_unique_name(
                f"{base}_{method}_result" if base else target_name or "result"
            )
            self.collected_names.append(name)

    def _process_default(self, node: ast.AST, target_name: str):
        """Process default case"""
        if target_name:
            unique_name = self._get_unique_name(target_name)
        else:
            base = self._extract_base_name_from_complex(node)
            unique_name = self._get_unique_name(base)
        self.collected_names.append(unique_name)

    def _process_binop_enhanced(self, node: ast.BinOp, base_name: str):
        """Enhanced binary operation processing"""
        left_name = self._extract_base_name_from_complex(node.left)
        right_name = self._extract_base_name_from_complex(node.right)
        op_str = self._get_op_symbol(node.op)

        if isinstance(node.left, (ast.BinOp, ast.Call)):
            self.collected_names.append(self._get_unique_name(f"{left_name}_temp"))
        if isinstance(node.right, (ast.BinOp, ast.Call)):
            self.collected_names.append(self._get_unique_name(f"{right_name}_temp"))

        result_name = base_name or f"{left_name}_{op_str}_{right_name}"
        self.collected_names.append(self._get_unique_name(result_name))

    def _process_select_enhanced(self, node: ast.Call, target_name: str):
        """Enhanced select processing"""
        cond_name = self._extract_base_name_from_complex(node.func.value)
        self.collected_names.append(self._get_unique_name(f"{cond_name}_cond"))

        for i, arg in enumerate(node.args):
            arg_name = self._extract_base_name_from_complex(arg)
            self.collected_names.append(self._get_unique_name(f"{arg_name}_sel{i}"))

        result = target_name or f"{cond_name}_select_result"
        self.collected_names.append(self._get_unique_name(result))

    def _process_pop_all_ports(self, target: ast.Tuple):
        """Process pop_all_ports with tuple unpacking"""
        for elt in target.elts:
            if isinstance(elt, ast.Name):
                self.collected_names.append(self._get_unique_name(f"{elt.id}_valid"))
                self.collected_names.append(self._get_unique_name(elt.id))

    def reset(self):
        """Reset the strategy state"""
        self.collected_names = []
        self.temp_counter = 0
        self.name_cache.clear()
        self.seen_names.clear()

class NamingManager:
    """Enhanced naming manager with better deduplication"""

    def __init__(self):
        self.strategy = NamingStrategy()
        self.line_contexts = {}
        self.line_name_cache = {}

    def generate_source_names(self, lineno: int, target_ast_node: ast.AST) -> typing.List[str]:
        """Generate source names with caching to avoid duplicates"""
        cache_key = (lineno, ast.dump(target_ast_node))
        if cache_key in self.line_name_cache:
            return self.line_name_cache[cache_key]

        context = NamingContext(
            ast_node=target_ast_node,
            target_names=self._extract_target_names(target_ast_node),
            lineno=lineno
        )

        names = self.strategy.generate_names(context)
        self.line_name_cache[cache_key] = names
        return names

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
        """Reset the manager state"""
        self.strategy.reset()
        self.line_contexts.clear()
        self.line_name_cache.clear()
