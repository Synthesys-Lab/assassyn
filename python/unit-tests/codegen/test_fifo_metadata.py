"""Ensure FIFO metadata captures module, expressions, and predicates."""

import os
import sys

import pytest

sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (  # type: ignore
    Module,
    SysBuilder,
    UInt,
    Port,
    module,
    Bits,
    push_condition,
    pop_condition,
)
from assassyn.codegen.verilog.design import CIRCTDumper
from assassyn.codegen.verilog.fifo_analysis import collect_fifo_metadata
from assassyn.ir.module import Port as IRPort


def test_fifo_metadata_records_predicates():
    sysb = SysBuilder("fifo_md")
    with sysb:

        class Pipe(Module):

            def __init__(self):
                super().__init__(ports={
                    'in0': Port(UInt(8)),
                    'out0': Port(UInt(8)),
                })

            @module.combinational
            def build(self):
                push_condition(Bits(1)(1))
                data = self.in0.pop()
                push_condition(Bits(1)(0))
                self.out0.push(data)
                pop_condition()
                pop_condition()

        Pipe().build()

    module_metadata, fifo_registry = collect_fifo_metadata(sysb)
    dumper = CIRCTDumper(
        module_metadata=module_metadata,
        fifo_registry=fifo_registry,
    )

    pipe_module = sysb.modules[0]
    metadata = dumper.module_metadata[pipe_module]
    fifo_meta = metadata.fifo

    assert len(fifo_meta.pops) == 1
    pop_entry = fifo_meta.pops[0]
    assert pop_entry.module is pipe_module
    assert pop_entry.predicate == "(Bits(1)(1))"

    assert len(fifo_meta.pushes) == 1
    push_entry = fifo_meta.pushes[0]
    assert push_entry.module is pipe_module
    assert push_entry.predicate == "(Bits(1)(1)) & (Bits(1)(0))"

    # FIFO registry mirrors per-module metadata
    fifo_registry = dumper.fifo_registry
    in_port = pipe_module.ports[0]
    out_port = pipe_module.ports[1]
    assert fifo_registry.metadata_for(out_port).pushes == [push_entry]
    assert fifo_registry.metadata_for(in_port).pops == [pop_entry]
    channel_view = list(metadata.fifo.iter_channels())
    assert {port for port, _, _ in channel_view} == {in_port, out_port}
    fifo_ports = list(metadata.fifo.ports)
    assert out_port in fifo_ports
    assert in_port in fifo_ports
    assert metadata.fifo.interactions_for(out_port) == [push_entry]
    assert metadata.fifo.interactions_for(in_port) == [pop_entry]
    for port, fifo_metadata, interactions in channel_view:
        assert fifo_metadata is fifo_registry.metadata_for(port)
        assert list(interactions) == list(metadata.fifo.interactions_for(port))

    # Backwards compatibility accessors still expose expression lists
    assert [entry.expr for entry in fifo_meta.pushes] == metadata.pushes
    assert [entry.expr for entry in fifo_meta.pops] == metadata.pops

    # Revisit the module in isolation to ensure FIFO operations skip the expose map
    isolated_metadata, isolated_registry = collect_fifo_metadata(sysb, modules=[pipe_module])
    isolated_dumper = CIRCTDumper(
        module_metadata=isolated_metadata,
        fifo_registry=isolated_registry,
    )
    isolated_dumper.sys = sysb
    isolated_dumper.visit_module(pipe_module)
    isolated_module_md = isolated_dumper.module_metadata[pipe_module]
    assert len(isolated_registry.metadata_for(out_port).pushes) == 1
    assert len(isolated_registry.metadata_for(in_port).pops) == 1
    assert [
        port for port, _, _ in isolated_module_md.fifo.iter_channels()
    ] == list(isolated_module_md.fifo.ports)
    # Re-run module generation to confirm metadata clears stale entries
    isolated_dumper.visit_module(pipe_module)
    assert len(isolated_registry.metadata_for(out_port).pushes) == 1
    assert len(isolated_registry.metadata_for(in_port).pops) == 1
    isolated_module_md = isolated_dumper.module_metadata[pipe_module]
    assert isolated_module_md.fifo.pushes[0] is isolated_registry.metadata_for(out_port).pushes[0]
    assert len(list(isolated_module_md.fifo.iter_channels())) == 2

    fifo_expose_keys = [key for key in isolated_dumper._exposes if isinstance(key, IRPort)]
    assert fifo_expose_keys == []


