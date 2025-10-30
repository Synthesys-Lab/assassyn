"""FIFO metadata analysis pre-pass for Verilog code generation."""

# pylint: disable=duplicate-code

from __future__ import annotations

from typing import Dict, List, Sequence, Set, Tuple, TYPE_CHECKING

from ...ir.const import Const
from ...ir.expr import Expr, FIFOPop, FIFOPush
from ...ir.expr.intrinsic import Intrinsic
from ...ir.visitor import Visitor
from .metadata import FIFORegistry, ModuleMetadata

if TYPE_CHECKING:
    from ...builder import SysBuilder
    from ...ir.module import Module


def collect_fifo_metadata(
    sys: "SysBuilder",
    modules: Sequence["Module"] | None = None,
) -> Tuple[Dict["Module", ModuleMetadata], FIFORegistry]:
    """Traverse modules in *sys* and build FIFO metadata.

    Args:
        sys: System builder containing the modules to analyse.
        modules: Optional subset of modules to visit. When omitted the helper walks
            every module and downstream module in *sys*.

    Returns:
        A tuple ``(module_metadata, fifo_registry)`` containing the populated
        metadata map and the shared registry.
    """

    if modules is None:
        modules_to_visit: List["Module"] = list(sys.modules) + list(sys.downstreams)
    else:
        modules_to_visit = list(dict.fromkeys(modules))

    if not modules_to_visit:
        return {}, FIFORegistry()

    system_members: Set["Module"] = set(sys.modules) | set(sys.downstreams)
    missing = [module for module in modules_to_visit if module not in system_members]
    if missing:
        missing_names = ", ".join(module.name for module in missing)
        raise ValueError(f"Modules not present in the system: {missing_names}")

    registry = FIFORegistry()
    module_metadata: Dict["Module", ModuleMetadata] = {}
    visitor = FIFOAnalysisVisitor(registry, module_metadata)

    for module in modules_to_visit:
        module_metadata[module] = ModuleMetadata(module, registry)

    visitor.analyse_modules(modules_to_visit)
    return module_metadata, registry


class FIFOAnalysisVisitor(Visitor):
    """Visitor that collects FIFO interactions ahead of code generation."""

    def __init__(
        self,
        registry: FIFORegistry,
        module_metadata: Dict["Module", ModuleMetadata],
    ) -> None:
        super().__init__()
        self._registry = registry
        self._module_metadata = module_metadata
        self._true_predicate: Const | None = None

    def analyse_modules(self, modules: Sequence["Module"]) -> None:
        """Analyse the provided modules and populate FIFO metadata."""

        for module in modules:
            self.current_module = module

            body = getattr(module, "body", None)
            if isinstance(body, list):
                for entry in body:
                    self.dispatch(entry)

            self.current_module = None

    def dispatch(self, node) -> None:  # type: ignore[override]
        if isinstance(node, Expr):
            self.visit_expr(node)

    # pylint: disable=too-many-return-statements
    def visit_expr(self, node: Expr) -> None:  # type: ignore[override]
        if isinstance(node, Intrinsic):
            return

        module = self.current_module
        if module is None:
            return

        metadata = self._module_metadata[module]

        if isinstance(node, FIFOPush):
            predicate_value = node.meta_cond if hasattr(node, "meta_cond") else None
            if predicate_value is None:
                predicate_value = self._true_predicate or self._materialise_true()
            interaction = self._registry.record_push(module, node, predicate_value)
            metadata.record_fifo_interaction(node.fifo, interaction)
            return

        if isinstance(node, FIFOPop):
            predicate_value = node.meta_cond if hasattr(node, "meta_cond") else None
            if predicate_value is None:
                predicate_value = self._true_predicate or self._materialise_true()
            interaction = self._registry.record_pop(module, node, predicate_value)
            metadata.record_fifo_interaction(node.fifo, interaction)
            return

    def _materialise_true(self) -> Const:
        """Return a cached constant true predicate."""
        # Lazy import to avoid circular dependencies during module load.
        from ...ir.dtype import Bits  # pylint: disable=import-outside-toplevel

        self._true_predicate = Bits(1)(1)
        return self._true_predicate
