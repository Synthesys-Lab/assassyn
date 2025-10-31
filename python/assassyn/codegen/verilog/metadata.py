"""Metadata structures for tracking information during Verilog code generation.

This module provides dataclasses to hold metadata collected during the code generation
pass that needs to be referenced in later compilation phases (e.g., during top-level
handoff).
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Dict, Iterator, List, Sequence, TYPE_CHECKING, Tuple

if TYPE_CHECKING:
    from ...ir.array import Array
    from ...ir.expr import ArrayRead, ArrayWrite, AsyncCall, Expr, FIFOPop, FIFOPush
    from ...ir.expr.intrinsic import Intrinsic
    from ...ir.module import Module, Port
    from ...ir.value import Value
else:
    from ...ir.array import Array  # type: ignore
    from ...ir.expr import ArrayRead, ArrayWrite, AsyncCall, Expr, FIFOPop, FIFOPush  # type: ignore
    from ...ir.expr.intrinsic import Intrinsic  # type: ignore
    from ...ir.module import Module, Port  # type: ignore
    from ...ir.value import Value  # type: ignore

CallList = List[AsyncCall]
ModuleList = List[Module]


@dataclass
class ArrayExposure:
    """Aggregated exposure data for a given array within a module."""

    array: Array
    writes_by_module: Dict[Module, Tuple[ArrayWrite, ...]] = field(default_factory=dict)
    reads: Tuple[ArrayRead, ...] = ()

    def add_write(self, module: Module, exposure: ArrayWrite) -> None:
        """Record an array write produced by *module*."""
        writes = list(self.writes_by_module.get(module, ()))
        writes.append(exposure)
        self.writes_by_module[module] = tuple(writes)

    def add_read(self, exposure: ArrayRead) -> None:
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
        self._values: List[Expr] = []
        self._async_triggers: Dict[Module, List[AsyncCall]] = {}
        self._frozen = False

    @property
    def arrays(self) -> Dict[Array, ArrayExposure]:
        """Return array exposure data keyed by the IR array."""
        return self._arrays

    @property
    def values(self) -> Tuple[Expr, ...]:
        """Return the value exposures that must surface as module outputs."""
        if isinstance(self._values, tuple):
            return self._values
        return tuple(self._values)

    @property
    def async_triggers(self) -> Dict[Module, Tuple[AsyncCall, ...]]:
        """Return async trigger exposures grouped by callee module."""
        return {
            module: tuple(entries) if not isinstance(entries, tuple) else entries
            for module, entries in self._async_triggers.items()
        }

    def record_array_write(
        self,
        array: Array,
        module: Module,
        expr: ArrayWrite,
    ) -> None:
        """Capture an array write exposure for *array* performed by *module*."""
        self._ensure_mutable()
        bucket = self._arrays.setdefault(array, ArrayExposure(array))
        bucket.add_write(module, expr)

    def record_array_read(self, array: Array, expr: ArrayRead) -> None:
        """Capture an array read exposure for *array*."""
        self._ensure_mutable()
        bucket = self._arrays.setdefault(array, ArrayExposure(array))
        bucket.add_read(expr)

    def record_value(
        self,
        expr: Expr,
    ) -> None:
        """Capture a valued expression that must be exposed externally."""
        self._ensure_mutable()
        self._values.append(expr)

    def record_async_trigger(
        self,
        callee: Module,
        call: AsyncCall,
    ) -> None:
        """Record an async trigger exposure for a specific callee module."""
        self._ensure_mutable()
        self._async_triggers.setdefault(callee, []).append(call)

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
            # Snap the list to a tuple so downstream property access does not rebuild copies.
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


class FIFOMetadata:
    """Per-FIFO metadata owned by the global registry."""

    def __init__(self) -> None:
        self._pushes: list[FIFOPush] | tuple[FIFOPush, ...] = []
        self._pops: list[FIFOPop] | tuple[FIFOPop, ...] = []
        self._frozen = False

    @property
    def pushes(self) -> Tuple[FIFOPush, ...]:
        """Return FIFO push interactions for this channel."""
        if isinstance(self._pushes, tuple):
            return self._pushes
        return tuple(self._pushes)

    @property
    def pops(self) -> Tuple[FIFOPop, ...]:
        """Return FIFO pop interactions for this channel."""
        if isinstance(self._pops, tuple):
            return self._pops
        return tuple(self._pops)

    def record_interaction(self, expr: FIFOPush | FIFOPop) -> None:
        """Append an interaction to the appropriate push/pop list."""
        self._ensure_mutable()
        if isinstance(expr, FIFOPush):
            assert not isinstance(self._pushes, tuple)
            self._pushes.append(expr)
        elif isinstance(expr, FIFOPop):
            assert not isinstance(self._pops, tuple)
            self._pops.append(expr)
        else:
            raise TypeError(f"Unsupported FIFO expression: {expr!r}")

    def iter_interactions(self) -> Iterator[FIFOPush | FIFOPop]:
        """Yield every interaction associated with this FIFO channel."""
        yield from self.pushes
        yield from self.pops

    def interactions_for_module(self, module: Module) -> Tuple[FIFOPush | FIFOPop, ...]:
        """Return the interactions emitted by the provided module."""
        return tuple(
            expr for expr in self.iter_interactions() if getattr(expr, "parent", None) is module
        )

    def is_empty(self) -> bool:
        """Return True when no interactions remain for this FIFO."""
        return not self.pushes and not self.pops

    def freeze(self) -> None:
        """Prevent further mutation and canonicalise collection types."""
        if self._frozen:
            return
        if not isinstance(self._pushes, tuple):
            # Tuples guarantee a stable snapshot and prevent accidental mutation post-analysis.
            self._pushes = tuple(self._pushes)
        if not isinstance(self._pops, tuple):
            self._pops = tuple(self._pops)
        self._frozen = True

    def _ensure_mutable(self) -> None:
        if self._frozen:
            raise RuntimeError("FIFOMetadata is frozen; cannot record new interactions")


class ModuleFIFOView:
    """Module-scoped view over registry-owned FIFO interactions."""

    def __init__(self, module: Module, registry: FIFORegistry) -> None:
        self._module = module
        self._registry = registry
        self._ports: Dict[Port, None] = {}
        self._interactions_by_port: Dict[
            Port, list[FIFOPush | FIFOPop] | tuple[FIFOPush | FIFOPop, ...]
        ] = {}
        self._pushes: list[FIFOPush] | tuple[FIFOPush, ...] = []
        self._pops: list[FIFOPop] | tuple[FIFOPop, ...] = []
        self._frozen = False

    @property
    def ports(self) -> Sequence[Port]:
        """Return the FIFO ports touched by the owning module."""
        return tuple(self._ports)

    @property
    def pushes(self) -> Tuple[FIFOPush, ...]:
        """Return the FIFO pushes recorded for this module."""
        if isinstance(self._pushes, tuple):
            return self._pushes
        return tuple(self._pushes)

    @property
    def pops(self) -> Tuple[FIFOPop, ...]:
        """Return the FIFO pops recorded for this module."""
        if isinstance(self._pops, tuple):
            return self._pops
        return tuple(self._pops)

    def register(self, fifo_port: Port, expr: FIFOPush | FIFOPop) -> None:
        """Record a FIFO interaction for the owning module."""
        self._ensure_mutable()
        self._ports.setdefault(fifo_port, None)
        bucket = self._interactions_by_port.setdefault(fifo_port, [])
        assert not isinstance(bucket, tuple)
        bucket.append(expr)
        if isinstance(expr, FIFOPush):
            assert not isinstance(self._pushes, tuple)
            self._pushes.append(expr)
        elif isinstance(expr, FIFOPop):
            assert not isinstance(self._pops, tuple)
            self._pops.append(expr)
        else:
            raise TypeError(f"Unsupported FIFO expression: {expr!r}")

    def interactions_for(self, fifo_port: Port) -> Tuple[FIFOPush | FIFOPop, ...]:
        """Fetch the module's interactions associated with the given FIFO port."""
        interactions = self._interactions_by_port.get(fifo_port, ())
        return tuple(interactions)

    def iter_channels(self) -> Iterator[tuple[Port, FIFOMetadata, Tuple[FIFOPush | FIFOPop, ...]]]:
        """Yield `(fifo_port, metadata, interactions)` triples for the module."""
        for fifo_port in self._ports:
            metadata = self._registry.metadata_for(fifo_port)
            interactions = self._interactions_by_port.get(fifo_port, ())
            yield fifo_port, metadata, tuple(interactions)

    def freeze(self) -> None:
        """Freeze stored FIFO expressions to prevent further mutation."""
        if self._frozen:
            return
        if not isinstance(self._pushes, tuple):
            # Emitters query pushes repeatedly, so capture a stable tuple snapshot.
            self._pushes = tuple(self._pushes)
        if not isinstance(self._pops, tuple):
            self._pops = tuple(self._pops)
        for port, interactions in list(self._interactions_by_port.items()):
            if not isinstance(interactions, tuple):
                self._interactions_by_port[port] = tuple(interactions)
        self._frozen = True

    def _ensure_mutable(self) -> None:
        if self._frozen:
            raise RuntimeError("ModuleFIFOView is frozen; cannot record new interactions")


