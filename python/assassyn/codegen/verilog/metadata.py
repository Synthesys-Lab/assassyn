"""Shared metadata structures for Verilog code generation."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum, auto
from types import MappingProxyType
from typing import (
    Dict,
    Mapping,
    NamedTuple,
    Tuple,
    List,
    TYPE_CHECKING,
)

if TYPE_CHECKING:
    from ...ir.array import Array
    from ...ir.expr import ArrayRead, ArrayWrite, AsyncCall, Expr, FIFOPop, FIFOPush
    from ...ir.expr.intrinsic import Intrinsic
    from ...ir.module import Module, Port
else:  # pragma: no cover - runtime imports only for type checking
    from ...ir.array import Array  # type: ignore
    from ...ir.expr import ArrayRead, ArrayWrite, AsyncCall, Expr, FIFOPop, FIFOPush  # type: ignore
    from ...ir.expr.intrinsic import Intrinsic  # type: ignore
    from ...ir.module import Module, Port  # type: ignore

FIFOExpr = FIFOPush | FIFOPop


class InteractionKind(Enum):
    """Kinds of interactions recorded between modules and shared resources."""

    ARRAY_READ = auto()
    ARRAY_WRITE = auto()
    FIFO_PUSH = auto()
    FIFO_POP = auto()


@dataclass
class ModuleBundle:
    """Mutable bucket of interactions gathered while analysing a module."""

    pushes: list[FIFOPush] = field(default_factory=list)
    pops: list[FIFOPop] = field(default_factory=list)
    fifo: dict[Port, list[FIFOExpr]] = field(default_factory=dict)
    writes: dict[Array, list[ArrayWrite]] = field(default_factory=dict)
    reads: dict[Array, list[ArrayRead]] = field(default_factory=dict)


@dataclass
class ArrayMetadata:
    """Compatibility container used by ArrayMetadataRegistry."""

    array: Array
    write_ports: Dict[Module, int] = field(default_factory=dict)
    read_ports_by_module: Dict[Module, List[int]] = field(default_factory=dict)
    read_order: List[Tuple[Module, ArrayRead]] = field(default_factory=list)
    read_expr_port: Dict[ArrayRead, int] = field(default_factory=dict)
    users: List[Module] = field(default_factory=list)


class AsyncLedger:
    """Book-keeping for async call relationships."""

    def __init__(self) -> None:
        """Initialise empty call maps."""
        self._by_module: Dict[Module, Dict[Module, list[AsyncCall]]] = {}
        self._by_callee: Dict[Module, list[AsyncCall]] = {}
        self._module_view: Dict[Module, Mapping[Module, Tuple[AsyncCall, ...]]] = {}
        self._callee_view: Dict[Module, Tuple[AsyncCall, ...]] = {}
        self._frozen = False

    def record(self, module: Module, callee: Module, call: AsyncCall) -> None:
        """Record an async call issued by *module* to *callee*."""
        if self._frozen:
            raise RuntimeError("AsyncLedger is frozen; cannot record new entries")
        self._by_module.setdefault(module, {}).setdefault(callee, []).append(call)
        self._by_callee.setdefault(callee, []).append(call)

    def calls_for_module(self, module: Module) -> Mapping[Module, Tuple[AsyncCall, ...]]:
        """Expose the frozen calls grouped by callee for *module*."""
        if not self._frozen:
            raise RuntimeError("AsyncLedger is not frozen")
        return self._module_view.get(module, MappingProxyType({}))

    def calls_by_callee(self, callee: Module) -> Tuple[AsyncCall, ...]:
        """Return all calls targeting *callee*."""
        if not self._frozen:
            raise RuntimeError("AsyncLedger is not frozen")
        return self._callee_view.get(callee, ())

    def freeze(self) -> None:
        """Convert the internal storage into immutable views."""
        if self._frozen:
            return
        self._module_view = {
            module: MappingProxyType({callee: tuple(calls) for callee, calls in by_callee.items()})
            for module, by_callee in self._by_module.items()
        }
        self._callee_view = {callee: tuple(calls) for callee, calls in self._by_callee.items()}
        self._frozen = True


class ModuleInteractionView(NamedTuple):
    """Immutable projection of interactions scoped to a module."""

    module: Module
    matrix: InteractionMatrix
    pushes: Tuple[FIFOPush, ...]
    pops: Tuple[FIFOPop, ...]
    fifo_ports: Tuple[Port, ...]
    fifo_map: Mapping[Port, Tuple[FIFOExpr, ...]]
    writes: Mapping[Array, Tuple[ArrayWrite, ...]]
    reads: Mapping[Array, Tuple[ArrayRead, ...]]


class ArrayInteractionView(NamedTuple):
    """Array-centric view of recorded reads and writes."""

    reads: Tuple[ArrayRead, ...]
    writers: Mapping[Module, Tuple[ArrayWrite, ...]]
    reads_by_module: Mapping[Module, Tuple[ArrayRead, ...]]


class FIFOInteractionView(NamedTuple):
    """FIFO-centric view of pushes and pops recorded in the matrix."""

    pushes: Tuple[FIFOPush, ...]
    pops: Tuple[FIFOPop, ...]


class InteractionMatrix:  # pylint: disable=too-many-instance-attributes
    """Centralised interaction store keyed by (module, resource, role)."""

    def __init__(self) -> None:
        self._modules: Dict[Module, ModuleBundle] = {}
        self._fifos: Dict[Port, dict[str, list[FIFOExpr]]] = {}
        self._module_views: Dict[Module, ModuleInteractionView] | None = None
        self._array_views: Dict[Array, ArrayInteractionView] | None = None
        self._fifo_views: Dict[Port, FIFOInteractionView] | None = None
        self.async_ledger = AsyncLedger()
        self._frozen = False

    def record(
        self,
        *,
        module: Module,
        resource: Array | Port,
        kind: InteractionKind,
        expr: Expr,
    ) -> None:
        """Record a single interaction emitted during analysis."""
        self._ensure_mutable()
        bundle = self._modules.setdefault(module, ModuleBundle())
        if isinstance(resource, Port):
            fifo = bundle.fifo.setdefault(resource, [])
            fifo.append(expr)
            fifo_bundle = self._fifos.setdefault(resource, {"pushes": [], "pops": []})
            if isinstance(expr, FIFOPush):
                bundle.pushes.append(expr)
                fifo_bundle["pushes"].append(expr)
            else:
                bundle.pops.append(expr)  # type: ignore[arg-type]
                fifo_bundle["pops"].append(expr)  # type: ignore[arg-type]
            return
        if kind is InteractionKind.ARRAY_WRITE:
            bundle.writes.setdefault(resource, []).append(expr)  # type: ignore[arg-type]
        elif kind is InteractionKind.ARRAY_READ:
            bundle.reads.setdefault(resource, []).append(expr)  # type: ignore[arg-type]
        else:
            raise TypeError(f"Unsupported array interaction kind: {kind}")

    def module_view(self, module: Module) -> ModuleInteractionView:
        """Return the frozen view for *module*."""
        if not self._frozen or self._module_views is None:
            raise RuntimeError("InteractionMatrix is not frozen; module view unavailable")
        view = self._module_views.get(module)
        if view is None:
            empty = ModuleInteractionView(
                module,
                self,
                (),
                (),
                (),
                MappingProxyType({}),
                MappingProxyType({}),
                MappingProxyType({}),
            )
            self._module_views[module] = empty
            return empty
        return view

    def array_view(self, array: Array) -> ArrayInteractionView:
        """Return the frozen array-level view for *array*."""
        if not self._frozen or self._array_views is None:
            raise RuntimeError("InteractionMatrix is not frozen; array view unavailable")
        view = self._array_views.get(array)
        if view is None:
            raise KeyError(f"Array {array} has no recorded interactions")
        return view

    def fifo_view(self, port: Port) -> FIFOInteractionView:
        """Return the frozen FIFO-level view for *port*."""
        if not self._frozen or self._fifo_views is None:
            raise RuntimeError("InteractionMatrix is not frozen; FIFO view unavailable")
        view = self._fifo_views.get(port)
        if view is None:
            raise KeyError(f"FIFO port {port} has no recorded interactions")
        return view

    def freeze(self) -> None:
        """Snapshot all recorded interactions into immutable views."""
        if self._frozen:
            return

        self.async_ledger.freeze()
        self._module_views = {
            module: ModuleInteractionView(
                module,
                self,
                tuple(bundle.pushes),
                tuple(bundle.pops),
                tuple(bundle.fifo.keys()),
                MappingProxyType(
                    {
                        port: tuple(exprs)
                        for port, exprs in bundle.fifo.items()
                    }
                ),
                MappingProxyType(
                    {
                        arr: tuple(exprs)
                        for arr, exprs in bundle.writes.items()
                    }
                ),
                MappingProxyType(
                    {
                        arr: tuple(exprs)
                        for arr, exprs in bundle.reads.items()
                    }
                ),
            )
            for module, bundle in self._modules.items()
        }

        array_reads: Dict[Array, list[ArrayRead]] = {}
        array_writers: Dict[Array, Dict[Module, list[ArrayWrite]]] = {}
        array_reads_by_mod: Dict[Array, Dict[Module, list[ArrayRead]]] = {}
        for module, bundle in self._modules.items():
            for array, writes in bundle.writes.items():
                array_writers.setdefault(array, {}).setdefault(module, []).extend(writes)
            for array, reads in bundle.reads.items():
                array_reads.setdefault(array, []).extend(reads)
                array_reads_by_mod.setdefault(array, {}).setdefault(module, []).extend(reads)

        self._array_views = {
            array: ArrayInteractionView(
                tuple(array_reads.get(array, ())),
                MappingProxyType(
                    {
                        mod: tuple(exprs)
                        for mod, exprs in array_writers.get(array, {}).items()
                    }
                ),
                MappingProxyType(
                    {
                        mod: tuple(exprs)
                        for mod, exprs in array_reads_by_mod.get(array, {}).items()
                    }
                ),
            )
            for array in array_reads.keys() | array_writers.keys()
        }

        self._fifo_views = {
            port: FIFOInteractionView(tuple(bundle["pushes"]), tuple(bundle["pops"]))
            for port, bundle in self._fifos.items()
        }

        self._frozen = True

    def _ensure_mutable(self) -> None:
        """Guard helper that prevents mutation after freeze()."""
        if self._frozen:
            raise RuntimeError("InteractionMatrix is frozen; cannot record new interactions")


@dataclass
class ModuleMetadata:  # pylint: disable=too-many-instance-attributes
    """Module-scoped metadata that decorates InteractionMatrix records."""

    module: Module
    matrix: InteractionMatrix
    _value_exposures: list[Expr] = field(default_factory=list)
    _finish_sites: list[Intrinsic] = field(default_factory=list)
    _calls: list[AsyncCall] = field(default_factory=list)
    _value_snapshot: Tuple[Expr, ...] | None = field(init=False, default=None)
    _finish_snapshot: Tuple[Intrinsic, ...] | None = field(init=False, default=None)
    _calls_snapshot: Tuple[AsyncCall, ...] | None = field(init=False, default=None)
    _interactions: ModuleInteractionView | None = field(init=False, default=None)
    _frozen: bool = field(init=False, default=False)

    def record_value(self, expr: Expr) -> None:
        """Track a value exposure encountered during analysis."""
        self._ensure_mutable()
        self._value_exposures.append(expr)

    def record_finish(self, expr: Intrinsic) -> None:
        """Record a FINISH intrinsic so cleanup can emit completion logic."""
        self._ensure_mutable()
        self._finish_sites.append(expr)

    def record_call(self, call: AsyncCall) -> None:
        """Register an async call issued by this module."""
        self._ensure_mutable()
        self._calls.append(call)

    def freeze(self) -> None:
        """Finalise the metadata and snapshot interaction projections."""
        if self._frozen:
            return
        self.matrix.freeze()
        self._value_snapshot = tuple(self._value_exposures)
        self._finish_snapshot = tuple(self._finish_sites)
        self._calls_snapshot = tuple(self._calls)
        self._value_exposures.clear()
        self._finish_sites.clear()
        self._calls.clear()
        self._interactions = self.matrix.module_view(self.module)
        self._frozen = True

    @property
    def value_exposures(self) -> Tuple[Expr, ...]:
        """Return the value exposures recorded for the module."""
        if self._value_snapshot is not None:
            return self._value_snapshot
        return tuple(self._value_exposures)

    @property
    def finish_sites(self) -> Tuple[Intrinsic, ...]:
        """Return the FINISH intrinsics that terminate the module."""
        if self._finish_snapshot is not None:
            return self._finish_snapshot
        return tuple(self._finish_sites)

    @property
    def calls(self) -> Tuple[AsyncCall, ...]:
        """Return async calls issued by the module."""
        if self._calls_snapshot is not None:
            return self._calls_snapshot
        return tuple(self._calls)

    @property
    def interactions(self) -> ModuleInteractionView:
        """Return the frozen interaction view for the module."""
        if self._interactions is None:
            raise RuntimeError("Module interactions are unavailable before freeze()")
        return self._interactions

    def _ensure_mutable(self) -> None:
        if self._frozen:
            raise RuntimeError("ModuleMetadata is frozen; cannot record new entries")


__all__ = [
    "InteractionKind",
    "InteractionMatrix",
    "ModuleInteractionView",
    "ArrayInteractionView",
    "FIFOInteractionView",
    "AsyncLedger",
    "ModuleMetadata",
    "ArrayMetadata",
]
