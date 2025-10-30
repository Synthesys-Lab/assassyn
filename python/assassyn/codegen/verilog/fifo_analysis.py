"""FIFO metadata analysis pre-pass for Verilog code generation."""

# pylint: disable=duplicate-code

from __future__ import annotations

from collections import defaultdict
from typing import Dict, List, Sequence, Set, Tuple, TYPE_CHECKING

from ...ir.expr import Expr, FIFOPop, FIFOPush
from ...ir.expr.intrinsic import Intrinsic
from ...ir.visitor import Visitor
from ...utils import namify
from .metadata import FIFORegistry, ModuleMetadata
from .predicate import PredicateStack
from .rval import dump_rval as dump_rval_impl

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
    formatter = _PredicateFormatter()
    visitor = FIFOAnalysisVisitor(registry, module_metadata, formatter)

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
        formatter: "_PredicateFormatter",
    ) -> None:
        super().__init__()
        self._registry = registry
        self._module_metadata = module_metadata
        self._formatter = formatter
        self._predicate_stack = PredicateStack()

    def analyse_modules(self, modules: Sequence["Module"]) -> None:
        """Analyse the provided modules and populate FIFO metadata."""

        for module in modules:
            self.current_module = module
            self._formatter.enter_module(module)
            self._predicate_stack.reset()

            body = getattr(module, "body", None)
            if isinstance(body, list):
                for entry in body:
                    self.dispatch(entry)

            self._predicate_stack.reset()
            self._formatter.leave_module()
            self.current_module = None

    def dispatch(self, node) -> None:  # type: ignore[override]
        if isinstance(node, Expr):
            self.visit_expr(node)

    # pylint: disable=too-many-return-statements
    def visit_expr(self, node: Expr) -> None:  # type: ignore[override]
        if isinstance(node, Intrinsic):
            opcode = node.opcode
            if opcode == Intrinsic.PUSH_CONDITION:
                cond_str = self._formatter.dump(node.args[0])
                self._predicate_stack.push(f"({cond_str})", node)
            elif opcode == Intrinsic.POP_CONDITION:
                self._predicate_stack.pop()
            return

        module = self.current_module
        if module is None:
            return

        metadata = self._module_metadata[module]
        predicate = self._predicate_stack.predicate()

        if isinstance(node, FIFOPush):
            interaction = self._registry.record_push(module, node, predicate)
            metadata.record_fifo_interaction(node.fifo, interaction)
            return

        if isinstance(node, FIFOPop):
            interaction = self._registry.record_pop(module, node, predicate)
            metadata.record_fifo_interaction(node.fifo, interaction)
            return


class _PredicateFormatter:
    """Formats predicate expressions using dumper-compatible naming rules."""

    def __init__(self) -> None:
        self._ctx = _AnalysisDumpContext()
        self._module_name: str | None = None

    def enter_module(self, module: "Module") -> None:
        """Prepare predicate formatting for *module*."""

        self._ctx.enter_module(module)
        self._module_name = namify(module.name)

    def leave_module(self) -> None:
        """Reset formatter state after leaving a module."""

        self._ctx.leave_module()
        self._module_name = None

    def dump(self, expr) -> str:
        """Format *expr* using the active module context."""

        return dump_rval_impl(self._ctx, expr, False, self._module_name)


class _AnalysisDumpContext:
    """Mimics the subset of CIRCTDumper used by ``dump_rval``."""

    def __init__(self) -> None:
        self.expr_to_name: Dict[Expr, str] = {}
        self.name_counters = defaultdict(int)
        self.current_module = None
        self.is_top_generation = False

    def enter_module(self, module: "Module") -> None:
        """Start formatting values while visiting *module*."""

        self.current_module = module
        self._reset_names()

    def leave_module(self) -> None:
        """Clear module-specific formatting state."""

        self.current_module = None
        self._reset_names()

    def get_external_port_name(self, node: Expr) -> str:
        """Derive the mangled port name for an external value."""

        producer_module = node.parent
        producer_name = namify(producer_module.name)
        base_port_name = namify(node.as_operand())
        if base_port_name.startswith("_"):
            base_port_name = f"port{base_port_name}"
        return f"{producer_name}_{base_port_name}"

    def _reset_names(self) -> None:
        """Clear expression naming caches."""

        self.expr_to_name.clear()
        self.name_counters.clear()
