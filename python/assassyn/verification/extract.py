"""Extract normalized validation models from backend metadata."""

from __future__ import annotations

from typing import TYPE_CHECKING

from ..codegen.verilog.array import ArrayMetadataRegistry
from ..codegen.verilog.schedule import compute_fifo_depths, compute_trigger_widths
from ..ir.memory.base import MemoryBase
from ..utils import namify
from .model import (
    ArrayReadPortTransition,
    ArrayTransition,
    ArrayWritePortTransition,
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
    trigger_widths = compute_trigger_widths(sys, fifo_depths, default_fifo_depth)
    _add_module_transitions(model, sys, trigger_widths)
    _add_fifo_transitions(model, fifo_depths)
    _add_array_transitions(model, sys)
    _add_async_call_transitions(model, sys, interactions)
    return model


def _add_module_transitions(
    model: ValidationModel,
    sys: "SysBuilder",
    trigger_widths: dict["Module", int],
) -> None:
    """Add module fire and trigger-counter relations."""

    for module in list(sys.modules) + list(sys.downstreams):
        source_name = _source_module_name(module)
        module_id = f"module:{source_name}"
        module_name = namify(module.name)
        model.modules[module_id] = ModuleTransition(
            coverage_id=module_id,
            module=source_name,
            fire_signal=f"inst_{module_name}.executed",
            event_count_signal=(
                f"{module_name}_trigger_counter_inst.count"
                if module in sys.modules
                else None
            ),
        )
        if module in sys.modules:
            width = int(trigger_widths[module])
            model.triggers[module_id] = TriggerTransition(
                coverage_id=module_id,
                module=source_name,
                rtl_count_signal=f"{module_name}_trigger_counter_inst.count",
                rtl_delta_signal=f"{module_name}_trigger_counter_delta",
                width=width,
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
                    count_signal=_fifo_count_signal(module_name, port_name, depth_log2),
                    count_width=max(1, int(depth_log2) + 1),
                    push_valid_signal=f"fifo_{module_name}_{port_name}_push_valid",
                    valid_signal=f"fifo_{module_name}_{port_name}_pop_valid",
                    ready_signal=f"fifo_{module_name}_{port_name}_push_ready",
                    pop_ready_signal=f"fifo_{module_name}_{port_name}_pop_ready",
                    data_signal=f"fifo_{module_name}_{port_name}_pop_data",
                    data_width=int(port.dtype.bits),
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


def _add_array_transitions(
    model: ValidationModel,
    sys: "SysBuilder",
) -> None:
    """Add RegArray read/write commit-boundary relations."""

    registry = ArrayMetadataRegistry()
    registry.collect(sys)
    for array in sys.arrays:
        owner = getattr(array, "owner", None)
        if isinstance(owner, MemoryBase) and array.is_payload(owner):
            continue

        metadata = registry.metadata_for(array)
        if metadata is None:
            continue
        if not metadata.write_ports and not metadata.read_order:
            continue

        array_name = namify(array.name)
        array_id = f"array:{array.name}"
        source_index_width = int(array.index_bits)
        index_width = max(1, source_index_width)
        write_ports = tuple(
            _array_write_port(array_name, module, port_idx)
            for module, port_idx in metadata.write_ports.items()
        )
        read_ports = tuple(
            _array_read_port(array_name, module, port_idx, source_index_width)
            for port_idx, (module, _expr) in enumerate(metadata.read_order)
        )
        model.arrays[array_id] = ArrayTransition(
            coverage_id=array_id,
            array=array.name,
            depth=int(array.size),
            index_width=index_width,
            data_width=int(array.scalar_ty.bits),
            write_ports=write_ports,
            read_ports=read_ports,
        )


def _array_write_port(
    array_name: str,
    module: "Module",
    port_idx: int,
) -> ArrayWritePortTransition:
    """Return the generated RTL mapping for one RegArray write port."""

    suffix = f"port{port_idx}"
    instance = f"array_writer_{array_name}"
    return ArrayWritePortTransition(
        writer=_source_module_name(module),
        port_index=int(port_idx),
        write_enable_signal=f"{instance}.w_{suffix}",
        write_index_signal=f"{instance}.widx_{suffix}",
        write_data_signal=f"{instance}.wdata_{suffix}",
        next_value_signal=f"{instance}.mem[{instance}.widx_{suffix}]",
    )


def _array_read_port(
    array_name: str,
    module: "Module",
    port_idx: int,
    source_index_width: int,
) -> ArrayReadPortTransition:
    """Return the generated RTL mapping for one RegArray read port."""

    suffix = f"port{port_idx}"
    instance = f"array_writer_{array_name}"
    return ArrayReadPortTransition(
        reader=_source_module_name(module),
        port_index=int(port_idx),
        read_index_signal=(
            f"{instance}.ridx_{suffix}" if source_index_width > 0 else None
        ),
        read_data_signal=f"{instance}.rdata_{suffix}",
    )


def _source_module_name(module: "Module") -> str:
    """Return the source-level class name used for validation IDs."""

    return module.__class__.__name__


def _fifo_count_signal(module_name: str, port_name: str, depth_log2: int) -> str:
    """Return the generated FIFO occupancy signal path."""

    prefix = f"fifo_{module_name}_{port_name}_inst"
    if int(depth_log2) == 0:
        return f"{prefix}.single_element_fifo.full"
    return f"{prefix}.multi_element_fifo.count"
