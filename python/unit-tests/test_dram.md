# Test implementation for DRAM memory operations
1. `test_dram.py`: A test suite that verifies memory operations by simulating a DRAM module with alternating read/write operations. It implements response handlers and drivers to validate that memory reads and writes behave correctly, with specific checks for data consistency. It uses DRAM function in `dram.py`. It also outputs the same result with `testbench/simulator/test.cpp`.

2. `dram.py`: The core DRAM module implementation that provides the fundamental memory operations infrastructure. It defines the DRAM architecture with configurable width and depth, handling the actual memory operations (read/write) and managing memory responses through its handler system.

see some intrinsic function descriptions in `python/assassyn/ir/expr/intrinsic.md`

