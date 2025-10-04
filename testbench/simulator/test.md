# DRAM Request Test Program Explanation

This program demonstrates how to use `MyWrapper` to send read and write requests to a simulated DRAM system.  
It is designed to produce the same result as `assassyn/python/ci-tests/test_dram.py`, ensuring that the C++ implementation matches the Python unit test. We use `libramulator.so` from `Ramulator2.0` to implement the `MyWrapper` interface, exposing initialization, request, ticking, and finishing functions.

---

## Overview

The program:
1. Initializes a memory wrapper with a configuration file.
2. Iterates over 200 cycles, alternating between **write** and **read** requests.
3. Sends requests to the memory system and prints:
   - The request status (success/failure for writes).
   - The response when a read request is completed.
4. Ticks the frontend and memory system to simulate time progression.
5. Finishes the simulation cleanly.

---

## Key Functions in `MyWrapper`

The `MyWrapper` class provides a simple interface for interacting with the simulated memory system.  
It allows you to initialize the system, send requests, tick the simulation, and finish cleanly.

---

### 1. `void init(const std::string &config_path);`

Initializes the memory wrapper with the given configuration file.

- **Parameter:**  
  `config_path` — Path to the YAML configuration file (e.g., `../../configs/example_config.yaml`).

- **Usage:**  
  Call this once before sending any memory requests.

### 2. `bool send_request(int64_t addr, bool is_write std::function<void(Ramulator::Request &)> callback);`

send request to memory system.

- **Parameter:**  
  `addr` — address.
  `is_write` - request type: read/write.
  `callback` - callback function, when request finished, the callback will be called (only for read request).

- **Output:**
  `bool` - request sent to memory system successfully or not

- **Usage:**  
  Call this function to send memory requests.

### 3. `void finish();`
- **Usage:** 
  Finalizes the simulation and collects statistics.

### 4. `void frontend_tick();` `void memory_system_tick();`
- **Usage:** 
  Advance the frontend and memory system by one simulation cycle.

---

