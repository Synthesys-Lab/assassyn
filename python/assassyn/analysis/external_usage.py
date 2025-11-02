"""Analysis of external module usage patterns."""

from __future__ import annotations

from collections import defaultdict
from typing import Dict, Iterable, MutableMapping, Optional, Set, Tuple

from ..ir.expr import Expr, FIFOPush, Operand
from ..ir.module.base import ModuleBase
from ..utils import unwrap_operand


def get_module(operand: Operand) -> ModuleBase | None:
    """Get the module that contains the given operand."""
    if isinstance(operand.user, Expr):
        parent = getattr(operand.user, "parent", None)
        if isinstance(parent, ModuleBase):
            return parent
    return None


class ExternalUsageIndex:
    """Precomputed index describing which modules consume a given expression."""

    __slots__ = ("_external_consumers", "_user_cache")

    def __init__(self) -> None:
        self._external_consumers: Dict[Expr, Set[ModuleBase]] = defaultdict(set)
        self._user_cache: Dict[Tuple[Expr, ModuleBase, bool], bool] = {}

    # ------------------------------------------------------------------ #
    # Population helpers
    # ------------------------------------------------------------------ #
    def record_module_externals(self, module: ModuleBase) -> None:
        """Register all external expressions consumed by *module*."""

        externals: Optional[MutableMapping[object, list[Operand]]] = getattr(
            module, "externals", None
        )
        if not externals:
            return

        for value in externals:
            self._record_external_value(module, value)

    def _record_external_value(self, module: ModuleBase, value: object) -> None:
        """Walk *value* and record any expressions owned by other modules."""

        for expr in self._iter_exprs(value):
            owner = getattr(expr, "parent", None)
            if not isinstance(owner, ModuleBase):
                continue
            if owner is module:
                continue
            self._external_consumers[expr].add(module)

    def _iter_exprs(self, root: object) -> Iterable[Expr]:
        """Yield expressions reachable from *root*."""

        stack = [root]
        seen: Set[int] = set()
        while stack:
            current = stack.pop()
            node = unwrap_operand(current)
            if not isinstance(node, Expr):
                continue
            identity = id(node)
            if identity in seen:
                continue
            seen.add(identity)
            yield node
            for operand in getattr(node, "operands", ()):
                stack.append(operand)

    # ------------------------------------------------------------------ #
    # Query helpers
    # ------------------------------------------------------------------ #
    def is_externally_used(
        self,
        expr: Expr,
        owning_module: Optional[ModuleBase],
        *,
        exclude_push: bool = True,
    ) -> bool:
        """Return ``True`` when *expr* is consumed by a different module."""

        if exclude_push and isinstance(expr, FIFOPush):
            return False

        module = owning_module or (
            expr.parent if isinstance(expr.parent, ModuleBase) else None
        )
        if module is None:
            return False

        consumers = self._external_consumers.get(expr, ())
        for consumer in consumers:
            if consumer is not module:
                return True

        cache_key = (expr, module, exclude_push)
        cached = self._user_cache.get(cache_key)
        if cached is not None:
            return cached

        result = False
        for user in expr.users:
            user_parent_module = get_module(user)
            if user_parent_module is None or user_parent_module is module:
                continue
            result = True
            break

        self._user_cache[cache_key] = result
        return result


def build_external_usage_index(modules: Iterable[ModuleBase]) -> ExternalUsageIndex:
    """Build an :class:`ExternalUsageIndex` covering *modules*."""

    index = ExternalUsageIndex()
    for module in modules:
        if isinstance(module, ModuleBase):
            index.record_module_externals(module)
    return index


def expr_externally_used(
    expr: Expr,
    exclude_push: bool,
    index: ExternalUsageIndex | None = None,
) -> bool:
    """Check if an expression is used outside its module."""

    if index is not None:
        owning_module = expr.parent if isinstance(expr.parent, ModuleBase) else None
        return index.is_externally_used(
            expr, owning_module, exclude_push=exclude_push
        )

    if exclude_push and isinstance(expr, FIFOPush):
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
