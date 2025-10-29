"""Shared predicate stack utilities for Verilog code generation and analysis."""

from __future__ import annotations

from dataclasses import dataclass
from typing import List, Optional, TYPE_CHECKING

from .utils import ensure_bits

if TYPE_CHECKING:
    from ...ir.expr import Expr


@dataclass
class PredicateFrame:
    """Single predicate frame containing the formatted condition and its origin."""

    text: str
    origin: Optional["Expr"] = None


class PredicateStack:
    """Utility to manage predicate stacking with shared formatting semantics."""

    def __init__(self) -> None:
        self._frames: List[PredicateFrame] = []

    def reset(self) -> None:
        """Clear the stack to its initial state."""
        self._frames.clear()

    def push(self, text: str, origin: Optional["Expr"] = None) -> None:
        """Push a predicate frame onto the stack."""
        self._frames.append(PredicateFrame(text=text, origin=origin))

    def pop(self) -> None:
        """Pop the most recent predicate frame when present."""
        if self._frames:
            self._frames.pop()

    def predicate(self) -> str:
        """Return the combined predicate string in Bits-form."""
        if not self._frames:
            return "Bits(1)(1)"
        parts = [ensure_bits(frame.text) for frame in self._frames]
        return " & ".join(parts)

    @property
    def frames(self) -> List[PredicateFrame]:
        """Expose the underlying predicate frames."""
        return self._frames
