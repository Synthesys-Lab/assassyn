"""Simulator helpers for working with ExternalSV modules."""

from __future__ import annotations

from typing import Dict, Iterable, Set, Tuple

from ...analysis import expr_externally_used
from ...ir.expr import Expr
from ...ir.module import Downstream, Module
from ...ir.module.external import ExternalSV
from ...ir.visitor import Visitor
from ...utils import namify


def external_handle_field(module_name: str) -> str:
    """Return the simulator struct field name for an ExternalSV handle."""

    return f"{namify(module_name)}_ffi"


class _ModuleValueExposureCollector(Visitor):
    """Collect expressions that need simulator-side caching."""

    def __init__(self):
        super().__init__()
        self.exprs: Set[Expr] = set()

    def visit_expr(self, node: Expr) -> None:
        if expr_externally_used(node, True):
            self.exprs.add(node)


def collect_module_value_exposures(module: Module) -> Set[Expr]:
    """Collect expressions that require simulator-side caching for a module."""

    body = getattr(module, "body", None)
    if not body:
        return set()

    collector = _ModuleValueExposureCollector()
    collector.current_module = module
    collector.visit_module(module)
    return collector.exprs

def gather_expr_validities(sys) -> Tuple[Set[Expr], Dict[Module, Set[Expr]]]:
    """Aggregate expressions whose values must be cached on the simulator."""

    exprs: Set[Expr] = set()
    module_expr_map: Dict[Module, Set[Expr]] = {}

    def record(module: Module, expr: Expr) -> None:
        exprs.add(expr)
        module_expr_map.setdefault(module, set()).add(expr)

    modules: Iterable[Module] = list(sys.modules) + list(sys.downstreams)
    for module in modules:
        if isinstance(module, Downstream):
            for expr in module.externals:
                if isinstance(expr, Expr):
                    record(module, expr)

        for expr in collect_module_value_exposures(module):
            record(module, expr)

        externals = getattr(module, "externals", None)
        if externals:
            for expr in externals:
                if isinstance(expr, Expr):
                    record(module, expr)

    return exprs, module_expr_map


def has_module_body(module: Module) -> bool:
    """Return True when the module has an elaborated body."""
    body = getattr(module, "body", None)
    if body is None:
        return False
    if isinstance(body, list):
        return bool(body)
    nested_body = getattr(body, "body", None)
    if isinstance(nested_body, list):
        return bool(nested_body)
    return bool(body)


def is_stub_external(module: Module) -> bool:
    """Return True if the ExternalSV module has no synthesized body."""

    return isinstance(module, ExternalSV) and not has_module_body(module)


def collect_external_intrinsics(sys):
    """Collect all ExternalIntrinsic instances from the system IR."""
    # pylint: disable=import-outside-toplevel
    from ...ir.expr.intrinsic import ExternalIntrinsic

    intrinsics = []

    class ExternalIntrinsicCollector(Visitor):
        """Visitor that collects ExternalIntrinsic instances."""

        def visit_expr(self, node):
            """Visit an expression and collect ExternalIntrinsic instances."""
            if isinstance(node, ExternalIntrinsic):
                intrinsics.append(node)

    visitor = ExternalIntrinsicCollector()
    visitor.visit_system(sys)

    return intrinsics


def collect_external_classes(external_intrinsics):
    """Return a mapping of unique ExternalSV classes referenced by intrinsics."""
    classes: Dict[str, type] = {}
    for intr in external_intrinsics:
        external_class = getattr(intr, "external_class", None)
        if external_class is None:
            continue
        classes.setdefault(external_class.__name__, external_class)
    return classes


__all__ = [
    "collect_external_intrinsics",
    "collect_external_classes",
    "collect_module_value_exposures",
    "external_handle_field",
    "gather_expr_validities",
    "has_module_body",
    "is_stub_external",
]
