"""Metadata structures for tracking information during Verilog code generation.

This module provides dataclasses to hold metadata collected during the code generation
pass that needs to be referenced in later compilation phases (e.g., during top-level
handoff).
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, Iterable, Iterator, List, Sequence, TYPE_CHECKING

if TYPE_CHECKING:
    from ...ir.array import Array
    from ...ir.expr import ArrayRead, AsyncCall, FIFOPop, FIFOPush
    from ...ir.module import Module, Port
else:
    Array = Any  # type: ignore
    ArrayRead = Any  # type: ignore
    AsyncCall = Any  # type: ignore
    FIFOPop = Any  # type: ignore
    FIFOPush = Any  # type: ignore
    Module = Any  # type: ignore
    Port = Any  # type: ignore

CallList = List[AsyncCall]
ModuleList = List[Module]


@dataclass
class ArrayMetadata:
    """Metadata describing how an IR array is accessed throughout the system."""

    array: Array
    write_ports: Dict[Module, int] = field(default_factory=dict)
    read_ports_by_module: Dict[Module, List[int]] = field(default_factory=dict)
    read_order: List[tuple[Module, ArrayRead]] = field(default_factory=list)
    read_expr_port: Dict[ArrayRead, int] = field(default_factory=dict)
    users: ModuleList = field(default_factory=list)


@dataclass
class FIFOInteraction:
    """Single FIFO interaction captured during code generation."""

    module: Module
    expr: FIFOPush | FIFOPop
    predicate: str
    is_push: bool


class FIFOMetadata:
    """Per-FIFO metadata owned by the global registry."""

    def __init__(self) -> None:
        self._pushes: List[FIFOInteraction] = []
        self._pops: List[FIFOInteraction] = []

    @property
    def pushes(self) -> List[FIFOInteraction]:
        """Return FIFO push interactions for this channel."""
        return self._pushes

    @property
    def pops(self) -> List[FIFOInteraction]:
        """Return FIFO pop interactions for this channel."""
        return self._pops

    def record_interaction(self, interaction: FIFOInteraction) -> None:
        """Append an interaction to the appropriate push/pop list."""
        if interaction.is_push:
            self._pushes.append(interaction)
        else:
            self._pops.append(interaction)

    def iter_interactions(self) -> Iterator[FIFOInteraction]:
        """Yield every interaction associated with this FIFO channel."""
        yield from self._pushes
        yield from self._pops

    def interactions_for_module(self, module: Module) -> List[FIFOInteraction]:
        """Return the interactions emitted by the provided module."""
        return [entry for entry in self.iter_interactions() if entry.module is module]

    def remove_module(self, module: Module) -> None:
        """Drop all interactions that belong to the provided module."""
        self._pushes[:] = [entry for entry in self._pushes if entry.module is not module]
        self._pops[:] = [entry for entry in self._pops if entry.module is not module]

    def is_empty(self) -> bool:
        """Return True when no interactions remain for this FIFO."""
        return not self._pushes and not self._pops


class ModuleFIFOView:
    """Module-scoped view over registry-owned FIFO interactions."""

    def __init__(self, module: Module, registry: FIFORegistry) -> None:
        self._module = module
        self._registry = registry
        self._ports: Dict[Port, None] = {}
        self._interactions_by_port: Dict[Port, List[FIFOInteraction]] = {}
        self.pushes: List[FIFOInteraction] = []
        self.pops: List[FIFOInteraction] = []

    def reset(self, registry: FIFORegistry | None = None) -> None:
        """Clear recorded interactions and optionally retarget the registry."""
        self._ports.clear()
        self._interactions_by_port.clear()
        self.pushes.clear()
        self.pops.clear()
        if registry is not None and registry is not self._registry:
            self._registry = registry

    @property
    def ports(self) -> Sequence[Port]:
        """Return the FIFO ports touched by the owning module."""
        return tuple(self._ports)

    def register(self, fifo_port: Port, interaction: FIFOInteraction) -> None:
        """Record a FIFO interaction for the owning module."""
        self._ports.setdefault(fifo_port, None)
        self._interactions_by_port.setdefault(fifo_port, []).append(interaction)
        if interaction.is_push:
            self.pushes.append(interaction)
        else:
            self.pops.append(interaction)

    def interactions_for(self, fifo_port: Port) -> List[FIFOInteraction]:
        """Fetch the module's interactions associated with the given FIFO port."""
        return list(self._interactions_by_port.get(fifo_port, ()))

    def iter_channels(self) -> Iterator[tuple[Port, FIFOMetadata, Sequence[FIFOInteraction]]]:
        """Yield `(fifo_port, metadata, interactions)` triples for the module."""
        for fifo_port in self._ports:
            metadata = self._registry.metadata_for(fifo_port)
            interactions = self._interactions_by_port.get(fifo_port, [])
            yield fifo_port, metadata, tuple(interactions)


