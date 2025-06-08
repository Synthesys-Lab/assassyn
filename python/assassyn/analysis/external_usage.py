
import typing

from ..ir.expr import Expr, FIFOPush
from ..ir.module import Module
from ..ir.block import CondBlock
from ..ir.visitor import Visitor
from ..ir.expr import Operand
from ..builder import SysBuilder

def get_module(operand: Operand) -> Module:
    if isinstance(operand.user, Expr):
        return operand.user.parent.module
    elif isinstance(operand, CondBlock):
        return operand.user.module

def expr_externally_used(expr: Expr, exclude_push: bool) -> typing.Set[Module]:
    """Check if an expression is used outside its module.
    Returns the module uses this expression.
    """

    # Push is NOT a combinational operation
    if exclude_push:
        if isinstance(expr, FIFOPush):
            return set()

    this_module = expr.parent.module

    res = set()

    # Check if any user is in a different module
    for user in expr.users:
        assert isinstance(user, Operand), f'{user} is a {type(user)}'
        user_parent_module = user.user.parent.module
        if user_parent_module != this_module:
            res.add(user_parent_module)

    return res

class BiExternalUsage:

    # Expressions used by this module externally
    module_uses_expr: typing.Dict[Module, typing.Set[Expr]]

    # Modules that externally use this expression
    expr_used_by_module: typing.Dict[Expr, typing.Set[Module]]

    def __init__(self):
        self.module_uses_expr = {}
        self.expr_used_by_module = {}

class ExternalUsageAnalyzer(Visitor):

    def __init__(self, sys: SysBuilder):
        self.res = BiExternalUsage()

    def visit_expr(self, node: Expr):
        self.res.expr_used_by_module[node] = expr_externally_used(node, False)
        for module in self.res.expr_used_by_module[node]:
            self.res.module_uses_expr[module].add(node)

def analyze_bidirectional_external_usage(sys: SysBuilder) -> BiExternalUsage:
    analyzer = ExternalUsageAnalyzer(sys)
    analyzer.visit_system(sys)
    return analyzer.res
