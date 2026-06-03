"""Tests for shared Verilog schedule equations used by validation."""

from assassyn.frontend import Int, Module, Port, SysBuilder, module
from assassyn.codegen.verilog.analysis import collect_fifo_metadata
from assassyn.codegen.verilog.schedule import (  # type: ignore
    compute_fifo_depths,
    compute_trigger_widths,
    group_async_triggers,
    group_fifo_pushes,
)


class Target(Module):
    """Target module used to check FIFO depth and trigger width equations."""

    def __init__(self):
        super().__init__(ports={"data": Port(Int(32))})

    @module.combinational
    def build(self):
        self.pop_all_ports(True)


class Source(Module):
    """Source module with one async call."""

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, target: Target):
        call = target.async_called(data=Int(32)(3))
        call.bind.set_fifo_depth(data=3)


def test_schedule_helpers_match_backend_depth_and_trigger_rules():
    """Depth and trigger helpers expose the same equations used by Top."""

    sysb = SysBuilder("schedule_equations")
    with sysb:
        target = Target()
        target.build()
        source = Source()
        source.build(target)

    module_metadata, interactions = collect_fifo_metadata(sysb)

    fifo_depths = compute_fifo_depths(sysb, module_metadata, default_fifo_depth=1)
    trigger_widths = compute_trigger_widths(sysb, fifo_depths, default_fifo_depth=1)
    fifo_pushes = group_fifo_pushes(module_metadata[source].interactions.pushes)
    async_triggers = group_async_triggers(interactions.async_ledger, source)

    assert fifo_depths[target][target.ports[0]] == 3
    assert trigger_widths[target] == 4
    assert (target, target.ports[0]) in fifo_pushes
    assert target in async_triggers
