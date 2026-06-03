"""Extract normalized validation models from backend metadata."""

from __future__ import annotations

from typing import TYPE_CHECKING

from ..codegen.verilog.schedule import compute_fifo_depths
from ..utils import namify
from .model import (
    AsyncCallTransition,
    FIFOTransition,
    ModuleTransition,
    RTLSignalMap,
    TriggerTransition,
    ValidationModel,
)

if TYPE_CHECKING:
    from ..builder import SysBuilder
    from ..codegen.verilog.metadata import InteractionMatrix, ModuleMetadata
    from ..ir.module import Module


def build_validation_model(
    sys: "SysBuilder",
    module_metadata: dict["Module", "ModuleMetadata"],
    interactions: "InteractionMatrix",
    default_fifo_depth: int,
) -> ValidationModel:
    """Build a normalized validation model from frozen Verilog metadata."""

    model = ValidationModel()
    fifo_depths = compute_fifo_depths(sys, module_metadata, default_fifo_depth)
    _add_module_transitions(model, sys)
    _add_fifo_transitions(model, fifo_depths)
    _add_async_call_transitions(model, sys, interactions)
    return model


def _add_module_transitions(model: ValidationModel, sys: "SysBuilder") -> None:
    """Add module fire and trigger-counter relations."""

    for module in list(sys.modules) + list(sys.downstreams):
        source_name = _source_module_name(module)
        module_id = f"module:{source_name}"
        module_name = namify(module.name)
        model.modules[module_id] = ModuleTransition(
            coverage_id=module_id,
            module=source_name,
            fire_signal=f"inst_{module_name}.executed",
            event_count_signal=f"{module_name}_trigger_count",
        )
        if module in sys.modules:
            model.triggers[module_id] = TriggerTransition(
                coverage_id=module_id,
                module=source_name,
                rtl_count_signal=f"{module_name}_trigger_count",
                rtl_delta_signal=f"{module_name}_trigger_counter_delta",
            )


def _add_fifo_transitions(
    model: ValidationModel,
    fifo_depths: dict["Module", dict["Port", int]],
) -> None:
    """Add FIFO queue relations."""

    for module, depth_by_port in fifo_depths.items():
        for port, depth_log2 in depth_by_port.items():
            module_name = namify(module.name)
            port_name = namify(port.name)
            source_name = _source_module_name(module)
            fifo_id = f"fifo:{source_name}.{port.name}"
            model.fifos[fifo_id] = FIFOTransition(
                coverage_id=fifo_id,
                module=source_name,
                port=port.name,
                configured_depth_log2=int(depth_log2),
                configured_depth=1 << int(depth_log2),
                rtl=RTLSignalMap(
                    count_signal=f"fifo_{module_name}_{port_name}_count",
                    valid_signal=f"fifo_{module_name}_{port_name}_pop_valid",
                    ready_signal=f"fifo_{module_name}_{port_name}_push_ready",
                    data_signal=f"fifo_{module_name}_{port_name}_pop_data",
                ),
            )


def _add_async_call_transitions(
    model: ValidationModel,
    sys: "SysBuilder",
    interactions: "InteractionMatrix",
) -> None:
    """Add async-call to FIFO alignment relations."""

    for callee in sys.modules:
        calls = interactions.async_ledger.calls_by_callee(callee)
        for index, call in enumerate(calls):
            caller = getattr(call, "parent", None)
            if caller is None:
                continue
            caller_name = _source_module_name(caller)
            callee_name = _source_module_name(callee)
            async_id = f"async:{caller_name}->{callee_name}:{index}"
            fifo_ids = tuple(
                f"fifo:{_source_module_name(push.fifo.module)}.{push.fifo.name}"
                for push in call.bind.pushes
            )
            model.async_calls[async_id] = AsyncCallTransition(
                coverage_id=async_id,
                caller=caller_name,
                callee=callee_name,
                fifo_ids=fifo_ids,
            )


def _source_module_name(module: "Module") -> str:
    """Return the source-level class name used for validation IDs."""

    return module.__class__.__name__
