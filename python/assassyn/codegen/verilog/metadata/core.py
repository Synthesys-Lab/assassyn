"""Core metadata structures shared across Verilog code generation."""

# pylint: disable=import-outside-toplevel, cyclic-import

from __future__ import annotations

from enum import Enum, auto
from types import MappingProxyType
from typing import Dict, Mapping, Tuple, TYPE_CHECKING, Union

if TYPE_CHECKING:
    from ....ir.array import Array
    from ....ir.expr import (
        ArrayRead,
        ArrayWrite,
        AsyncCall,
        Expr,
        FIFOPop,
        FIFOPush,
    )
    from ....ir.module import Module, Port
else:  # pragma: no cover - runtime imports only for type checking
    from ....ir.array import Array  # type: ignore
    from ....ir.expr import (  # type: ignore
        ArrayRead,
        ArrayWrite,
        AsyncCall,
        Expr,
        FIFOPop,
        FIFOPush,
    )
    from ....ir.module import Module, Port  # type: ignore

FIFOExpr = Union[FIFOPush, FIFOPop]


class InteractionKind(Enum):
    """Kinds of interactions recorded between modules and shared resources."""

    ARRAY_READ = auto()
    ARRAY_WRITE = auto()
    FIFO_PUSH = auto()
    FIFO_POP = auto()


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
        self._ensure_mutable()
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

    def _ensure_mutable(self) -> None:
        if self._frozen:
            raise RuntimeError("AsyncLedger is frozen; cannot record new entries")


class InteractionMatrix:  # pylint: disable=too-many-instance-attributes
    """Centralised interaction store keyed by (module, resource, role)."""

    def __init__(self) -> None:
        self._modules: Dict[Module, "ModuleBundle"] = {}
        self._fifos: Dict[Port, dict[str, list[FIFOExpr]]] = {}
        self._module_views: Dict[Module, "ModuleInteractionView"] | None = None
        self._array_views: Dict[Array, "ArrayInteractionView"] | None = None
        self._fifo_views: Dict[Port, "FIFOInteractionView"] | None = None
        self.async_ledger = AsyncLedger()
        self._frozen = False

    def record(
        self,
        *,
        module: Module,
        resource: Array | Port,
        kind: InteractionKind,
        expr: "Expr",
    ) -> None:
        """Record a single interaction emitted during analysis."""
        self._ensure_mutable()
        bundle = self._modules.get(module)
        if bundle is None:
            from .module import ModuleBundle  # local import to avoid circular dependency

            bundle = ModuleBundle()
            self._modules[module] = bundle
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

    def module_view(self, module: Module) -> "ModuleInteractionView":
        """Return the frozen view for *module*."""
        if not self._frozen or self._module_views is None:
            raise RuntimeError("InteractionMatrix is not frozen; module view unavailable")
        view = self._module_views.get(module)
        if view is None:
            from .module import ModuleInteractionView  # local import

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

    def array_view(self, array: Array) -> "ArrayInteractionView":
        """Return the frozen array-level view for *array*."""
        if not self._frozen or self._array_views is None:
            raise RuntimeError("InteractionMatrix is not frozen; array view unavailable")
        view = self._array_views.get(array)
        if view is None:
            raise KeyError(f"Array {array} has no recorded interactions")
        return view

    def fifo_view(self, port: Port) -> "FIFOInteractionView":
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

        from .array import ArrayInteractionView
        from .fifo import FIFOInteractionView
        from .module import ModuleInteractionView

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

        array_reads: Dict[Array, list["ArrayRead"]] = {}
        array_writers: Dict[Array, Dict[Module, list["ArrayWrite"]]] = {}
        array_reads_by_mod: Dict[Array, Dict[Module, list["ArrayRead"]]] = {}
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


__all__ = (
    "FIFOExpr",
    "InteractionKind",
    "InteractionMatrix",
    "AsyncLedger",
)
