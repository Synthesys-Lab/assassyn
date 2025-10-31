"""Ensure generated PyCDE headers pull runtime helpers from assassyn.pycde_wrapper."""

import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import Module, SysBuilder, module  # noqa: E402
from assassyn.codegen.verilog.design import generate_design  # noqa: E402


def test_design_header_imports_pycde_wrapper(tmp_path):
    """Generated design.py should import shared wrapper primitives."""
    sysb = SysBuilder("pycde_header")
    with sysb:
        class Dummy(Module):
            def __init__(self):
                super().__init__(ports={})

            @module.combinational
            def build(self):
                return None

        Dummy().build()

    out_dir = tmp_path / "gen"
    os.makedirs(out_dir, exist_ok=True)
    design_path = out_dir / "design.py"
    generate_design(str(design_path), sysb)

    text = design_path.read_text(encoding="utf-8")

    assert "from assassyn.pycde_wrapper import FIFO, TriggerCounter, build_register_file" in text
    assert "class FIFOImpl(Module):" not in text
    assert "class TriggerCounterImpl(Module):" not in text
