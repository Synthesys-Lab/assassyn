"""Utilities for collecting callback-related intrinsics."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional, TYPE_CHECKING

from ...ir.visitor import Visitor
from ...ir.expr.intrinsic import Intrinsic
from ...utils import namify
from .utils import fifo_name

if TYPE_CHECKING:
    from ...builder import SysBuilder
    from ...ir.module import Module


@dataclass
class CallbackMetadata:
    """Metadata derived from DRAM-related intrinsic usage."""

    memory: Optional[str] = None
    store: Optional[str] = None
    mem_user_rdata: Optional[str] = None


_METADATA_STORE = {"value": CallbackMetadata()}


def set_current_callback_metadata(metadata: CallbackMetadata) -> None:
    """Store metadata for later lookup during code generation."""

    _METADATA_STORE["value"] = metadata


def get_current_callback_metadata() -> CallbackMetadata:
    """Return the most recently collected metadata."""

    return _METADATA_STORE["value"]


class CallbackIntrinsicCollector(Visitor):
    """Visitor that gathers metadata needed for simulator callbacks."""

    def __init__(self, metadata: CallbackMetadata) -> None:
        super().__init__()
        self._metadata = metadata

    def collect(self, sys: "SysBuilder") -> CallbackMetadata:
        """Collect callback metadata for all modules in the system."""
        for module in sys.modules[:] + sys.downstreams[:]:
            self.visit_module(module)
        self.current_module = None
        return self._metadata

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
            self._metadata.mem_user_rdata = fifo_name(dram_port)
        elif node.opcode == Intrinsic.MEM_WRITE and self.current_module is not None:
            payload = node.args[0]
            array_name = getattr(payload, "name", None)
            if array_name is not None:
                self._metadata.store = namify(array_name)
            self._metadata.memory = self.current_module.name


def collect_callback_intrinsics(sys: "SysBuilder") -> CallbackMetadata:
    """Helper function to collect callback metadata from a system."""
    metadata = CallbackMetadata()
    collector = CallbackIntrinsicCollector(metadata)
    collector.collect(sys)
    set_current_callback_metadata(metadata)
    return metadata
