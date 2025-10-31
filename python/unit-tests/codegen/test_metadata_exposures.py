"""Verify that metadata pre-pass records exposures and flags."""

import os
import sys
from types import SimpleNamespace

import pytest

sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (  # type: ignore
    Module,
    SysBuilder,
    UInt,
    Port,
    RegArray,
    Condition,
    module,
    finish,
)
from assassyn.codegen.verilog.analysis import collect_fifo_metadata  # type: ignore
from assassyn.codegen.verilog.metadata import (  # type: ignore
    FIFORegistry,
    ModuleExposure,
    ModuleMetadata,
)
from assassyn.ir.expr.call import AsyncCall, FIFOPush  # type: ignore
from assassyn.ir.expr.expr import FIFOPop  # type: ignore
from assassyn.ir.expr.intrinsic import Intrinsic  # type: ignore


class Callee(Module):
    """Simple callee used to drive async trigger exposure metadata."""

    def __init__(self):
        super().__init__(ports={'input_port': Port(UInt(8))})

    @module.combinational
    def build(self):
        pass


def test_metadata_exposures_capture():
    """Ensure module metadata records finish flags, async calls, and array exposures."""

    sys_builder = SysBuilder("metadata_exposures")

    with sys_builder:

        class ExposureModule(Module):  # type: ignore[misc]

            def __init__(self):
                super().__init__(ports={
                    'cond': Port(UInt(1)),
                    'idx': Port(UInt(1)),
                    'value': Port(UInt(8)),
                    'fifo_in': Port(UInt(8)),
                    'fifo_out': Port(UInt(8)),
                })

            @module.combinational
            def build(self):
                cond = self.cond.pop()
                idx = self.idx.pop()
                value = self.value.pop()
                source = self.fifo_in.pop()
                target = self.fifo_out
                array = RegArray(UInt(8), 2, name="meta_store")
                callee = Callee()

                with Condition(cond):
                    self.finish_cond = cond
                    write_port = array & self
                    write_port[idx] <= value
                    target.push(value)
                    bound = callee.bind(input_port=source)
                    self.async_expr = bound.async_called(input_port=source)
                    finish()

        instance = ExposureModule()
        instance.build()

    module_metadata, fifo_registry = collect_fifo_metadata(sys_builder)
    dumper_metadata = module_metadata[instance]

    finish_sites = dumper_metadata.finish_sites
    assert finish_sites, "expected finish sites to be recorded"
    assert all(site.opcode == Intrinsic.FINISH for site in finish_sites)
    assert all(site.meta_cond is not None for site in finish_sites)
    assert finish_sites[0].meta_cond is instance.finish_cond
    assert len(dumper_metadata.calls) == 1
    assert isinstance(dumper_metadata.calls[0], AsyncCall)

    array_exposures = dumper_metadata.exposures.arrays
    assert array_exposures, "expected array exposures to be recorded"
    for exposure in array_exposures.values():
        assert exposure.writes_by_module, "array exposures should capture writes per module"
        for writer, writes in exposure.writes_by_module.items():
            assert writer is instance
            assert writes, "expected recorded array writes"
            assert all(hasattr(write, "meta_cond") for write in writes)
        # Reads are captured when gatherable; the current test exercises writes primarily.

    async_triggers = dumper_metadata.exposures.async_triggers
    assert async_triggers, "expected async trigger exposure metadata"
    for entries in async_triggers.values():
        assert entries, "async trigger entries should not be empty"
        for call in entries:
            assert call in dumper_metadata.calls
            assert getattr(call, "meta_cond", None) is not None

    for expr in dumper_metadata.exposures.values:
        assert getattr(expr, "meta_cond", None) is not None

    metadata_pushes = dumper_metadata.fifo.pushes
    assert metadata_pushes, "FIFO push metadata should be recorded"
    metadata_pops = dumper_metadata.fifo.pops
    assert metadata_pops, "FIFO pop metadata should be recorded"
    for expr in metadata_pushes + metadata_pops:
        assert getattr(expr, "meta_cond", None) is not None
    interactions_map = dumper_metadata.fifo.interactions_by_kind
    assert interactions_map[FIFOPush] == metadata_pushes
    assert interactions_map[FIFOPop] == metadata_pops

    # Verify FIFO registry mirrors module metadata
    fifo_ports = {expr.fifo for expr in metadata_pushes + metadata_pops}
    for fifo_port in fifo_ports:
        channel = fifo_registry.metadata_for(fifo_port)
        assert channel.pushes or channel.pops
        channel_map = channel.interactions_by_kind
        if channel.pushes:
            assert channel_map[FIFOPush] == channel.pushes
        if channel.pops:
            assert channel_map[FIFOPop] == channel.pops


def test_metadata_freeze_stabilizes_views():
    """Document why metadata stays mutable until frozen."""

    exposure = ModuleExposure()
    value_expr = SimpleNamespace(meta_cond=object())
    exposure.record_value(value_expr)
    assert exposure.values == (value_expr,)
    pre_snapshot = exposure.values
    assert pre_snapshot == (value_expr,)

    exposure.freeze()
    post_snapshot = exposure.values
    assert post_snapshot is exposure.values
    with pytest.raises(RuntimeError):
        exposure.record_value(SimpleNamespace(meta_cond=None))

    registry = FIFORegistry()

    class _StubModule:

        def __init__(self, name: str):
            self.name = name
            self.ports: tuple = ()

        def as_operand(self) -> str:
            return self.name

    dummy_module = _StubModule("dummy")
    metadata = ModuleMetadata(dummy_module, registry)

    metadata.exposures.record_value(value_expr)
    finish_expr = SimpleNamespace(opcode="FINISH", meta_cond=None)
    metadata.record_finish(finish_expr)

    fifo_port = Port(UInt(8))
    fifo_port.name = "fifo0"

    fifo_port.module = dummy_module

    push_expr = FIFOPush(fifo_port, UInt(8)(0))
    push_expr.parent = dummy_module
    registry.record_push(dummy_module, push_expr, None)
    metadata.record_fifo_interaction(fifo_port, push_expr)

    metadata.freeze()
    registry.freeze()

    assert metadata.exposures.values is metadata.exposures.values
    assert metadata.finish_sites is metadata.finish_sites
    assert isinstance(metadata.finish_sites, tuple)
    assert metadata.fifo.pushes is metadata.fifo.pushes
    channel = registry.metadata_for(fifo_port)
    assert channel.pushes is channel.pushes

    with pytest.raises(RuntimeError):
        metadata.exposures.record_value(SimpleNamespace(meta_cond=None))
    with pytest.raises(RuntimeError):
        metadata.record_finish(SimpleNamespace(opcode="FINISH", meta_cond=None))
    with pytest.raises(RuntimeError):
        metadata.record_fifo_interaction(fifo_port, push_expr)
    with pytest.raises(RuntimeError):
        registry.record_push(dummy_module, FIFOPush(fifo_port, UInt(8)(0)), None)
