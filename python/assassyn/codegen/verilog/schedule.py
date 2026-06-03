"""Shared schedule equations for Verilog lowering and validation."""

from __future__ import annotations

from collections import defaultdict
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ...builder import SysBuilder
    from ...ir.expr import AsyncCall, FIFOPush
    from ...ir.module import Module, Port
    from .metadata import AsyncLedger, ModuleMetadata


def compute_fifo_depths(
    sys: "SysBuilder",
    module_metadata: dict["Module", "ModuleMetadata"],
    default_fifo_depth: int,
) -> dict["Module", dict["Port", int]]:
    """Return per-module, per-port FIFO depth log2 values."""

    depth_map = {
        module: {port: default_fifo_depth for port in getattr(module, "ports", [])}
        for module in list(sys.modules) + list(sys.downstreams)
    }

    for metadata in module_metadata.values():
        for push in metadata.interactions.pushes:
            fifo_port = push.fifo
            owner = fifo_port.module
            if owner not in depth_map:
                continue
            depth = push.fifo_depth
            if not isinstance(depth, int) or depth <= 0:
                depth = default_fifo_depth
            current = depth_map[owner].get(fifo_port, default_fifo_depth)
            depth_map[owner][fifo_port] = max(current, depth)

    return depth_map


def compute_trigger_widths(
    sys: "SysBuilder",
    fifo_depths: dict["Module", dict["Port", int]],
    default_fifo_depth: int,
) -> dict["Module", int]:
    """Return trigger-counter widths matching generated RTL rules."""

    widths: dict["Module", int] = {}
    for module in sys.modules:
        depth_by_port = fifo_depths.get(module, {})
        if not depth_by_port:
            depth_log2 = default_fifo_depth
        else:
            depths = list(depth_by_port.values())
            depth_log2 = depths[0]
            if any(depth != depth_log2 for depth in depths):
                raise RuntimeError(
                    f"Inconsistent FIFO depths for module {module.name}: {depths}"
                )
        widths[module] = max(1, int(depth_log2) + 1)
    return widths


def group_fifo_pushes(pushes) -> dict[tuple["Module", "Port"], tuple["FIFOPush", ...]]:
    """Group FIFO pushes by target module and port."""

    grouped: dict[tuple["Module", "Port"], list["FIFOPush"]] = defaultdict(list)
    for push in pushes:
        grouped[(push.fifo.module, push.fifo)].append(push)
    return {key: tuple(value) for key, value in grouped.items()}


def group_async_triggers(
    async_ledger: "AsyncLedger",
    module: "Module",
) -> dict["Module", tuple["AsyncCall", ...]]:
    """Return async calls issued by *module*, grouped by callee."""

    return dict(async_ledger.calls_for_module(module))
