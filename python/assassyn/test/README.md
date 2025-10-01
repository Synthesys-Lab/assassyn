# Test Utility

## Exposed Interface

This module provides utility functions for unit tests.
Users can just plug in their top and checker functions to `test_sys()`.

````python
def test_sys(name: str, top: callable, check: callable, config: dict):
    # update the config
    # ...

    # Build the given system
    with SysBuilder(name) as sys:
        top()
    # Elaborate the system
    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())
    # Check the simulator output
    raw = utils.run_simulator(simulator_path)
    checker(raw)
    # Check the verilator output when available 
    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        checker(raw)
````
