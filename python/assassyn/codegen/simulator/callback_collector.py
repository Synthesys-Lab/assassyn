"""Visitors for collecting callback-related intrinsics."""

from __future__ import annotations

from typing import Dict, TYPE_CHECKING

from ...ir.visitor import Visitor
from ...ir.expr.intrinsic import Intrinsic
from ...utils import namify
from .utils import fifo_name

if TYPE_CHECKING:
    from ...builder import SysBuilder
    from ...ir.module import Module


class CallbackIntrinsicCollector(Visitor):
    """Visitor that gathers metadata needed for simulator callbacks."""

    def __init__(self) -> None:
        super().__init__()
        self._modules_for_callback: Dict[str, str] = {}

    def collect(self, sys: "SysBuilder") -> Dict[str, str]:
        """Collect callback metadata for all modules in the system."""
        for module in sys.modules[:] + sys.downstreams[:]:
            self.visit_module(module)
        # Reset current module state to avoid leaking references
        self.current_module = None
        return self._modules_for_callback

    def visit_module(self, node: "Module") -> None:  # type: ignore[override]
        previous_module = self.current_module
        self.current_module = node
        super().visit_module(node)
        self.current_module = previous_module

    def visit_expr(self, node):  # type: ignore[override]
        if isinstance(node, Intrinsic):
            self._handle_intrinsic(node)

    def _handle_intrinsic(self, node: Intrinsic) -> None:
        if node.opcode == Intrinsic.USE_DRAM:
            dram_port = node.args[0]
            self._modules_for_callback["MemUser_rdata"] = fifo_name(dram_port)
        elif node.opcode == Intrinsic.MEM_WRITE and self.current_module is not None:
            payload = node.args[0]
            array_name = getattr(payload, "name", None)
            if array_name is not None:
                self._modules_for_callback["store"] = namify(array_name)
            self._modules_for_callback["memory"] = self.current_module.name


def collect_callback_intrinsics(sys: "SysBuilder") -> Dict[str, str]:
    """Helper function to collect callback metadata from a system."""
    collector = CallbackIntrinsicCollector()
    return collector.collect(sys)