@dataclass
class ModuleMetadata:
    """Metadata collected during module code generation."""

    module: Module
    registry: FIFORegistry
    has_finish: bool = False
    calls: CallList = field(default_factory=list)
    fifo: ModuleFIFOView = field(init=False)
    fifo_ready: bool = False

    def __post_init__(self) -> None:
        self.fifo = ModuleFIFOView(self.module, self.registry)

    def reset_for_analysis(self, registry: FIFORegistry) -> None:
        """Prepare FIFO metadata for a fresh analysis run."""
        if self.registry is not registry:
            self.registry = registry
            self.fifo = ModuleFIFOView(self.module, registry)
        else:
            self.fifo.reset()
        self.fifo_ready = False

    def mark_fifo_ready(self) -> None:
        """Mark FIFO metadata as populated by the analysis pre-pass."""
        self.fifo_ready = True

    def prepare_for_codegen(self) -> None:
        """Clear transient state ahead of code emission."""
        self.has_finish = False
        self.calls.clear()

    @property
    def pushes(self) -> List[FIFOPush]:
        """Expose FIFO push expressions recorded for this module."""
        return [entry.expr for entry in self.fifo.pushes]

    @property
    def pops(self) -> List[FIFOPop]:
        """Expose FIFO pop expressions recorded for this module."""
        return [entry.expr for entry in self.fifo.pops]

    def record_fifo_interaction(self, fifo_port: Port, interaction: FIFOInteraction) -> None:
        """Track an interaction produced by this module on the given FIFO port."""
        self.fifo.register(fifo_port, interaction)


class FIFORegistry:
    """Maintain FIFO metadata indexed by FIFO ports."""

    def __init__(self) -> None:
        self._metadata_by_fifo: Dict[Port, FIFOMetadata] = {}

    def metadata_for(self, fifo_port: Port) -> FIFOMetadata:
        """Return the metadata object for `fifo_port`, creating it when missing."""
        metadata = self._metadata_by_fifo.get(fifo_port)
        if metadata is None:
            metadata = FIFOMetadata()
            self._metadata_by_fifo[fifo_port] = metadata
        return metadata

    def record_push(self, module: Module, expr: FIFOPush, predicate: str) -> FIFOInteraction:
        """Record a FIFO push performed by `module`."""
        fifo_port = expr.fifo
        interaction = FIFOInteraction(module=module, expr=expr, predicate=predicate, is_push=True)
        metadata = self.metadata_for(fifo_port)
        metadata.record_interaction(interaction)
        return interaction

    def record_pop(self, module: Module, expr: FIFOPop, predicate: str) -> FIFOInteraction:
        """Record a FIFO pop performed by `module`."""
        fifo_port = expr.fifo
        interaction = FIFOInteraction(module=module, expr=expr, predicate=predicate, is_push=False)
        metadata = self.metadata_for(fifo_port)
        metadata.record_interaction(interaction)
        return interaction

    def reset(self) -> None:
        """Drop all recorded FIFO metadata."""
        self._metadata_by_fifo.clear()

    def clear_for_module(self, module: Module, fifo_ports: Iterable[Port] | None = None) -> None:
        """Remove every interaction produced by `module` across all FIFOs."""
        if fifo_ports is None:
            fifo_ports = [
                fifo_port
                for fifo_port, metadata in self._metadata_by_fifo.items()
                if metadata.interactions_for_module(module)
            ]
        else:
            fifo_ports = list(dict.fromkeys(fifo_ports))

        for fifo_port in fifo_ports:
            metadata = self._metadata_by_fifo.get(fifo_port)
            if metadata is None:
                continue
            metadata.remove_module(module)
            if metadata.is_empty():
                self._metadata_by_fifo.pop(fifo_port, None)
