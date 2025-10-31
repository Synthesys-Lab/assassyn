"""Tests for Array ownership metadata and integrations."""

import pytest

from assassyn.builder import SysBuilder
from assassyn.codegen.verilog.array import ArrayMetadataRegistry
from assassyn.codegen.verilog.design import CIRCTDumper
from assassyn.ir.array import RegArray
from assassyn.ir.dtype import UInt
from assassyn.ir.memory.sram import SRAM
from assassyn.ir.memory.dram import DRAM
from assassyn.ir.module import Module, combinational


def test_regarray_defaults_to_owner_none_outside_module():
    """Top-level RegArray instances should default to owner=None."""

    sys = SysBuilder("array_owner_default")
    with sys:
        arr = RegArray(UInt(8), 4, name="kind_arr")

    assert arr.owner is None


def test_regarray_owner_records_defining_module():
    """Arrays created inside a module record that module as the owner."""

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
    assert arr.owner is holder


def test_memory_payload_and_aux_buffers_reference_memory_instance():
    """Memory modules should record themselves as owners for internal arrays."""

    sys = SysBuilder("array_owner_memory")
    with sys:
        sram = SRAM(16, 32, None)
        dram = DRAM(16, 32, None)

    payload = sram._payload  # pylint: disable=protected-access
    dout = sram.dout
    dram_payload = dram._payload  # pylint: disable=protected-access

    assert payload.owner is sram
    assert payload is sram._payload  # pylint: disable=protected-access

    assert dout.owner is sram
    assert dram_payload.owner is dram


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
    dumper.array_metadata.collect(sys)

    assert dumper.array_metadata.metadata_for(reg_arr) is not None
    assert dumper.array_metadata.metadata_for(sram._payload) is None  # pylint: disable=protected-access

    standalone_registry = ArrayMetadataRegistry()
    standalone_registry.collect(sys)
    assert standalone_registry.metadata_for(reg_arr) is not None
    assert standalone_registry.metadata_for(sram._payload) is None  # pylint: disable=protected-access


def test_assign_owner_validates_types():
    """assign_owner should accept modules/memories/None and reject other objects."""

    class Holder(Module):
        def __init__(self):
            super().__init__(ports={})
            self.storage = None

        @combinational
        def build(self):
            self.storage = RegArray(UInt(4), 2)

    sys = SysBuilder("array_owner_assign")
    with sys:
        holder = Holder()
        holder.build()

    arr = holder.storage
    assert arr.owner is holder

    arr.assign_owner(None)
    assert arr.owner is None

    arr.assign_owner(holder)
    assert arr.owner is holder

    with pytest.raises(TypeError):
        arr.assign_owner("memory")  # type: ignore[arg-type]


def test_is_payload_helper_detects_memory_payloads():
    """Array.is_payload should identify payload buffers and reject non-payload arrays."""

    sys = SysBuilder("array_is_payload")
    with sys:
        sram = SRAM(32, 64, None)
        dram = DRAM(32, 64, None)
        reg_arr = RegArray(UInt(8), 2, name="plain")

    sram_payload = sram._payload  # pylint: disable=protected-access
    dram_payload = dram._payload  # pylint: disable=protected-access

    assert sram_payload.is_payload(SRAM) is True
    assert dram_payload.is_payload(DRAM) is True

    assert sram_payload.is_payload(DRAM) is False
    assert sram.dout.is_payload(SRAM) is False
    assert reg_arr.is_payload(SRAM) is False
    assert reg_arr.is_payload(DRAM) is False


def test_is_payload_rejects_non_memory_arguments():
    """Array.is_payload should raise TypeError when the argument is not memory-related."""

    sys = SysBuilder("array_is_payload_invalid")
    with sys:
        reg_arr = RegArray(UInt(8), 2, name="plain")

    with pytest.raises(TypeError) as excinfo_cls:
        reg_arr.is_payload(int)
    assert (
        str(excinfo_cls.value)
        == "Array.is_payload expects a MemoryBase subclass or instance; got <class 'int'>"
    )

    with pytest.raises(TypeError) as excinfo_inst:
        reg_arr.is_payload("memory")  # type: ignore[arg-type]
    assert (
        str(excinfo_inst.value)
        == "Array.is_payload expects a MemoryBase subclass or instance; got <class 'str'>"
    )
