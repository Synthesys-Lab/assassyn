"""Array metadata utilities for Verilog code generation."""

from __future__ import annotations

from typing import Any, Dict, Iterable, List, Optional, Tuple, TYPE_CHECKING

from .metadata import ArrayMetadata
from ...ir.array import Array
from ...ir.memory.base import MemoryBase
from ...ir.expr import ArrayRead, ArrayWrite, Expr
from ...builder import SysBuilder

if TYPE_CHECKING:
    from ...ir.module import Module
else:
    Module = Any  # type: ignore


class ArrayMetadataRegistry:
    """Collects and serves metadata for shared IR arrays."""

    def __init__(self) -> None:
        self._metadata: Dict[Array, ArrayMetadata] = {}
        self._read_expr_lookup: Dict[ArrayRead, Tuple[Array, int]] = {}

    def clear(self) -> None:
        """Drop all cached metadata."""
        self._metadata.clear()
        self._read_expr_lookup.clear()

    def collect(self, sys: SysBuilder) -> None:
        """Populate the registry by analysing the given system.

        Module bodies are already flattened (see `DONE-remove-block`), so we can
        iterate each body's entries directly without relying on CIRCTDumper
        helpers.
        """
        self.clear()
        modules: List[Module] = list(sys.modules) + list(sys.downstreams)
        for arr in sys.arrays:
            owner = arr.owner
            if isinstance(owner, MemoryBase) and arr.is_payload(owner):
                continue

            writers = arr.get_write_ports().keys()
            for module in writers:
                self.register_writer(arr, module)
            for module in modules:
                body = getattr(module, "body", None)
                if body is None:
                    continue
                for expr in body:
                    if not isinstance(expr, Expr):
                        continue
                    if isinstance(expr, ArrayRead) and expr.array is arr:
                        self.register_read(arr, module, expr)
                    elif isinstance(expr, ArrayWrite) and expr.array is arr:
                        self.mark_user(arr, module)

    def ensure(self, array: Array) -> ArrayMetadata:
        """Return the metadata object for an array, creating it if needed."""
        meta = self._metadata.get(array)
        if meta is None:
            meta = ArrayMetadata(array=array)
            self._metadata[array] = meta
        return meta

    def metadata_for(self, array: Array) -> Optional[ArrayMetadata]:
        """Fetch metadata for an array if it exists."""
        return self._metadata.get(array)

    def arrays(self) -> Iterable[Array]:
        """Iterate over arrays tracked by the registry."""
        return self._metadata.keys()

    def register_writer(self, array: Array, module: Module) -> int:
        """Assign a write-port index to the given module."""
        meta = self.ensure(array)
        if module in meta.write_ports:
            return meta.write_ports[module]
        port_idx = len(meta.write_ports)
        meta.write_ports[module] = port_idx
        self.mark_user(array, module)
        return port_idx

    def register_read(self, array: Array, module: Module, expr: ArrayRead) -> int:
        """Assign a read-port index to the given expression and module."""
        meta = self.ensure(array)
        if expr in meta.read_expr_port:
            return meta.read_expr_port[expr]

        port_idx = len(meta.read_order)
        meta.read_expr_port[expr] = port_idx
        meta.read_order.append((module, expr))
        module_ports = meta.read_ports_by_module.setdefault(module, [])
        module_ports.append(port_idx)
        self._read_expr_lookup[expr] = (array, port_idx)
        self.mark_user(array, module)
        return port_idx

    def mark_user(self, array: Array, module: Module) -> None:
        """Record that a module touches the given array."""
        meta = self.ensure(array)
        if module not in meta.users:
            meta.users.append(module)

    def write_port_index(self, array: Array, module: Module) -> Optional[int]:
        """Lookup the write-port index for a module."""
        meta = self._metadata.get(array)
        if meta is None:
            return None
        return meta.write_ports.get(module)

    def write_port_count(self, array: Array) -> int:
        """Return the number of write ports for an array."""
        meta = self._metadata.get(array)
        return len(meta.write_ports) if meta else 0

    def read_port_indices(self, array: Array, module: Module) -> List[int]:
        """Return the read-port indices used by a module."""
        meta = self._metadata.get(array)
        if meta is None:
            return []
        return meta.read_ports_by_module.get(module, [])

    def read_port_count(self, array: Array) -> int:
        """Return the number of read ports for an array."""
        meta = self._metadata.get(array)
        return len(meta.read_order) if meta else 0

    def read_port_index_for_expr(self, expr: ArrayRead) -> Optional[int]:
        """Lookup the read-port index for a specific ArrayRead expression."""
        entry = self._read_expr_lookup.get(expr)
        if entry is None:
            return None
        return entry[1]

    def users_for(self, array: Array) -> List[Module]:
        """Return the modules that read or write the array."""
        meta = self._metadata.get(array)
        if meta is None:
            return []
        return list(meta.users)

__all__ = ["ArrayMetadataRegistry"]
