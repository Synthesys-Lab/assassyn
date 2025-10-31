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
from assassyn.codegen.verilog.analysis import collect_fifo_metadata
from assassyn.ir.const import Const
from assassyn.ir.expr import BinaryOp
from assassyn.ir.expr.expr import Operand
from assassyn.ir.expr.call import FIFOPush
from assassyn.ir.expr.expr import FIFOPop


def unwrap_value(node):
    """Return the underlying IR value when *node* is an Operand."""
    return node.value if isinstance(node, Operand) else node


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

    module_metadata, interactions = collect_fifo_metadata(sysb)
    dumper = CIRCTDumper(
        module_metadata=module_metadata,
        interactions=interactions,
    )

    pipe_module = sysb.modules[0]
    metadata = dumper.module_metadata[pipe_module]
    module_view = metadata.interactions

    pops = module_view.pops
    assert len(pops) == 1
    pop_expr = pops[0]
    assert isinstance(pop_expr, FIFOPop)
    assert pop_expr.parent is pipe_module
    pop_predicate = unwrap_value(pop_expr.meta_cond)
    assert isinstance(pop_predicate, Const)
    assert pop_predicate.value == 1

    pushes = module_view.pushes
    assert len(pushes) == 1
    push_expr = pushes[0]
    assert isinstance(push_expr, FIFOPush)
    assert push_expr.parent is pipe_module
    push_predicate = unwrap_value(push_expr.meta_cond)
    assert isinstance(push_predicate, BinaryOp)
    assert push_predicate.opcode == BinaryOp.BITWISE_AND
    lhs = unwrap_value(push_predicate.lhs)
    rhs = unwrap_value(push_predicate.rhs)
    assert isinstance(lhs, Const)
    assert lhs.value == 1
    assert isinstance(rhs, Const)
    assert rhs.value == 0

    # FIFO registry mirrors per-module metadata
    interactions = dumper.interactions
    in_port = pipe_module.ports[0]
    out_port = pipe_module.ports[1]
    assert interactions.fifo_view(out_port).pushes == (push_expr,)
    assert interactions.fifo_view(in_port).pops == (pop_expr,)
    fifo_ports = module_view.fifo_ports
    assert {port for port in fifo_ports} == {in_port, out_port}
    for port in fifo_ports:
        fifo_metadata = dumper.interactions.fifo_view(port)
        interactions_for_port = module_view.fifo_map[port]
        assert fifo_metadata.pushes == tuple(expr for expr in interactions_for_port if isinstance(expr, FIFOPush))
        assert fifo_metadata.pops == tuple(expr for expr in interactions_for_port if isinstance(expr, FIFOPop))
        assert all(isinstance(expr, (FIFOPush, FIFOPop)) for expr in interactions_for_port)

    # Revisit the module in isolation to ensure FIFO operations skip the expose map
    isolated_metadata, isolated_interactions = collect_fifo_metadata(sysb, modules=[pipe_module])
    isolated_dumper = CIRCTDumper(
        module_metadata=isolated_metadata,
        interactions=isolated_interactions,
    )
    isolated_dumper.sys = sysb
    isolated_dumper.visit_module(pipe_module)
    isolated_module_md = isolated_dumper.module_metadata[pipe_module]
    assert len(isolated_dumper.interactions.fifo_view(out_port).pushes) == 1
    assert len(isolated_dumper.interactions.fifo_view(in_port).pops) == 1
    assert list(isolated_module_md.interactions.fifo_ports)
    # Re-run module generation to confirm metadata clears stale entries
    isolated_dumper.visit_module(pipe_module)
    assert len(isolated_dumper.interactions.fifo_view(out_port).pushes) == 1
    assert len(isolated_dumper.interactions.fifo_view(in_port).pops) == 1
    isolated_module_md = isolated_dumper.module_metadata[pipe_module]
    assert (
        isolated_module_md.interactions.pushes[0]
        is isolated_dumper.interactions.fifo_view(out_port).pushes[0]
    )
    assert len(isolated_module_md.interactions.fifo_ports) == 2

    assert not hasattr(isolated_dumper, '_exposes')


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

    module_metadata, interactions = collect_fifo_metadata(sysb)
    dumper = CIRCTDumper(
        module_metadata=module_metadata,
        interactions=interactions,
    )

    consumer_module = consumer
    producer_module = producer
    consumer_port = consumer_module.ports[0]

    consumer_md = dumper.module_metadata[consumer_module]
    producer_md = dumper.module_metadata[producer_module]
    fifo_view = dumper.interactions.fifo_view(consumer_port)

    assert fifo_view.pushes
    assert fifo_view.pops
    assert isinstance(fifo_view.pushes[0], FIFOPush)
    assert isinstance(fifo_view.pops[0], FIFOPop)
    assert fifo_view.pushes[0].parent is producer_module
    assert fifo_view.pops[0].parent is consumer_module

    # Module metadata still exposes aggregated views
    assert producer_md.interactions.pushes[0] is fifo_view.pushes[0]
    assert consumer_md.interactions.pops[0] is fifo_view.pops[0]
    producer_ports = list(producer_md.interactions.fifo_ports)
    consumer_ports = list(consumer_md.interactions.fifo_ports)
    assert producer_ports == [consumer_port]
    assert consumer_ports == [consumer_port]
    per_module = producer_md.interactions.fifo_map[consumer_port]
    assert tuple(per_module) == fifo_view.pushes


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

    base_metadata, base_interactions = collect_fifo_metadata(sysb)
    assert len(base_interactions.fifo_view(out_port).pushes) == 1
    assert len(base_interactions.fifo_view(in_port).pops) == 1

    # Re-run analysis for the pipe module only; metadata should stay consistent and
    # independent from the base snapshot.
    partial_metadata, partial_interactions = collect_fifo_metadata(sysb, modules=[pipe_module])
    fifo_meta_out = partial_interactions.fifo_view(out_port)
    fifo_meta_in = partial_interactions.fifo_view(in_port)
    assert len(fifo_meta_out.pushes) == 1
    assert len(fifo_meta_in.pops) == 1
    push_predicate = unwrap_value(fifo_meta_out.pushes[0].meta_cond)
    assert isinstance(push_predicate, BinaryOp)
    assert push_predicate.opcode == BinaryOp.BITWISE_AND
    lhs = unwrap_value(push_predicate.lhs)
    rhs = unwrap_value(push_predicate.rhs)
    assert isinstance(lhs, Const)
    assert lhs.value == 1
    assert isinstance(rhs, Const)
    assert rhs.value == 0

    # Visiting the module with only the refreshed metadata should succeed without
    # mutating the registry snapshot.
    isolated_dumper = CIRCTDumper(
        module_metadata=partial_metadata,
        interactions=partial_interactions,
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
