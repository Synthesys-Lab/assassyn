"""Ensure FIFO metadata captures module, expressions, and predicates."""

import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (
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

    dumper = CIRCTDumper()
    dumper.visit_system(sysb)

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
    pushes_by_port = metadata.fifo_by_port[out_port]
    pops_by_port = metadata.fifo_by_port[in_port]
    assert pushes_by_port is fifo_registry.metadata_for(out_port)
    assert pops_by_port is fifo_registry.metadata_for(in_port)
    assert pushes_by_port.pushes == [push_entry]
    assert pops_by_port.pops == [pop_entry]

    # Backwards compatibility accessors still expose expression lists
    assert [entry.expr for entry in fifo_meta.pushes] == metadata.pushes
    assert [entry.expr for entry in fifo_meta.pops] == metadata.pops

    # Revisit the module in isolation to ensure FIFO operations skip the expose map
    isolated_dumper = CIRCTDumper()
    isolated_dumper.sys = sysb
    isolated_dumper.visit_module(pipe_module)
    isolated_registry = isolated_dumper.fifo_registry
    isolated_module_md = isolated_dumper.module_metadata[pipe_module]
    assert len(isolated_registry.metadata_for(out_port).pushes) == 1
    assert len(isolated_registry.metadata_for(in_port).pops) == 1
    # Re-run module generation to confirm metadata clears stale entries
    isolated_dumper.visit_module(pipe_module)
    assert len(isolated_registry.metadata_for(out_port).pushes) == 1
    assert len(isolated_registry.metadata_for(in_port).pops) == 1
    isolated_module_md = isolated_dumper.module_metadata[pipe_module]
    assert isolated_module_md.fifo.pushes[0] is isolated_registry.metadata_for(out_port).pushes[0]

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

    dumper = CIRCTDumper()
    dumper.visit_system(sysb)

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