def test_fifo_registry_cross_module_sharing():
    sysb = SysBuilder("fifo_registry_cross")

    with sysb:

        class Consumer(Module):

            def __init__(self):
                super().__init__(ports={
                    'data_in': Port(UInt(8)),
                })

            @module.combinational
            def build(self):
                push_condition(Bits(1)(1))
                self.data_in.pop()
                pop_condition()

        class Producer(Module):

            def __init__(self):
                super().__init__(ports={
                    'data_out': Port(UInt(8)),
                })

            @module.combinational
            def build(self, consumer: Consumer):
                push_condition(Bits(1)(1))
                consumer.data_in.push(UInt(8)(7))
                pop_condition()

        consumer = Consumer()
        producer = Producer()
        consumer.build()
        producer.build(consumer)

    module_metadata, fifo_registry = collect_fifo_metadata(sysb)
    dumper = CIRCTDumper(
        module_metadata=module_metadata,
        fifo_registry=fifo_registry,
    )

    consumer_module = consumer
    producer_module = producer
    consumer_port = consumer_module.ports[0]

    consumer_md = dumper.module_metadata[consumer_module]
    producer_md = dumper.module_metadata[producer_module]
    fifo_registry = dumper.fifo_registry
    fifo_meta = fifo_registry.metadata_for(consumer_port)

    assert len(fifo_meta.pushes) == 1
    assert len(fifo_meta.pops) == 1
    assert fifo_meta.pushes[0].module is producer_module
    assert fifo_meta.pops[0].module is consumer_module

    # Module metadata still exposes aggregated views
    assert producer_md.fifo.pushes[0] is fifo_meta.pushes[0]
    assert consumer_md.fifo.pops[0] is fifo_meta.pops[0]
    producer_ports = list(producer_md.fifo.ports)
    consumer_ports = list(consumer_md.fifo.ports)
    assert producer_ports == [consumer_port]
    assert consumer_ports == [consumer_port]
    for port, fifo_metadata, interactions in producer_md.fifo.iter_channels():
        assert port is consumer_port
        assert fifo_metadata is fifo_meta
        assert list(interactions) == producer_md.fifo.interactions_for(port)


def test_fifo_analysis_single_module_refresh():
    sysb = SysBuilder("fifo_prepass_incremental")

    with sysb:

        class Pipe(Module):

            def __init__(self):
                super().__init__(ports={
                    'in0': Port(UInt(8)),
                    'out0': Port(UInt(8)),
                })

            @module.combinational
            def build(self):
                push_condition(Bits(1)(1))
                data = self.in0.pop()
                push_condition(Bits(1)(0))
                self.out0.push(data)
                pop_condition()
                pop_condition()

        Pipe().build()

    pipe_module = sysb.modules[0]
    in_port = pipe_module.ports[0]
    out_port = pipe_module.ports[1]

    base_metadata, base_registry = collect_fifo_metadata(sysb)
    fifo_meta = base_registry.metadata_for(out_port)
    assert len(fifo_meta.pushes) == 1
    fifo_meta = base_registry.metadata_for(in_port)
    assert len(fifo_meta.pops) == 1

    # Re-run analysis for the pipe module only; metadata should stay consistent and
    # independent from the base snapshot.
    partial_metadata, partial_registry = collect_fifo_metadata(sysb, modules=[pipe_module])
    fifo_meta_out = partial_registry.metadata_for(out_port)
    fifo_meta_in = partial_registry.metadata_for(in_port)
    assert len(fifo_meta_out.pushes) == 1
    assert len(fifo_meta_in.pops) == 1
    assert fifo_meta_out.pushes[0].predicate == "(Bits(1)(1)) & (Bits(1)(0))"

    # Visiting the module with only the refreshed metadata should succeed without
    # mutating the registry snapshot.
    isolated_dumper = CIRCTDumper(
        module_metadata=partial_metadata,
        fifo_registry=partial_registry,
    )
    isolated_dumper.sys = sysb
    isolated_dumper.visit_module(pipe_module)


def test_circtdumper_requires_fifo_metadata():
    sysb = SysBuilder("fifo_requires_metadata")

    with sysb:

        class Pipe(Module):

            def __init__(self):
                super().__init__(ports={
                    'in0': Port(UInt(8)),
                    'out0': Port(UInt(8)),
                })

            @module.combinational
            def build(self):
                data = self.in0.pop()
                self.out0.push(data)

        Pipe().build()

    dumper = CIRCTDumper()
    pipe_module = sysb.modules[0]

    with pytest.raises(RuntimeError, match="FIFO metadata"):
        dumper.visit_module(pipe_module)
