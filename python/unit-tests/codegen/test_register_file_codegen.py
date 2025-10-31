"""Ensure register-file generation defers to assassyn.pycde_wrapper.build_register_file."""

from __future__ import annotations

import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (  # type: ignore
    Condition,
    Module,
    Port,
    RegArray,
    SysBuilder,
    UInt,
    module,
)
from assassyn.codegen.verilog.design import generate_design  # type: ignore
from assassyn.pycde_wrapper import build_register_file  # type: ignore
from assassyn.utils import namify  # type: ignore
from pycde.types import UInt as PycdeUInt  # type: ignore


def _emit_design(sys_builder: SysBuilder, tmp_path, filename: str = 'design.py') -> str:
    """Generate design.py for the provided system and return its contents."""

    out_dir = tmp_path / "gen"
    out_dir.mkdir(parents=True, exist_ok=True)
    design_path = out_dir / filename
    generate_design(str(design_path), sys_builder)
    return design_path.read_text(encoding="utf-8")


def test_register_file_helper_invocation(tmp_path):
    """Generated arrays should be emitted via build_register_file instead of inline classes."""

    sys_builder = SysBuilder("register_file_helper")

    with sys_builder:
        class ArrayUser(Module):
            def __init__(self):
                super().__init__(
                    ports={
                        "en": Port(UInt(1)),
                        "idx": Port(UInt(2)),
                        "data": Port(UInt(8)),
                    }
                )

            @module.combinational
            def build(self):
                en = self.en.pop()
                idx = self.idx.pop()
                data = self.data.pop()
                array = RegArray(
                    UInt(8),
                    4,
                    name="helper_array",
                    initializer=[0, 1, 2, 3],
                )
                with Condition(en):
                    write_port = array & self
                    write_port[idx] <= data
                array[idx]

        ArrayUser().build()

    arrays = list(sys_builder.arrays)
    assert len(arrays) == 1
    class_name = namify(arrays[0].name)

    text = _emit_design(sys_builder, tmp_path)

    assert f"{class_name} = build_register_file(" in text
    assert f"class {class_name}(Module)" not in text
    assert "num_write_ports=1" in text
    assert "num_read_ports=1" in text
    assert "initializer=[0, 1, 2, 3]" in text
    assert "include_read_index=True" in text


def test_single_entry_array_omits_read_index(tmp_path):
    """Width-one arrays should omit read-index ports when using the helper."""

    sys_builder = SysBuilder("single_entry_helper")

    with sys_builder:
        class SingleEntry(Module):
            def __init__(self):
                super().__init__(
                    ports={
                        "en": Port(UInt(1)),
                        "data": Port(UInt(8)),
                    }
                )

            @module.combinational
            def build(self):
                en = self.en.pop()
                data = self.data.pop()
                array = RegArray(UInt(8), 1, name="single_array")
                with Condition(en):
                    write_port = array & self
                    write_port[0] <= data
                array[0]

        SingleEntry().build()

    arrays = list(sys_builder.arrays)
    assert len(arrays) == 1
    class_name = namify(arrays[0].name)

    text = _emit_design(sys_builder, tmp_path)

    assert f"{class_name} = build_register_file(" in text
    assert "include_read_index=False" in text
    assert "ridx_port" not in text


def test_build_register_file_prioritises_highest_port_first():
    """The helper should retain reverse-priority semantics for writes (highest index wins)."""

    module_cls = build_register_file(
        "rf_module",
        PycdeUInt(8),
        depth=4,
        num_write_ports=2,
        num_read_ports=0,
    )
    assert "reversed" in module_cls.construct.gen_func.__code__.co_names
