"""Tests for Array ownership metadata and integrations."""

from dataclasses import FrozenInstanceError

import pytest

from assassyn.builder import SysBuilder
from assassyn.codegen.verilog.design import CIRCTDumper
from assassyn.ir.array import RegArray, RegisterOwner, MemoryOwner
from assassyn.ir.dtype import UInt
from assassyn.ir.memory.sram import SRAM
from assassyn.ir.memory.dram import DRAM
from assassyn.ir.module import Module, combinational


def test_regarray_defaults_to_register_owner_without_module():
    """Top-level RegArray instances should default to RegisterOwner with no module."""

    sys = SysBuilder("array_owner_default")
    with sys:
        arr = RegArray(UInt(8), 4, name="kind_arr")

    assert isinstance(arr.owner, RegisterOwner)
    assert arr.owner.module is None


def test_regarray_owner_records_defining_module():
    """Arrays created inside a module inherit a RegisterOwner that points to the module."""

    class Holder(Module):
        def __init__(self):
            super().__init__(ports={})
            self.storage = None

        @combinational
        def build(self):
            self.storage = RegArray(UInt(16), 2, name="module_reg")

    sys = SysBuilder("array_owner_module")
    with sys:
        holder = Holder()
        holder.build()

    arr = holder.storage
    assert isinstance(arr.owner, RegisterOwner)
    assert arr.owner.module is holder


def test_memory_payload_and_aux_buffers_have_memory_owner():
    """Memory modules tag payload and auxiliary buffers with MemoryOwner descriptors."""

    sys = SysBuilder("array_owner_memory")
    with sys:
        sram = SRAM(16, 32, None)
        dram = DRAM(16, 32, None)

    payload = sram._payload  # pylint: disable=protected-access
    dout = sram.dout
    dram_payload = dram._payload  # pylint: disable=protected-access

    assert isinstance(payload.owner, MemoryOwner)
    assert payload.owner.memory is sram
    assert payload.owner.role == "payload"

    assert isinstance(dout.owner, MemoryOwner)
    assert dout.owner.memory is sram
    assert dout.owner.role == "dout"

    assert isinstance(dram_payload.owner, MemoryOwner)
    assert dram_payload.owner.memory is dram
    assert dram_payload.owner.role == "payload"


def test_metadata_registry_skips_memory_payloads():
    """ArrayMetadataRegistry should ignore memory payloads while keeping regular arrays."""

    class Reader(Module):
        def __init__(self):
            super().__init__(ports={})

        @combinational
        def build(self, arr):
            _ = arr[0]

    sys = SysBuilder("array_owner_metadata")
    with sys:
        reg_arr = RegArray(UInt(8), 4, name="regular_arr")
        sram = SRAM(8, 16, None)

        reader = Reader()
        reader.build(reg_arr)

    dumper = CIRCTDumper()
    dumper.array_metadata.collect(dumper, sys)

    assert dumper.array_metadata.metadata_for(reg_arr) is not None
    assert dumper.array_metadata.metadata_for(sram._payload) is None  # pylint: disable=protected-access


def test_owner_descriptors_are_immutable():
    """Frozen dataclasses ensure ownership descriptors cannot be mutated in place."""

    sys = SysBuilder("array_owner_immutability")
    with sys:
        arr = RegArray(UInt(4), 2)

    with pytest.raises(FrozenInstanceError):
        arr.owner.module = None


def test_assign_owner_enforces_descriptor_type():
    """assign_owner should swap descriptors and reject invalid inputs."""

    sys = SysBuilder("array_owner_assign")
    with sys:
        arr = RegArray(UInt(4), 2)

    replacement = RegisterOwner(module=None)
    arr.assign_owner(replacement)
    assert arr.owner is replacement

    with pytest.raises(TypeError):
        arr.assign_owner("memory")  # type: ignore[arg-type]

