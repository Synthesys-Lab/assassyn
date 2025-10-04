# PyRamulator: Python Wrapper for Ramulator DRAM Simulator

This document explains the structure and functionality of the provided Python script, with a focus on the `PyRamulator` class and its interactions with the underlying C++ DRAM simulator libraries (`libwrapper.so` and `libramulator.so`).

---

## Overview

The Python module serves as a **`ctypes` binding** to a C++ DRAM simulation environment. It allows Python code to:
- Initialize a DRAM simulator.
- Send memory access requests.
- Advance the simulation clock.
- Handle callbacks from C++ to Python.

The interface is primarily defined through the `PyRamulator` class, which encapsulates a `MyWrapper` C++ object exposed via the shared library `libwrapper.so`.

---

## PyRamulator functions

`def __init__(self, config_path: str)`: Initialize the dram simulator with config file.
`def __del__(self):`: Destructor that releases the C++ simulator object when the Python instance is destroyed.
`def get_memory_tCK(self) -> float`: Retrieves the DRAM clock period (tCK) from the simulator.
`def finish(self)`: Signals the simulator that all operations are complete and finalizes internal state.
`def frontend_tick(self)`: Advances the frontend of the memory simulator by one cycle.
`def memory_system_tick(self)`: Advances the memory system (DRAM-side) by one simulation cycle.
`def send_request(self, addr: int, is_write: bool, callback, ctx) -> bool`: Sends a memory access request (read/write) to the simulator and registers a callback to be triggered upon read request completion.

