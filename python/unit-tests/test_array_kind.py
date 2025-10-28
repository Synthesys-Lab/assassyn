"""Tests for Array.kind classification and codegen integration."""

from assassyn.builder import SysBuilder
from assassyn.ir.array import RegArray, ArrayKind
from assassyn.ir.dtype import UInt
from assassyn.ir.memory.sram import SRAM
from assassyn.ir.memory.dram import DRAM
from assassyn.ir.module import Module, module
from assassyn.codegen.verilog.design import CIRCTDumper


def test_regarray_defaults_to_reg_kind():
    """RegArray without overrides should default to ArrayKind.REG."""
    sys = SysBuilder("array_kind_default")
    with sys:
        arr = RegArray(UInt(8), 4, name="kind_arr")

    assert arr.kind is ArrayKind.REG


def test_memory_payload_kinds():
    """MemoryBase subclasses should tag their payload arrays."""
    sys = SysBuilder("array_kind_memory_payloads")
    with sys:
        sram = SRAM(16, 32, None)
        dram = DRAM(16, 32, None)

    assert sram._payload.kind is ArrayKind.SRAM_PAYLOAD  # pylint: disable=protected-access
    assert dram._payload.kind is ArrayKind.DRAM_PAYLOAD  # pylint: disable=protected-access


def test_metadata_registry_skips_memory_payloads():
    """ArrayMetadataRegistry should ignore memory payloads while keeping regular arrays."""

    class Reader(Module):
        def __init__(self):
            super().__init__(ports={})

        @module.combinational
        def build(self, arr):
            _ = arr[0]  # create an ArrayRead so metadata sees a usage

    sys = SysBuilder("array_kind_metadata")
    with sys:
        reg_arr = RegArray(UInt(8), 4, name="regular_arr")
        sram = SRAM(8, 16, None)

        reader = Reader()
        reader.build(reg_arr)

    dumper = CIRCTDumper()
    dumper.array_metadata.collect(dumper, sys)

    assert dumper.array_metadata.metadata_for(reg_arr) is not None
    assert dumper.array_metadata.metadata_for(sram._payload) is None  # pylint: disable=protected-access
