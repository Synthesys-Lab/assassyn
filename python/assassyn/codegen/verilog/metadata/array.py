"""Array-centric metadata helpers for Verilog code generation."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Dict, List, Mapping, NamedTuple, Tuple, TYPE_CHECKING

if TYPE_CHECKING:
    from ....ir.array import Array
    from ....ir.expr import ArrayRead, ArrayWrite
    from ....ir.module import Module
else:  # pragma: no cover - runtime imports only for type checking
    from ....ir.array import Array  # type: ignore
    from ....ir.expr import ArrayRead, ArrayWrite  # type: ignore
    from ....ir.module import Module  # type: ignore


class ArrayInteractionView(NamedTuple):
    """Array-centric view of recorded reads and writes."""

    reads: Tuple["ArrayRead", ...]
    writers: Mapping["Module", Tuple["ArrayWrite", ...]]
    reads_by_module: Mapping["Module", Tuple["ArrayRead", ...]]


@dataclass
class ArrayMetadata:
    """Compatibility container used by ArrayMetadataRegistry."""

    array: "Array"
    write_ports: Dict["Module", int] = field(default_factory=dict)
    read_ports_by_module: Dict["Module", List[int]] = field(default_factory=dict)
    read_order: List[Tuple["Module", "ArrayRead"]] = field(default_factory=list)
    read_expr_port: Dict["ArrayRead", int] = field(default_factory=dict)
    users: List["Module"] = field(default_factory=list)


__all__ = [
    "ArrayInteractionView",
    "ArrayMetadata",
]
