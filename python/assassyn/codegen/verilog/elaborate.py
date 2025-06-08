"""Elaborate Assassyn IR to Verilog."""

import os
from pathlib import Path
from .testbench import generate_testbench
from .design import generate_design

from ...builder import SysBuilder

from ...utils import create_and_clean_dir

def elaborate(sys: SysBuilder, **kwargs) -> str:
    """Elaborate the system into Verilog.

    Args:
        sys: The system to elaborate
        **kwargs: Configuration options including:
            - verilog: The simulator to use ("Verilator", "VCS", or None)
            - resource_base: Path to resources
            - override_dump: Whether to override existing files
            - sim_threshold: Simulation threshold
            - idle_threshold: Idle threshold
            - random: Whether to randomize execution
            - fifo_depth: Default FIFO depth

    Returns:
        Path to the generated Verilog files
    """

    path = kwargs.get('path', os.getcwd())
    path = Path(path) / f"{sys.name}_verilog"

    create_and_clean_dir(path)

    logs = generate_design(path / "design.py", sys)

    generate_testbench(path / "tb.py", sys, kwargs['sim_threshold'], logs)

    return path
