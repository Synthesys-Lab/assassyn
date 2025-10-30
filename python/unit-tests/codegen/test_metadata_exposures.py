"""Verify that metadata pre-pass records exposures and flags."""

import os
import sys

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
from assassyn.codegen.verilog.fifo_analysis import collect_fifo_metadata  # type: ignore
from assassyn.ir.expr.call import AsyncCall  # type: ignore


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

    assert dumper_metadata.has_finish is True
    assert len(dumper_metadata.calls) == 1
    assert isinstance(dumper_metadata.calls[0], AsyncCall)

    array_exposures = dumper_metadata.exposures.arrays
    assert array_exposures, "expected array exposures to be recorded"
    for exposure in array_exposures.values():
        assert exposure.writes_by_module, "array exposures should capture writes per module"
        for writer, writes in exposure.writes_by_module.items():
            assert writer is instance
            assert writes, "expected recorded array writes"
        # Reads are captured when gatherable; the current test exercises writes primarily.

    async_triggers = dumper_metadata.exposures.async_triggers
    assert async_triggers, "expected async trigger exposure metadata"
    for entries in async_triggers.values():
        assert entries, "async trigger entries should not be empty"
        for entry in entries:
            assert entry.call in dumper_metadata.calls

    metadata_pushes = dumper_metadata.fifo.pushes
    assert metadata_pushes, "FIFO push metadata should be recorded"
    metadata_pops = dumper_metadata.fifo.pops
    assert metadata_pops, "FIFO pop metadata should be recorded"
    for interaction in metadata_pushes + metadata_pops:
        assert interaction.predicate is not None

    # Verify FIFO registry mirrors module metadata
    fifo_ports = {entry.expr.fifo for entry in metadata_pushes + metadata_pops}
    for fifo_port in fifo_ports:
        channel = fifo_registry.metadata_for(fifo_port)
        assert channel.pushes or channel.pops
