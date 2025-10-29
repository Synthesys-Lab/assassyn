"""FIFO metadata analysis pre-pass for Verilog code generation."""

from __future__ import annotations

from typing import Sequence, TYPE_CHECKING

from ...ir.expr import Expr, FIFOPop, FIFOPush
from ...ir.expr.intrinsic import Intrinsic
from ...ir.visitor import Visitor
from .metadata import ModuleMetadata
from .predicate import PredicateStack

if TYPE_CHECKING:
    from ...builder import SysBuilder
    from ...ir.module import Module
    from .design import CIRCTDumper


class FIFOAnalysisVisitor(Visitor):
    """Visitor that collects FIFO interactions ahead of code generation."""

    def __init__(self, dumper: "CIRCTDumper") -> None:
        super().__init__()
        self._dumper = dumper
        self._predicate_stack = PredicateStack()

    def analyze(self, _sys: "SysBuilder", modules: Sequence["Module"]) -> None:
        """Analyse the provided modules and populate FIFO metadata."""
        dumper = self._dumper
        registry = dumper.fifo_registry

        for module in modules:
            metadata = self._ensure_metadata(module)
            metadata.reset_for_analysis(registry)

        # When analysing a subset, clear previous entries for those modules.
        for module in modules:
            registry.clear_for_module(module)

        for module in modules:
            self._analyze_module(module)

    def _ensure_metadata(self, module: "Module") -> ModuleMetadata:
        dumper = self._dumper
        metadata = dumper.module_metadata.get(module)
        if metadata is None:
            metadata = ModuleMetadata(module, dumper.fifo_registry)
            dumper.module_metadata[module] = metadata
        return metadata

    def _analyze_module(self, module: "Module") -> None:
        dumper = self._dumper
        metadata = dumper.module_metadata[module]
        body = getattr(module, "body", None)

        prev_module = dumper.current_module
        prev_ctx = dumper.module_ctx
        dumper.current_module = module
        dumper.module_ctx = module

        self.current_module = module
        self._predicate_stack.reset()

        if isinstance(body, list):
            for entry in body:
                self._dispatch(entry)

        metadata.mark_fifo_ready()
        self._predicate_stack.reset()

        self.current_module = None
        dumper.current_module = prev_module
        dumper.module_ctx = prev_ctx

    def _dispatch(self, node) -> None:
        if isinstance(node, Expr):
            self.visit_expr(node)

    # pylint: disable=too-many-return-statements
    def visit_expr(self, node: Expr) -> None:
        if isinstance(node, Intrinsic):
            opcode = node.opcode
            if opcode == Intrinsic.PUSH_CONDITION:
                cond_str = self._dumper.dump_rval(node.args[0], False)
                self._predicate_stack.push(f"({cond_str})", node)
            elif opcode == Intrinsic.POP_CONDITION:
                self._predicate_stack.pop()
            return

        if isinstance(node, FIFOPush):
            predicate = self._predicate_stack.predicate()
            interaction = self._dumper.fifo_registry.record_push(
                self.current_module, node, predicate
            )
            self._dumper.module_metadata[self.current_module].record_fifo_interaction(
                node.fifo, interaction
            )
            return

        if isinstance(node, FIFOPop):
            predicate = self._predicate_stack.predicate()
            interaction = self._dumper.fifo_registry.record_pop(
                self.current_module, node, predicate
            )
            self._dumper.module_metadata[self.current_module].record_fifo_interaction(
                node.fifo, interaction
            )
            return
