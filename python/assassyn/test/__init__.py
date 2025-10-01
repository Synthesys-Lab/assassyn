"""Test utilities for assassyn systems."""

from assassyn.frontend import SysBuilder
from assassyn.backend import elaborate
from assassyn import utils
import inspect

def run_test(name: str, top: callable, checker: callable, **config):
    """
    Lightweight test utility for assassyn systems.

    Args:
        name: System name (must be unique across testcases)
        top: Callable that builds the system (receives no args or sys, uses sys context)
        checker: Callable that validates simulator output (receives raw string)
        **config: Additional config passed to elaborate()
            (e.g., sim_threshold, idle_threshold, random)
    """
    sys = SysBuilder(name)
    with sys:
        # Check if top() accepts a parameter
        sig = inspect.signature(top)
        if len(sig.parameters) > 0:
            top(sys)
        else:
            top()

    # Set defaults, allow overrides
    cfg = {'verilog': utils.has_verilator()}
    cfg.update(config)

    simulator_path, verilator_path = elaborate(sys, **cfg)

    raw = utils.run_simulator(simulator_path)
    checker(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        checker(raw)
