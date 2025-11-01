"""FIFO-centric metadata helpers for Verilog code generation."""

from __future__ import annotations

from typing import NamedTuple, Tuple, TYPE_CHECKING

if TYPE_CHECKING:
    from ....ir.expr import FIFOPop, FIFOPush
else:  # pragma: no cover - runtime imports only for type checking
    from ....ir.expr import FIFOPop, FIFOPush  # type: ignore


class FIFOInteractionView(NamedTuple):
    """FIFO-centric view of pushes and pops recorded in the matrix."""

    pushes: Tuple["FIFOPush", ...]
    pops: Tuple["FIFOPop", ...]


__all__ = [
    "FIFOInteractionView",
]
