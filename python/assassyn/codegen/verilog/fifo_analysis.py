"""FIFO metadata analysis pre-pass for Verilog code generation."""

# pylint: disable=duplicate-code

from __future__ import annotations

from typing import Dict, List, Optional, Sequence, Set, Tuple, TYPE_CHECKING

from ...analysis.external_usage import expr_externally_used
from ...ir.const import Const
from ...ir.expr import AsyncCall, Expr, FIFOPop, FIFOPush, Log
from ...ir.expr.array import ArrayRead, ArrayWrite
from ...ir.expr.intrinsic import ExternalIntrinsic, Intrinsic, PureIntrinsic
from ...ir.visitor import Visitor
from .metadata import FIFORegistry, ModuleMetadata
from ...utils import unwrap_operand

if TYPE_CHECKING:
    from ...builder import SysBuilder
    from ...ir.module import Module
    from ...ir.value import Value


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

    for metadata in module_metadata.values():
        metadata.exposures.freeze()

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

    # pylint: disable=too-many-return-statements,too-many-branches
    def visit_expr(self, node: Expr) -> None:  # type: ignore[override]
        module = self.current_module
        if module is None:
            return

        metadata = self._module_metadata[module]

        if isinstance(node, Intrinsic):
            self._handle_intrinsic(metadata, node)
            return

        if isinstance(node, FIFOPush):
            predicate_value, tokens = self._predicate_snapshot(node)
            interaction = self._registry.record_push(module, node, predicate_value, tokens)
            metadata.record_fifo_interaction(node.fifo, interaction)
            return

        if isinstance(node, FIFOPop):
            predicate_value, tokens = self._predicate_snapshot(node)
            interaction = self._registry.record_pop(module, node, predicate_value, tokens)
            metadata.record_fifo_interaction(node.fifo, interaction)
            if expr_externally_used(node, True):
                metadata.exposures.record_value(node, predicate_value, tokens)
            return

        if isinstance(node, AsyncCall):
            metadata.calls.append(node)
            callee = node.bind.callee
            predicate, tokens = self._predicate_snapshot(node)
            metadata.exposures.record_async_trigger(callee, node, predicate, tokens)
            return

        if isinstance(node, ArrayWrite):
            predicate, tokens = self._predicate_snapshot(node)
            metadata.exposures.record_array_write(
                node.array,
                node.module,
                node,
                predicate,
                tokens,
            )
            return

        if isinstance(node, ArrayRead):
            metadata.exposures.record_array_read(node.array, node)
            return

        if isinstance(node, Log):
            self._record_log_exposures(metadata, node)
            return

        # General valued expression exposure tracking.
        if node.is_valued():
            if isinstance(node, ExternalIntrinsic):
                return

            if (
                isinstance(node, PureIntrinsic)
                and node.opcode == PureIntrinsic.EXTERNAL_OUTPUT_READ
            ):
                instance_operand = unwrap_operand(node.args[0])
                instance_owner = getattr(instance_operand, "parent", None)
                if instance_owner is module:
                    return

            if not expr_externally_used(node, True):
                return

            unwrapped = unwrap_operand(node)
            if isinstance(unwrapped, Const):
                return

            predicate, tokens = self._predicate_snapshot(node)
            metadata.exposures.record_value(node, predicate, tokens)

    def _handle_intrinsic(self, metadata: ModuleMetadata, node: Intrinsic) -> None:
        intrinsic = node.opcode

        if intrinsic == Intrinsic.FINISH:
            metadata.has_finish = True
            return

        if intrinsic == Intrinsic.ASSERT:
            if node.args:
                self._record_value_exposure(metadata, node.args[0])
            return

        # Other intrinsics (WAIT_UNTIL, predicate stack ops, etc.) do not
        # require additional metadata.

    def _predicate_snapshot(self, expr: Expr) -> tuple[Optional['Value'], tuple['Value', ...]]:
        predicate = getattr(expr, "meta_cond", None)
        tokens = getattr(expr, "predicate_tokens", ())
        if not tokens:
            return predicate, tuple()
        return predicate, tuple(tokens)

    def _record_value_exposure(self, metadata: ModuleMetadata, value) -> None:
        expr = unwrap_operand(value)
        if isinstance(expr, Const):
            return
        if not isinstance(expr, Expr):
            return
        predicate, tokens = self._predicate_snapshot(expr)
        metadata.exposures.record_value(expr, predicate, tokens)

    def _record_log_exposures(self, metadata: ModuleMetadata, node: Log) -> None:
        self._record_value_exposure(metadata, node.meta_cond)
        for operand in node.values:
            self._record_value_exposure(metadata, operand)
