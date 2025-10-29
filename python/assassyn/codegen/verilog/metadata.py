"""Metadata structures for tracking information during Verilog code generation.

This module provides dataclasses to hold metadata collected during the code generation
pass that needs to be referenced in later compilation phases (e.g., during top-level
harness generation).
"""

from collections import defaultdict
from dataclasses import dataclass, field
from typing import Dict, List, Tuple, TYPE_CHECKING, Any, Set

if TYPE_CHECKING:
    from ...ir.expr import FIFOPush, AsyncCall, FIFOPop, ArrayRead
    from ...ir.array import Array
    from ...ir.module import Module, Port
else:
    FIFOPush = Any  # type: ignore
    AsyncCall = Any  # type: ignore
    FIFOPop = Any  # type: ignore
    ArrayRead = Any  # type: ignore
    Array = Any  # type: ignore
    Module = Any  # type: ignore
    Port = Any  # type: ignore

CallList = List['AsyncCall']
ModuleList = List['Module']


@dataclass
class ArrayMetadata:
    """Metadata describing how an IR array is accessed throughout the system."""

    array: 'Array'
    write_ports: Dict['Module', int] = field(default_factory=dict)
    read_ports_by_module: Dict['Module', List[int]] = field(default_factory=dict)
    read_order: List[Tuple['Module', 'ArrayRead']] = field(default_factory=list)
    read_expr_port: Dict['ArrayRead', int] = field(default_factory=dict)
    users: ModuleList = field(default_factory=list)


@dataclass
class FIFOPushMetadata:
    """Metadata entry describing a FIFO push operation."""

    module: 'Module'
    expr: 'FIFOPush'
    predicate: str


@dataclass
class FIFOPopMetadata:
    """Metadata entry describing a FIFO pop operation."""

    module: 'Module'
    expr: 'FIFOPop'
    predicate: str


@dataclass
class FIFOMetadata:
    """Metadata container tracking FIFO pushes and pops with predicate context."""

    pushes: List[FIFOPushMetadata] = field(default_factory=list)
    pops: List[FIFOPopMetadata] = field(default_factory=list)

    def record_push(self, module: 'Module', expr: 'FIFOPush', predicate: str) -> None:
        """Record a FIFO push along with its predicate."""
        self.pushes.append(FIFOPushMetadata(module=module, expr=expr, predicate=predicate))

    def record_pop(self, module: 'Module', expr: 'FIFOPop', predicate: str) -> None:
        """Record a FIFO pop along with its predicate."""
        self.pops.append(FIFOPopMetadata(module=module, expr=expr, predicate=predicate))

    def push_exprs(self) -> List['FIFOPush']:
        """Return just the FIFO push expressions for backwards compatibility."""
        return [entry.expr for entry in self.pushes]

    def pop_exprs(self) -> List['FIFOPop']:
        """Return just the FIFO pop expressions for backwards compatibility."""
        return [entry.expr for entry in self.pops]


