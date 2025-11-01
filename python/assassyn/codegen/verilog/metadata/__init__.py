"""Public package surface for Verilog metadata structures."""

from __future__ import annotations

from .core import (
    FIFOExpr,
    AsyncLedger,
    InteractionKind,
    InteractionMatrix,
    __all__ as _CORE_EXPORTS,
)
from .module import (
    ModuleBundle,
    ModuleInteractionView,
    ModuleMetadata,
    __all__ as _MODULE_EXPORTS,
)
from .array import ArrayInteractionView, ArrayMetadata, __all__ as _ARRAY_EXPORTS
from .fifo import FIFOInteractionView, __all__ as _FIFO_EXPORTS
from .external import (
    ExternalRead,
    ExternalRegistry,
    __all__ as _EXTERNAL_EXPORTS,
)

__all__ = (
    *_CORE_EXPORTS,
    *_MODULE_EXPORTS,
    *_ARRAY_EXPORTS,
    *_FIFO_EXPORTS,
    *_EXTERNAL_EXPORTS,
)
