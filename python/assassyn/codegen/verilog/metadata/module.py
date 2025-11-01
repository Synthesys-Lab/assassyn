"""Module-scoped metadata helpers for Verilog code generation."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Mapping, NamedTuple, Tuple, TYPE_CHECKING

from .core import FIFOExpr, InteractionMatrix

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
    from ....ir.expr.intrinsic import Intrinsic
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
    from ....ir.expr.intrinsic import Intrinsic  # type: ignore
    from ....ir.module import Module, Port  # type: ignore


@dataclass
class ModuleBundle:
    """Mutable bucket of interactions gathered while analysing a module."""

    pushes: list[FIFOPush] = field(default_factory=list)
    pops: list[FIFOPop] = field(default_factory=list)
    fifo: dict[Port, list[FIFOExpr]] = field(default_factory=dict)
    writes: dict[Array, list[ArrayWrite]] = field(default_factory=dict)
    reads: dict[Array, list[ArrayRead]] = field(default_factory=dict)


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


@dataclass
class ModuleMetadata:  # pylint: disable=too-many-instance-attributes
    """Module-scoped metadata that decorates InteractionMatrix records."""

    module: Module
    matrix: InteractionMatrix
    _value_exposures: list["Expr"] = field(default_factory=list)
    _finish_sites: list["Intrinsic"] = field(default_factory=list)
    _calls: list["AsyncCall"] = field(default_factory=list)
    _value_snapshot: Tuple["Expr", ...] | None = field(init=False, default=None)
    _finish_snapshot: Tuple["Intrinsic", ...] | None = field(init=False, default=None)
    _calls_snapshot: Tuple["AsyncCall", ...] | None = field(init=False, default=None)
    _interactions: ModuleInteractionView | None = field(init=False, default=None)
    _frozen: bool = field(init=False, default=False)

    def record_value(self, expr: "Expr") -> None:
        """Track a value exposure encountered during analysis."""
        self._ensure_mutable()
        self._value_exposures.append(expr)

    def record_finish(self, expr: "Intrinsic") -> None:
        """Record a FINISH intrinsic so cleanup can emit completion logic."""
        self._ensure_mutable()
        self._finish_sites.append(expr)

    def record_call(self, call: "AsyncCall") -> None:
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
    def value_exposures(self) -> Tuple["Expr", ...]:
        """Return the value exposures recorded for the module."""
        if self._value_snapshot is not None:
            return self._value_snapshot
        return tuple(self._value_exposures)

    @property
    def finish_sites(self) -> Tuple["Intrinsic", ...]:
        """Return the FINISH intrinsics that terminate the module."""
        if self._finish_snapshot is not None:
            return self._finish_snapshot
        return tuple(self._finish_sites)

    @property
    def calls(self) -> Tuple["AsyncCall", ...]:
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
    "ModuleBundle",
    "ModuleInteractionView",
    "ModuleMetadata",
]
