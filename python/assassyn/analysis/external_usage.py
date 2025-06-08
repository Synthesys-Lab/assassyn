
"""Analysis of external module usage patterns."""

import typing

from ..ir.expr import Expr, FIFOPush
from ..ir.module import Module
from ..ir.block import CondBlock
from ..ir.expr import Operand

def get_module(operand: Operand) -> Module:
    """Get the module that contains the given operand."""
    if isinstance(operand.user, Expr):
        return operand.user.parent.module
    if isinstance(operand, CondBlock):
        return operand.user.module
    assert False, f'Unexpected operand type: {type(operand)}'

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
