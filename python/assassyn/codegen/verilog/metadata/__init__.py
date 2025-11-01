"""Public package surface for Verilog metadata structures."""

from __future__ import annotations

from .core import (
    FIFOExpr,
    AsyncLedger,
    InteractionKind,
    InteractionMatrix,
    __all__ as _CORE_EXPORTS,
)
from .module import ModuleBundle, ModuleInteractionView, ModuleMetadata
from .array import ArrayInteractionView, ArrayMetadata
from .fifo import FIFOInteractionView

__all__ = _CORE_EXPORTS + (
    "ModuleBundle",
    "ModuleInteractionView",
    "ModuleMetadata",
    "ArrayInteractionView",
    "ArrayMetadata",
    "FIFOInteractionView",
)
