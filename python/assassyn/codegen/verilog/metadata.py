"""Metadata structures for tracking information during Verilog code generation.

This module provides dataclasses to hold metadata collected during the code generation
pass that needs to be referenced in later compilation phases (e.g., during top-level
harness generation).
"""

from dataclasses import dataclass, field
from typing import Dict, List, Tuple, TYPE_CHECKING, Any

if TYPE_CHECKING:
    from ...ir.expr import FIFOPush, AsyncCall, FIFOPop, ArrayRead
    from ...ir.array import Array
    from ...ir.module import Module
else:
    FIFOPush = Any  # type: ignore
    AsyncCall = Any  # type: ignore
    FIFOPop = Any  # type: ignore
    ArrayRead = Any  # type: ignore
    Array = Any  # type: ignore
    Module = Any  # type: ignore

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

    @property
    def pushes(self) -> List['FIFOPush']:
        """Backwards-compatible access to FIFO push expressions."""
        return self.fifo.push_exprs()

    @property
    def pops(self) -> List['FIFOPop']:
        """Backwards-compatible access to FIFO pop expressions."""
        return self.fifo.pop_exprs()

    # Future extensions:
    # has_wait_until: bool = False
    # array_usage: Optional[List[Array]] = None
