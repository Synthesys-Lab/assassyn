# Test Utilities (__init__.py)

## Section 0. Summary

Helpers for constructing systems and running checks for the Rust simulator and optional Verilator flow.

## Section 1. Exposed Interfaces

### run_test
```python
def run_test(name: str, top: callable, checker: callable, **kwargs):
    """
    Lightweight test utility for assassyn systems.

    @param name Unique system name used for workspace foldering
    @param top Builder callable; zero-arg or accepts a SysBuilder
    @param checker Function that consumes raw simulator (and optionally Verilator) output
    @param **kwargs Passed into backend.config(); keys include:
        - sim_threshold (int)
        - idle_threshold (int)
        - fifo_depth (int)
        - random (bool)
        - verilog (bool): run Verilator or not; default auto-detect via utils.has_verilator()
    """
```

Behavior:
- Builds a system with `SysBuilder` and `top`.
- Elaborates codegen to simulator and (optionally) Verilog artifacts.
- Always runs the Rust simulator and calls `checker(raw)`.
- If `verilog=True` and Verilator output is available, runs Verilator and calls `checker(raw)` again.

Simulator-only runs:
```python
run_test("name", top, checker, verilog=False)
```
Skips Verilator regardless of availability.

### dump_ir
```python
def dump_ir(name: str, builder: callable, checker: callable, print_dump: bool = True):
    """
    Build IR and pass repr(sys) to checker for assertions.

    @param name Unique identifier for the dump section
    @param builder Callable that receives SysBuilder
    @param checker Function that consumes repr(sys)
    @param print_dump If True, prints the IR dump
    """
```

## Section 2. Internal Helpers

No additional internal helpers are exposed from this unit.
