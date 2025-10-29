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

    # Backwards compatibility accessors still expose expression lists
    assert [entry.expr for entry in fifo_meta.pushes] == metadata.pushes
    assert [entry.expr for entry in fifo_meta.pops] == metadata.pops

    # Revisit the module in isolation to ensure FIFO operations skip the expose map
    isolated_dumper = CIRCTDumper()
    isolated_dumper.sys = sysb
    isolated_dumper.visit_module(pipe_module)

    fifo_expose_keys = [key for key in isolated_dumper._exposes if isinstance(key, IRPort)]
    assert fifo_expose_keys == []
