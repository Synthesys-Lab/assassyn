"""FIFO metadata analysis pre-pass for Verilog code generation."""

# pylint: disable=duplicate-code

from __future__ import annotations

from typing import Dict, List, Sequence, Set, Tuple, TYPE_CHECKING

from ...analysis.external_usage import expr_externally_used
from ...ir.const import Const
from ...ir.expr import AsyncCall, Expr, FIFOPop, FIFOPush, Log
from ...ir.expr.array import ArrayRead, ArrayWrite
from ...ir.expr.intrinsic import ExternalIntrinsic, Intrinsic, PureIntrinsic
from ...ir.visitor import Visitor
from ..simulator.external import collect_external_intrinsics
from ...utils import unwrap_operand
from .metadata import (
    ExternalRead,
    ExternalRegistry,
    InteractionKind,
    InteractionMatrix,
    ModuleMetadata,
)

if TYPE_CHECKING:
    from ...builder import SysBuilder
    from ...ir.module import Module
    from ...ir.value import Value


def collect_fifo_metadata(
    sys: "SysBuilder",
    modules: Sequence["Module"] | None = None,
) -> Tuple[Dict["Module", ModuleMetadata], InteractionMatrix]:
    """Traverse modules in *sys* and build FIFO metadata.

    Args:
        sys: System builder containing the modules to analyse.
        modules: Optional subset of modules to visit. When omitted the helper walks
            every module and downstream module in *sys*.

    Returns:
        A tuple ``(module_metadata, interactions)`` containing the populated
        metadata map and the shared interaction matrix.
    """

    if modules is None:
        modules_to_visit: List["Module"] = list(sys.modules) + list(sys.downstreams)
    else:
        modules_to_visit = list(dict.fromkeys(modules))

    if not modules_to_visit:
        return {}, InteractionMatrix()

    system_members: Set["Module"] = set(sys.modules) | set(sys.downstreams)
    missing = [module for module in modules_to_visit if module not in system_members]
    if missing:
        missing_names = ", ".join(module.name for module in missing)
        raise ValueError(f"Modules not present in the system: {missing_names}")

    matrix = InteractionMatrix()
    module_metadata: Dict["Module", ModuleMetadata] = {
        module: ModuleMetadata(module, matrix) for module in modules_to_visit
    }
    visitor = FIFOAnalysisVisitor(matrix, module_metadata)

    visitor.analyse_modules(modules_to_visit)

    matrix.freeze()
    for metadata in module_metadata.values():
        metadata.freeze()

    return module_metadata, matrix


class FIFOAnalysisVisitor(Visitor):
    """Visitor that collects FIFO interactions ahead of code generation."""

    def __init__(
        self,
        matrix: InteractionMatrix,
        module_metadata: Dict["Module", ModuleMetadata],
    ) -> None:
        super().__init__()
        self._matrix = matrix
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

        if isinstance(node, (FIFOPush, FIFOPop)):
            kind = (
                InteractionKind.FIFO_PUSH
                if isinstance(node, FIFOPush)
                else InteractionKind.FIFO_POP
            )
            self._matrix.record(module=module, resource=node.fifo, kind=kind, expr=node)
            if isinstance(node, FIFOPop) and expr_externally_used(node, True):
                metadata.record_value(node)
            return

        if isinstance(node, AsyncCall):
            metadata.record_call(node)
            callee = node.bind.callee
            self._matrix.async_ledger.record(module, callee, node)
            return

        if isinstance(node, ArrayWrite):
            self._matrix.record(
                module=module,
                resource=node.array,
                kind=InteractionKind.ARRAY_WRITE,
                expr=node,
            )
            return

        if isinstance(node, ArrayRead):
            self._matrix.record(
                module=module,
                resource=node.array,
                kind=InteractionKind.ARRAY_READ,
                expr=node,
            )
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

            metadata.record_value(node)

    def _handle_intrinsic(self, metadata: ModuleMetadata, node: Intrinsic) -> None:
        intrinsic = node.opcode

        if intrinsic == Intrinsic.FINISH:
            metadata.record_finish(node)
            return

        if intrinsic == Intrinsic.ASSERT:
            if node.args:
                self._record_value_exposure(metadata, node.args[0])
            return

        # Other intrinsics (WAIT_UNTIL, predicate stack ops, etc.) do not
        # require additional metadata.

    def _record_value_exposure(self, metadata: ModuleMetadata, value) -> None:
        expr = unwrap_operand(value)
        if isinstance(expr, Const):
            return
        if not isinstance(expr, Expr):
            return
        metadata.record_value(expr)

    def _record_log_exposures(self, metadata: ModuleMetadata, node: Log) -> None:
        self._record_value_exposure(metadata, node.meta_cond)
        for operand in node.values:
            self._record_value_exposure(metadata, operand)


def collect_external_metadata(sys: "SysBuilder") -> ExternalRegistry:
    """Analyse *sys* to gather external module metadata for Verilog codegen."""

    registry = ExternalRegistry()
    external_intrinsics = collect_external_intrinsics(sys)
    for intrinsic in external_intrinsics:
        owner = getattr(intrinsic, "parent", None)
        if owner is None:
            continue
        registry.record_instance(intrinsic, owner)

    modules_to_scan: List["Module"] = list(sys.modules) + list(sys.downstreams)
    for module in modules_to_scan:
        body = getattr(module, "body", None)
        if body is None:
            continue
        for expr in body:
            if (
                not isinstance(expr, PureIntrinsic)
                or expr.opcode != PureIntrinsic.EXTERNAL_OUTPUT_READ
            ):
                continue

            instance_operand = unwrap_operand(expr.args[0])
            if not isinstance(instance_operand, ExternalIntrinsic):
                continue

            producer = registry.owner_for(instance_operand)
            if producer is None or producer is module:
                continue

            port_operand = expr.args[1]
            port_name = (
                port_operand.value
                if hasattr(port_operand, "value")
                else port_operand
            )
            index_operand = expr.args[2] if len(expr.args) > 2 else None

            registry.record_cross_module_read(
                ExternalRead(
                    expr=expr,
                    producer=producer,
                    consumer=module,
                    instance=instance_operand,
                    port_name=port_name,
                    index_operand=index_operand,
                )
            )

    registry.freeze()
    return registry
