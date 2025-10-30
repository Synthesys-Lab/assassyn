"""Metadata structures for tracking information during Verilog code generation.

This module provides dataclasses to hold metadata collected during the code generation
pass that needs to be referenced in later compilation phases (e.g., during top-level
handoff).
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, Iterator, List, Sequence, TYPE_CHECKING, Tuple

if TYPE_CHECKING:
    from ...ir.array import Array
    from ...ir.expr import ArrayRead, ArrayWrite, AsyncCall, Expr, FIFOPop, FIFOPush
    from ...ir.module import Module, Port
    from ...ir.value import Value
else:
    Array = Any  # type: ignore
    ArrayRead = Any  # type: ignore
    ArrayWrite = Any  # type: ignore
    AsyncCall = Any  # type: ignore
    Expr = Any  # type: ignore
    FIFOPop = Any  # type: ignore
    FIFOPush = Any  # type: ignore
    Module = Any  # type: ignore
    Port = Any  # type: ignore
    Value = Any  # type: ignore

CallList = List[AsyncCall]
ModuleList = List[Module]
ArrayReadList = List['ArrayRead']
ArrayWriteList = List['ArrayWrite']


@dataclass(frozen=True)
class ArrayWriteExposure:
    """Exposure metadata for a single array write expression."""

    expr: 'ArrayWrite'
    predicate: 'Value'


@dataclass(frozen=True)
class ArrayReadExposure:
    """Exposure metadata for an array read expression."""

    expr: 'ArrayRead'


@dataclass(frozen=True)
class ValueExposure:
    """Metadata describing a valued expression that must be exposed externally."""

    expr: 'Expr'
    predicate: 'Value'


@dataclass(frozen=True)
class AsyncTriggerExposure:
    """Metadata describing an async call that contributes to a trigger sum."""

    call: 'AsyncCall'
    predicate: 'Value'


@dataclass
class ArrayExposure:
    """Aggregated exposure data for a given array within a module."""

    array: Array
    writes_by_module: Dict[Module, Tuple[ArrayWriteExposure, ...]] = field(default_factory=dict)
    reads: Tuple[ArrayReadExposure, ...] = ()

    def add_write(self, module: Module, exposure: ArrayWriteExposure) -> None:
        """Record an array write produced by *module*."""
        writes = list(self.writes_by_module.get(module, ()))
        writes.append(exposure)
        self.writes_by_module[module] = tuple(writes)

    def add_read(self, exposure: ArrayReadExposure) -> None:
        """Record an array read exposure."""
        self.reads = self.reads + (exposure,)


class ModuleExposure:
    """Mutable exposure accumulator for a module, frozen post-analysis."""

    __slots__ = (
        "_arrays",
        "_values",
        "_async_triggers",
        "_frozen",
    )

    def __init__(self) -> None:
        self._arrays: Dict[Array, ArrayExposure] = {}
        self._values: List[ValueExposure] = []
        self._async_triggers: Dict[Module, List[AsyncTriggerExposure]] = {}
        self._frozen = False

    @property
    def arrays(self) -> Dict[Array, ArrayExposure]:
        """Return array exposure data keyed by the IR array."""
        return self._arrays

    @property
    def values(self) -> Tuple[ValueExposure, ...]:
        """Return the value exposures that must surface as module outputs."""
        if isinstance(self._values, tuple):
            return self._values
        return tuple(self._values)

    @property
    def async_triggers(self) -> Dict[Module, Tuple[AsyncTriggerExposure, ...]]:
        """Return async trigger exposures grouped by callee module."""
        return {
            module: tuple(entries) if not isinstance(entries, tuple) else entries
            for module, entries in self._async_triggers.items()
        }

    def record_array_write(
        self,
        array: Array,
        module: Module,
        expr: 'ArrayWrite',
        predicate: 'Value',
    ) -> None:
        """Capture an array write exposure for *array* performed by *module*."""
        self._ensure_mutable()
        exposure = ArrayWriteExposure(expr=expr, predicate=predicate)
        bucket = self._arrays.setdefault(array, ArrayExposure(array))
        bucket.add_write(module, exposure)

    def record_array_read(self, array: Array, expr: 'ArrayRead') -> None:
        """Capture an array read exposure for *array*."""
        self._ensure_mutable()
        exposure = ArrayReadExposure(expr=expr)
        bucket = self._arrays.setdefault(array, ArrayExposure(array))
        bucket.add_read(exposure)

    def record_value(self, expr: 'Expr', predicate: 'Value') -> None:
        """Capture a valued expression that must be exposed externally."""
        self._ensure_mutable()
        self._values.append(ValueExposure(expr=expr, predicate=predicate))

    def record_async_trigger(self, callee: Module, call: 'AsyncCall', predicate: 'Value') -> None:
        """Record an async trigger exposure for a specific callee module."""
        self._ensure_mutable()
        entry = AsyncTriggerExposure(call=call, predicate=predicate)
        self._async_triggers.setdefault(callee, []).append(entry)

    def freeze(self) -> None:
        """Prevent further mutation and canonicalise collection types."""
        if self._frozen:
            return
        for array, exposure in list(self._arrays.items()):
            exposure.reads = tuple(exposure.reads)
            exposure.writes_by_module = {
                module: tuple(entries)
                for module, entries in exposure.writes_by_module.items()
            }
            self._arrays[array] = exposure
        if not isinstance(self._values, tuple):
            self._values = tuple(self._values)
        for module, entries in list(self._async_triggers.items()):
            if not isinstance(entries, tuple):
                self._async_triggers[module] = tuple(entries)
        self._frozen = True

    def _ensure_mutable(self) -> None:
        if self._frozen:
            raise RuntimeError("ModuleExposure is frozen; cannot record new entries")


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
    predicate: Value
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
    exposures: ModuleExposure = field(default_factory=ModuleExposure)
    fifo: ModuleFIFOView = field(init=False)

    def __post_init__(self) -> None:
        self.fifo = ModuleFIFOView(self.module, self.registry)

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

    def record_push(self, module: Module, expr: FIFOPush, predicate: Value) -> FIFOInteraction:
        """Record a FIFO push performed by `module`."""
        fifo_port = expr.fifo
        interaction = FIFOInteraction(module=module, expr=expr, predicate=predicate, is_push=True)
        metadata = self.metadata_for(fifo_port)
        metadata.record_interaction(interaction)
        return interaction

    def record_pop(self, module: Module, expr: FIFOPop, predicate: Value) -> FIFOInteraction:
        """Record a FIFO pop performed by `module`."""
        fifo_port = expr.fifo
        interaction = FIFOInteraction(module=module, expr=expr, predicate=predicate, is_push=False)
        metadata = self.metadata_for(fifo_port)
        metadata.record_interaction(interaction)
        return interaction
