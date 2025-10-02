# Test Utility

## Exposed Interface

This module provides `run_test()` for simplified unit testing of assassyn systems.

### Usage

```python
run_test(name: str, top: callable, checker: callable, **config)
```

**Arguments:**
- `name`: System name (must be unique across testcases)
- `top`: Callable that builds the system (receives no args or sys parameter)
- `checker`: Callable that validates simulator output (receives raw string)
- `**config`: Additional config passed to elaborate() (e.g., sim_threshold, idle_threshold, random)

**What it does:**
1. Builds the system using SysBuilder with the provided top function
2. Elaborates the system with default config (verilog enabled if verilator available)
3. Runs the simulator and validates output with the checker function
4. If verilator is available, runs verilator and validates its output too
