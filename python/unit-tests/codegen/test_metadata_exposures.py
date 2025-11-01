"""Verify that metadata pre-pass records exposures and flags."""

import os
import sys
from types import SimpleNamespace
import importlib

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
    InteractionKind,
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

    module_metadata, interactions = collect_fifo_metadata(sys_builder)
    dumper_metadata = module_metadata[instance]
    module_view = dumper_metadata.interactions

    finish_sites = dumper_metadata.finish_sites
    assert finish_sites, "expected finish sites to be recorded"
    assert isinstance(finish_sites, tuple)
    assert all(site.opcode == Intrinsic.FINISH for site in finish_sites)
    assert all(site.meta_cond is not None for site in finish_sites)
    assert finish_sites[0].meta_cond is instance.finish_cond

    assert dumper_metadata.calls, "expected async calls to be recorded"
    assert isinstance(dumper_metadata.calls, tuple)
    call = dumper_metadata.calls[0]
    assert isinstance(call, AsyncCall)

    array_resource = next(iter(module_view.writes))
    writes = module_view.writes.get(array_resource, ())
    assert writes, "expected array writes to be recorded"
    assert isinstance(writes, tuple)
    assert all(hasattr(write, "meta_cond") for write in writes)
    assert module_view.writes[array_resource] is writes
    assert tuple(module_view.writes.keys()) == (array_resource,)
    assert tuple(module_view.reads.keys()) == ()

    array_view = interactions.array_view(array_resource)
    writers = array_view.writers
    assert instance in writers
    assert writers[instance] == writes
    assert array_view.reads == ()
    assert array_view.reads_by_module.get(instance, ()) == ()

    async_groups = interactions.async_ledger.calls_for_module(instance)
    assert async_groups, "expected async trigger exposure metadata"
    callee = call.bind.callee
    assert callee in async_groups
    callee_calls = async_groups[callee]
    assert isinstance(callee_calls, tuple)
    assert callee_calls and callee_calls[0] is call

    value_exposures = dumper_metadata.value_exposures
    assert isinstance(value_exposures, tuple)
    assert value_exposures is dumper_metadata.value_exposures
    for expr in value_exposures:
        assert getattr(expr, "meta_cond", None) is not None

    metadata_pushes = module_view.pushes
    metadata_pops = module_view.pops
    assert metadata_pushes, "FIFO push metadata should be recorded"
    assert metadata_pops, "FIFO pop metadata should be recorded"
    assert isinstance(metadata_pushes, tuple)
    assert isinstance(metadata_pops, tuple)
    for expr in metadata_pushes + metadata_pops:
        assert getattr(expr, "meta_cond", None) is not None

    fifo_ports = module_view.fifo_ports
    assert fifo_ports
    for fifo_port in fifo_ports:
        fifo_bucket = interactions.fifo_view(fifo_port)
        interactions_for_port = module_view.fifo_map[fifo_port]
        assert isinstance(interactions_for_port, tuple)
        for expr in interactions_for_port:
            assert any(expr is candidate for candidate in metadata_pushes + metadata_pops)

    for fifo_port in fifo_ports:
        fifo_view = interactions.fifo_view(fifo_port)
        pushes = fifo_view.pushes
        pops = fifo_view.pops
        if pushes:
            assert pushes == tuple(expr for expr in metadata_pushes if expr.fifo is fifo_port)
        if pops:
            assert pops == tuple(expr for expr in metadata_pops if expr.fifo is fifo_port)

    assert interactions.module_view(instance) is module_view


def test_metadata_freeze_stabilizes_views():
    """Document why metadata stays mutable until frozen."""

    sysb = SysBuilder("metadata_freeze_views")
    with sysb:

        class FreezeModule(Module):

            def __init__(self):
                super().__init__(ports={
                    'in0': Port(UInt(8)),
                    'out0': Port(UInt(8)),
                })

            @module.combinational
            def build(self):
                data = self.in0.pop()
                self.out0.push(data)
                finish()

        instance = FreezeModule()
        instance.build()

    module_metadata, interactions = collect_fifo_metadata(sysb)
    metadata = module_metadata[instance]
    module_view = metadata.interactions

    pushes = module_view.pushes
    pops = module_view.pops
    finish_sites = metadata.finish_sites
    assert finish_sites is metadata.finish_sites
    assert isinstance(finish_sites, tuple)
    assert isinstance(metadata.value_exposures, tuple)
    assert metadata.value_exposures is metadata.value_exposures
    for port in module_view.fifo_ports:
        fifo_view = interactions.fifo_view(port)
        interactions_for_port = module_view.fifo_map[port]
        assert fifo_view.pushes is fifo_view.pushes
        assert fifo_view.pops is fifo_view.pops
        sample_interaction = interactions_for_port[0]
        with pytest.raises(RuntimeError):
            interactions.record(
                module=instance,
                resource=port,
                kind=InteractionKind.FIFO_PUSH if isinstance(sample_interaction, FIFOPush) else InteractionKind.FIFO_POP,
                expr=sample_interaction,
            )

    # The shared matrix returns the same view object for the module.
    assert interactions.module_view(instance) is module_view


def test_metadata_package_reexports():
    """Ensure the verilog metadata package re-exports the split submodules."""

    pkg = importlib.import_module("assassyn.codegen.verilog.metadata")

    core = importlib.import_module("assassyn.codegen.verilog.metadata.core")
    array_pkg = importlib.import_module("assassyn.codegen.verilog.metadata.array")
    module_pkg = importlib.import_module("assassyn.codegen.verilog.metadata.module")
    fifo_pkg = importlib.import_module("assassyn.codegen.verilog.metadata.fifo")

    assert pkg.InteractionKind is core.InteractionKind
    assert pkg.InteractionMatrix is core.InteractionMatrix
    assert pkg.AsyncLedger is core.AsyncLedger

    assert pkg.ModuleBundle is module_pkg.ModuleBundle
    assert pkg.ModuleInteractionView is module_pkg.ModuleInteractionView
    assert pkg.ModuleMetadata is module_pkg.ModuleMetadata

    assert pkg.ArrayInteractionView is array_pkg.ArrayInteractionView
    assert pkg.ArrayMetadata is array_pkg.ArrayMetadata

    assert pkg.FIFOInteractionView is fifo_pkg.FIFOInteractionView

    external_pkg = importlib.import_module("assassyn.codegen.verilog.metadata.external")
    assert pkg.ExternalRegistry is external_pkg.ExternalRegistry
    assert pkg.ExternalRead is external_pkg.ExternalRead

    # The package should expose the canonical public surface via __all__.
    expected = {
        "InteractionKind",
        "InteractionMatrix",
        "AsyncLedger",
        "ModuleBundle",
        "ModuleInteractionView",
        "ModuleMetadata",
        "ArrayInteractionView",
        "ArrayMetadata",
        "FIFOInteractionView",
        "ExternalRegistry",
        "ExternalRead",
    }
    assert expected.issubset(set(pkg.__all__)), "__all__ missing expected metadata exports"
