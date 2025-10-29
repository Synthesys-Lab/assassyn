"""Verify cleanup wiring for FIFO operations relies on metadata, not exposes."""

import os
import re
import sys

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
from assassyn.ir.module import Port as IRPort
from assassyn.utils import namify


def test_fifo_cleanup_metadata_drives_handshakes():
    sysb = SysBuilder("fifo_cleanup_md")
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
                self.out0.push(data)
                pop_condition()

        Pipe().build()

    pipe_module = sysb.modules[0]

    dumper = CIRCTDumper()
    dumper.sys = sysb
    dumper.visit_module(pipe_module)

    fifo_meta = dumper.module_metadata[pipe_module].fifo
    assert len(fifo_meta.pushes) == 1
    assert len(fifo_meta.pops) == 1

    fifo_expose_keys = [key for key in dumper._exposes if isinstance(key, IRPort)]
    assert fifo_expose_keys == []

    code = "\n".join(dumper.code)
    module_prefix = namify(pipe_module.name)
    assert "reduce(or_, [" in code
    push_valid_pattern = (
        rf"self\.{module_prefix}_out0_push_valid = executed_wire & "
        rf"\(.+\) & self\.fifo_{module_prefix}_out0_push_ready"
    )
    assert re.search(push_valid_pattern, code)
    assert re.search(rf"self\.{module_prefix}_out0_push_data = ", code)
    assert re.search(r"self\.in0_pop_ready = executed_wire & \(.+\)", code)