@dataclass
class ModuleMetadata:
    """Metadata collected during module code generation."""

    module: Module
    registry: FIFORegistry
    calls: CallList = field(default_factory=list)
    exposures: ModuleExposure = field(default_factory=ModuleExposure)
    fifo: ModuleFIFOView = field(init=False)
    _finish_sites: List[Intrinsic] = field(init=False, default_factory=list)
    _frozen: bool = field(init=False, default=False)

    def __post_init__(self) -> None:
        self.fifo = ModuleFIFOView(self.module, self.registry)
        self._finish_sites = []
        self._frozen = False

    @property
    def pushes(self) -> Tuple[FIFOPush, ...]:
        """Expose FIFO push expressions recorded for this module."""
        return self.fifo.pushes

    @property
    def pops(self) -> Tuple[FIFOPop, ...]:
        """Expose FIFO pop expressions recorded for this module."""
        return self.fifo.pops

    def record_fifo_interaction(self, fifo_port: Port, expr: FIFOPush | FIFOPop) -> None:
        """Track an interaction produced by this module on the given FIFO port."""
        if self._frozen:
            raise RuntimeError("ModuleMetadata is frozen; cannot record FIFO interactions")
        self.fifo.register(fifo_port, expr)

    @property
    def finish_sites(self) -> Tuple[Intrinsic, ...]:
        """Return recorded FINISH intrinsics for this module."""
        if isinstance(self._finish_sites, tuple):
            return self._finish_sites
        return tuple(self._finish_sites)

    def record_finish(self, expr: Intrinsic) -> None:
        """Record a FINISH intrinsic encountered during analysis."""
        if self._frozen:
            raise RuntimeError("ModuleMetadata is frozen; cannot record finish sites")
        self._finish_sites.append(expr)

    def freeze(self) -> None:
        """Prevent further mutation of metadata collections."""
        if self._frozen:
            return
        self.exposures.freeze()
        self.fifo.freeze()
        if not isinstance(self._finish_sites, tuple):
            # Converting to tuple provides a deterministic, read-only view for emitters.
            self._finish_sites = tuple(self._finish_sites)
        self._frozen = True


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

    def record_interaction(
        self,
        _module: Module,
        expr: FIFOPush | FIFOPop,
        _predicate: Value | None,
    ) -> FIFOPush | FIFOPop:
        """Record a FIFO interaction driven by `module`."""
        fifo_port = expr.fifo
        metadata = self.metadata_for(fifo_port)
        metadata.record_interaction(expr)
        return expr

    def record_push(
        self,
        _module: Module,
        expr: FIFOPush,
        predicate: Value | None,
    ) -> FIFOPush:
        """Record a FIFO push performed by `module`."""
        return self.record_interaction(_module, expr, predicate)

    def record_pop(
        self,
        _module: Module,
        expr: FIFOPop,
        predicate: Value | None,
    ) -> FIFOPop:
        """Record a FIFO pop performed by `module`."""
        return self.record_interaction(_module, expr, predicate)

    def freeze(self) -> None:
        """Freeze all FIFO metadata managed by the registry."""
        for metadata in self._metadata_by_fifo.values():
            metadata.freeze()