@dataclass
class ModuleMetadata:
    """Metadata collected during module code generation.
    
    This class holds information about a module that is discovered during the code
    generation pass and needs to be referenced later (e.g., during top-level harness
    generation).
    
    Attributes:
        has_finish: Whether the module contains a FINISH intrinsic. This flag is
            set to True when codegen_intrinsic encounters a FINISH operation, allowing
            top-level generation to determine which modules need their finish signals
            collected without walking the module body again.
        calls: List of AsyncCall expressions found in this module. Collected during
            expression generation to avoid redundant walking.
        fifo: Aggregated FIFO metadata capturing expressions, modules, and predicates.
    """
    has_finish: bool = False
    calls: CallList = field(default_factory=list)
    fifo: FIFOMetadata = field(default_factory=FIFOMetadata)
    fifo_by_port: Dict['Port', FIFOMetadata] = field(default_factory=dict)

    @property
    def pushes(self) -> List['FIFOPush']:
        """Backwards-compatible access to FIFO push expressions."""
        return self.fifo.push_exprs()

    @property
    def pops(self) -> List['FIFOPop']:
        """Backwards-compatible access to FIFO pop expressions."""
        return self.fifo.pop_exprs()

    def register_fifo_push(
        self,
        fifo_port: 'Port',
        fifo_metadata: FIFOMetadata,
        entry: FIFOPushMetadata,
    ) -> None:
        """Record a FIFO push in both the aggregate and per-port views."""
        self.fifo.pushes.append(entry)
        self._attach_fifo_metadata(fifo_port, fifo_metadata)

    def register_fifo_pop(
        self,
        fifo_port: 'Port',
        fifo_metadata: FIFOMetadata,
        entry: FIFOPopMetadata,
    ) -> None:
        """Record a FIFO pop in both the aggregate and per-port views."""
        self.fifo.pops.append(entry)
        self._attach_fifo_metadata(fifo_port, fifo_metadata)

    def _attach_fifo_metadata(
        self,
        fifo_port: 'Port',
        fifo_metadata: FIFOMetadata,
    ) -> None:
        existing = self.fifo_by_port.get(fifo_port)
        if existing is not None:
            # Ensure the registry stays consistent if the module revisits the port.
            if existing is not fifo_metadata:
                raise ValueError("FIFO metadata registry mismatch for port")
            return
        self.fifo_by_port[fifo_port] = fifo_metadata

    # Future extensions:
    # has_wait_until: bool = False
    # array_usage: Optional[List[Array]] = None


class FIFORegistry:
    """Maintain a FIFO-indexed view of metadata alongside per-module tracking."""

    def __init__(self) -> None:
        self._metadata_by_fifo: Dict['Port', FIFOMetadata] = {}
        self._fifos_by_module: Dict['Module', Set['Port']] = defaultdict(set)

    def metadata_for(self, fifo_port: 'Port') -> FIFOMetadata:
        """Return the metadata object for a FIFO port, creating it if needed."""
        metadata = self._metadata_by_fifo.get(fifo_port)
        if metadata is None:
            metadata = FIFOMetadata()
            self._metadata_by_fifo[fifo_port] = metadata
        return metadata

    def record_push(
        self,
        module: 'Module',
        expr: FIFOPush,
        predicate: str,
    ) -> Tuple[FIFOMetadata, FIFOPushMetadata]:
        """Record a FIFO push in the FIFO-indexed view."""
        fifo_port = expr.fifo
        fifo_metadata = self.metadata_for(fifo_port)
        entry = FIFOPushMetadata(module=module, expr=expr, predicate=predicate)
        fifo_metadata.pushes.append(entry)
        self._fifos_by_module[module].add(fifo_port)
        return fifo_metadata, entry

    def record_pop(
        self,
        module: 'Module',
        expr: FIFOPop,
        predicate: str,
    ) -> Tuple[FIFOMetadata, FIFOPopMetadata]:
        """Record a FIFO pop in the FIFO-indexed view."""
        fifo_port = expr.fifo
        fifo_metadata = self.metadata_for(fifo_port)
        entry = FIFOPopMetadata(module=module, expr=expr, predicate=predicate)
        fifo_metadata.pops.append(entry)
        self._fifos_by_module[module].add(fifo_port)
        return fifo_metadata, entry

    def clear_for_module(self, module: 'Module') -> None:
        """Remove any FIFO entries associated with the given module."""
        fifo_ports = self._fifos_by_module.pop(module, None)
        if not fifo_ports:
            return
        for fifo_port in fifo_ports:
            fifo_metadata = self._metadata_by_fifo.get(fifo_port)
            if fifo_metadata is None:
                continue
            fifo_metadata.pushes[:] = [
                entry for entry in fifo_metadata.pushes if entry.module is not module
            ]
            fifo_metadata.pops[:] = [
                entry for entry in fifo_metadata.pops if entry.module is not module
            ]
            if not fifo_metadata.pushes and not fifo_metadata.pops:
                self._metadata_by_fifo.pop(fifo_port, None)
