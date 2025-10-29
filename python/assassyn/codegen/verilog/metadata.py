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
    PushList = List[FIFOPush]
    CallList = List[AsyncCall]
    PopList = List[FIFOPop]
    ModuleList = List[Module]
else:
    PushList = List[Any]
    CallList = List[Any]
    PopList = List[Any]
    ModuleList = List[Any]
    Array = Any
    Module = Any
    ArrayRead = Any


@dataclass
class ArrayMetadata:
    """Metadata describing how an IR array is accessed throughout the system."""

    array: Array
    write_ports: Dict[Module, int] = field(default_factory=dict)
    read_ports_by_module: Dict[Module, List[int]] = field(default_factory=dict)
    read_order: List[Tuple[Module, ArrayRead]] = field(default_factory=list)
    read_expr_port: Dict[ArrayRead, int] = field(default_factory=dict)
    users: ModuleList = field(default_factory=list)


@dataclass
class PostDesignGeneration:
    """Metadata collected during module code generation.
    
    This class holds information about a module that is discovered during the code
    generation pass and needs to be referenced later (e.g., during top-level harness
    generation).
    
    Attributes:
        has_finish: Whether the module contains a FINISH intrinsic. This flag is
            set to True when codegen_intrinsic encounters a FINISH operation, allowing
            top-level generation to determine which modules need their finish signals
            collected without walking the module body again.
        pushes: List of FIFOPush expressions found in this module. Collected during
            expression generation to avoid redundant walking.
        calls: List of AsyncCall expressions found in this module. Collected during
            expression generation to avoid redundant walking.
        pops: List of FIFOPop expressions found in this module. Collected during
            expression generation to avoid redundant walking.
    """
    has_finish: bool = False
    pushes: PushList = field(default_factory=list)
    calls: CallList = field(default_factory=list)
    pops: PopList = field(default_factory=list)
    # Future extensions:
    # has_wait_until: bool = False
    # array_usage: Optional[List[Array]] = None
