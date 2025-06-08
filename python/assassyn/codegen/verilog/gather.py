"""Gather classes for the Verilog backend."""

from collections import defaultdict

from ...builder import SysBuilder
from ...ir.module import Module
from ...ir.expr import Expr

from .utils import select_1h

class Gather:
    """Gather multiple conditional values into a single value."""

    def __init__(self, cond: str, value: str, bits: int):
        """Initialize a Gather instance."""
        self.bits = bits
        self.condition = [cond]
        self.value = [value]

    def and_(self, cond: str, join: str) -> str:
        """Combine condition with the gathered conditions."""
        if self.is_conditional():
            gather_cond = " || ".join([f"({x})" for x in self.condition])
            return f"({cond}) && ({gather_cond})"
        return cond

    def select_1h(self) -> str:
        """Select a value based on one-hot encoding."""
        if self.is_conditional():
            return select_1h(
                zip(self.condition, self.value),
                self.bits
            )
        return self.value[0]

    def is_conditional(self) -> bool:
        """Check if this gather is conditional."""
        assert self.condition, "Condition list cannot be empty"
        return bool(self.condition[0])

    def push(self, cond: str, value: str, bits: int):
        """Push a new conditional value into the gather."""
        assert self.is_conditional(), "Cannot push to a non-conditional gather"
        assert self.bits == bits, f"Bit width mismatch: {self.bits} != {bits}"
        self.condition.append(cond)
        self.value.append(value)

class ExternalUsage:
    """Track expressions used by external modules."""

    def __init__(self):
        """Initialize an ExternalUsage instance."""
        self.module_use_external_expr = defaultdict(set)
        self.expr_externally_used = defaultdict(set)

    def is_externally_used(self, expr: Expr) -> bool:
        """Check if an expression is externally used."""
        module = expr.parent.module
        return module in self.expr_externally_used and expr in self.expr_externally_used[module]

    def out_bounds(self, module: Module):
        """Get expressions used externally from this module."""
        if module in self.expr_externally_used:
            return self.expr_externally_used[module]
        return set()

    def in_bounds(self, module: Module):
        """Get external expressions used by this module."""
        if module in self.module_use_external_expr:
            return self.module_use_external_expr[module]
        return set()

def gather_exprs_externally_used(sys: SysBuilder) -> ExternalUsage:
    """Gather all expressions used by external modules."""
    from ...analysis import analyze_bidirectional_external_usage

    temp = analyze_bidirectional_external_usage(sys)

    result = ExternalUsage()
    result.module_use_external_expr = temp.module_uses_expr

    in_bounds = {}
    for expr, modules in temp.expr_used_by_module.items():
        if expr.parent.module not in in_bounds:
            in_bounds[expr.parent.module] = set()
        in_bounds[expr.parent.module] = in_bounds[expr.parent.module].union(modules)

    result.expr_externally_used = in_bounds

    return result
