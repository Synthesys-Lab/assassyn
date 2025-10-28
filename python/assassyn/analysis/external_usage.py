"""Analysis of external module usage patterns."""

from ..ir.expr import Expr, FIFOPush
from ..ir.module.base import ModuleBase
from ..ir.expr import Operand

def get_module(operand: Operand) -> ModuleBase | None:
    """Get the module that contains the given operand."""
    if isinstance(operand.user, Expr):
        parent = getattr(operand.user, 'parent', None)
        if isinstance(parent, ModuleBase):
            return parent
    return None

def expr_externally_used(expr: Expr, exclude_push: bool) -> bool:
    """Check if an expression is used outside its module."""
    if exclude_push:
        if isinstance(expr, FIFOPush):
            return False

    this_module = expr.parent if isinstance(expr.parent, ModuleBase) else None
    if this_module is None:
        return False

    for user in expr.users:
        user_parent_module = get_module(user)
        if user_parent_module is None:
            continue
        if user_parent_module != this_module:
            return True

    return False
