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
from assassyn.codegen.verilog.analysis import collect_fifo_metadata
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

    module_metadata, interactions = collect_fifo_metadata(sysb, modules=[pipe_module])
    dumper = CIRCTDumper(module_metadata=module_metadata, interactions=interactions)
    dumper.sys = sysb
    dumper.visit_module(pipe_module)

    module_entry = dumper.module_metadata[pipe_module]
    fifo_meta = module_entry.interactions
    assert len(fifo_meta.pushes) == 1
    assert len(fifo_meta.pops) == 1

    assert not hasattr(dumper, '_exposes')

    in_port = pipe_module.ports[0]
    out_port = pipe_module.ports[1]
    assert dumper.interactions.fifo_view(out_port).pushes == fifo_meta.pushes
    assert dumper.interactions.fifo_view(in_port).pops == fifo_meta.pops

    code = "\n".join(dumper.code)
    module_prefix = namify(pipe_module.name)
    assert "reduce(operator.or_, [" in code
    push_valid_pattern = (
        rf"self\.{module_prefix}_out0_push_valid = executed_wire & "
        rf"\(.+\) & self\.fifo_{module_prefix}_out0_push_ready"
    )
    assert re.search(push_valid_pattern, code)
    assert re.search(rf"self\.{module_prefix}_out0_push_data = ", code)
    assert re.search(r"self\.in0_pop_ready = executed_wire & \(.+\)", code)
