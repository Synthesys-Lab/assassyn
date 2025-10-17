# PyRamulator - Python Wrapper for Ramulator2

## Design Documents

- [Memory System Architecture](../../../docs/design/arch/memory.md) - Memory system design
- [Simulator Design](../../../docs/design/internal/simulator.md) - Simulator design and code generation
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system
- [Architecture Overview](../../../docs/design/arch/arch.md) - Overall system architecture

## Related Modules

- [Simulator Generation](../codegen/simulator/simulator.md) - Simulator code generation
- [Module Generation](../codegen/simulator/modules.md) - Module-to-Rust translation
- [Memory Modules](../ir/memory/) - Memory system IR modules

## Summary

PyRamulator provides a Python interface to the Ramulator2 memory simulator through C++ wrapper libraries. This module enables Assassyn's simulator backend to integrate with external memory simulation capabilities, supporting the [Module](../ir/module/module.md) pipeline stage architecture described in the design documents. The wrapper handles cross-platform shared library loading and provides callback-based memory request handling for simulation coordination.

**PyRamulator Integration with Assassyn Module Concept:** PyRamulator integrates with Assassyn's module system through:

1. **Module Integration**: PyRamulator instances are integrated with Assassyn modules for memory simulation
2. **Pipeline Stage Support**: Supports the credit-based pipeline architecture described in [pipeline.md](../../../docs/design/internal/pipeline.md)
3. **Callback Integration**: Memory callbacks are integrated with the simulator's callback system
4. **Memory Interface**: Provides a consistent memory interface for the simulator backend

**Platform-Specific Details:** PyRamulator handles platform-specific requirements:

1. **macOS RTLD_GLOBAL Mode**: On macOS, libraries are loaded with `RTLD_GLOBAL` mode for proper symbol resolution
2. **Cross-Platform Library Loading**: Handles different library loading mechanisms across platforms
3. **Library Fallback Mechanism**: Implements fallback mechanisms for library loading failures
4. **Platform-Specific Error Handling**: Provides platform-specific error messages and handling

**Callback Parameter Description:** The callback parameter is used for both read and write operations:

1. **Read Operations**: Callback is called with read data when read requests complete
2. **Write Operations**: Callback is called with write confirmation when write requests complete
3. **Unified Interface**: Single callback interface handles both read and write operations
4. **Request Tracking**: Callbacks are used to track request completion and data transfer

**Request Structure Completion:** The Request structure includes all necessary fields for memory operations:

1. **Address Field**: Memory address for the request
2. **Data Field**: Data to be written (for write operations)
3. **Request Type**: Type of request (read or write)
4. **Callback Field**: Callback function for request completion
5. **Request ID**: Unique identifier for request tracking

**__del__ Method Conditional Cleanup Logic:** The `__del__` method implements conditional cleanup:

1. **Instance Check**: Only performs cleanup if the instance exists
2. **Resource Cleanup**: Cleans up allocated resources and handles
3. **Error Handling**: Handles cleanup errors gracefully
4. **Memory Management**: Ensures proper memory management during destruction

## Exposed Interfaces

### PyRamulator Class

The main interface class that encapsulates memory simulation functionality.

#### `__init__(config_path: str)`

Initializes a new PyRamulator instance with the specified configuration file.

**Parameters:**
- `config_path` (str): Path to the YAML configuration file (e.g., `example_config.yaml`)

**Raises:**
- `RuntimeError`: If the CRamualator2Wrapper instance cannot be created

**Example:**
```python
from assassyn.ramulator2 import PyRamulator
sim = PyRamulator("/path/to/config.yaml")
```

#### `get_memory_tCK() -> float`

Returns the memory clock period (tCK) in nanoseconds.

**Returns:**
- `float`: Memory clock period in nanoseconds

**Example:**
```python
clock_period = sim.get_memory_tCK()
print(f"Memory clock period: {clock_period} ns")
```

#### `send_request(addr: int, is_write: bool, callback, ctx) -> bool`

Sends a memory request to the simulated memory system.

**Parameters:**
- `addr` (int): Memory address for the request
- `is_write` (bool): `True` for write request, `False` for read request
- `callback`: Python function to call when request completes (for read requests)
- `ctx`: Context object passed to the callback function

**Returns:**
- `bool`: `True` if request was successfully enqueued, `False` otherwise

**Raises:**
- `ValueError`: If callback is `None`

**Example:**
```python
def request_callback(req, cycle):
    print(f"Request completed at cycle {cycle}: addr={req.addr}")

success = sim.send_request(0x1000, False, request_callback, 42)
```

#### `frontend_tick()`

Advances the frontend simulation by one clock cycle. This processes incoming requests and manages the request queue.

**Example:**
```python
sim.frontend_tick()
```

#### `memory_system_tick()`

Advances the memory system simulation by one clock cycle. This processes memory operations and updates timing.

**Example:**
```python
sim.memory_system_tick()
```

#### `finish()`

Finalizes the simulation and collects statistics. Should be called when simulation is complete.

**Example:**
```python
sim.finish()
```

#### `__del__()`

Destructor that automatically cleans up the underlying C++ wrapper instance when the Python object is garbage collected.

### Request Structure

The `Request` class represents a memory request with the following key fields:

- `addr` (int64): Memory address
- `arrive` (int64): Cycle when request arrived
- `depart` (int64): Cycle when request completed
- `command` (int): Memory command type
- `is_stat_updated` (bool): Whether statistics were updated

### `get_library_paths() -> tuple`

Gets the paths to both the wrapper and ramulator2 shared libraries by constructing them directly from ASSASSYN_HOME.

**Returns:**
- `tuple`: Tuple containing (wrapper_lib_path, ramulator2_lib_path) both without extensions

**Raises:**
- `FileNotFoundError`: If the ASSASSYN_HOME environment variable is not set

**Explanation:**
This function constructs library paths directly from the ASSASSYN_HOME environment variable, eliminating the need for CMake-generated path files. The paths are constructed as:
- Wrapper lib: `{ASSASSYN_HOME}/tools/c-ramulator2-wrapper/build/lib/libwrapper`
- Ramulator2 lib: `{ASSASSYN_HOME}/3rd-party/ramulator2/libramulator`

This unified approach reduces code duplication and provides a single source of truth for library path construction. The function is used internally during module initialization to load the required shared libraries.

### `load_shared_library(lib_path: str) -> ctypes.CDLL`

Loads a shared library with fallback for different extensions and platform-specific handling.

**Parameters:**
- `lib_path` (str): Path to the shared library (without extension)

**Returns:**
- `ctypes.CDLL`: Loaded shared library object

**Raises:**
- `FileNotFoundError`: If no compatible library is found with any supported extension

**Explanation:**
This function handles cross-platform shared library loading with the following behavior:
1. Checks if the path already has an extension and loads it directly
2. Tries different extensions in order of preference: `.dylib` (macOS), `.so` (Linux), `.dll` (Windows)
3. Uses `RTLD_GLOBAL` mode on macOS for compatibility with recursive shared object dependencies as documented in [simulator.md](../codegen/simulator/simulator.md)
4. Raises a clear error message if no compatible library is found

**Note:** This function is called automatically during module import to load the wrapper and ramulator2 libraries. The loaded libraries are stored in the module-level variables `wrapper` and `ramulator`.

## Module Initialization

When the `ramulator2` module is imported, it automatically:

1. **Constructs Library Paths**: Calls `get_library_paths()` to determine the correct paths for both libraries
2. **Loads Shared Libraries**: Uses `load_shared_library()` to load both the wrapper and ramulator2 libraries
3. **Sets Up Function Bindings**: Configures the ctypes function signatures for all wrapper functions
4. **Stores Library References**: Keeps references to the loaded libraries in module-level variables

This initialization happens once per Python process and ensures that all PyRamulator instances can use the same loaded libraries efficiently.

## Usage Pattern

A typical simulation loop follows this pattern:

```python
from assassyn.ramulator2 import PyRamulator, Request

# Initialize simulator
sim = PyRamulator("config.yaml")

# Simulation loop
for cycle in range(num_cycles):
    # Send requests
    if should_send_request:
        sim.send_request(address, is_write, callback, context)
    
    # Advance simulation
    sim.frontend_tick()
    sim.memory_system_tick()

# Clean up
sim.finish()
```

## Dependencies

The PyRamulator module requires:
- `libwrapper`: Built from `tools/c-ramulator2-wrapper/CRamualator2Wrapper.cpp` (with OS-appropriate extension)
- `libramulator`: Core Ramulator2 library from `3rd-party/ramulator2/` (with OS-appropriate extension)
- Python `ctypes` module for C library interfacing

## Cross-Platform Support

PyRamulator automatically handles different operating systems by:
1. Detecting the current OS using `sys.platform`
2. Loading the appropriate shared library extension (`.so`, `.dll`, or `.dylib`)
3. Providing fallback mechanisms to try alternative extensions if the primary one fails
4. Raising a clear error message if no compatible library is found

## Building Requirements

Before using PyRamulator, ensure the wrapper library is built:

```bash
cd tools/c-ramulator2-wrapper
mkdir -p build
cd build
cmake ..
make
```

This creates `libwrapper` (with the appropriate OS extension) in the `build/lib/` directory. PyRamulator locates and loads the correct shared library for your platform.

## Cross-Validation Suite

PyRamulator is part of a comprehensive cross-validation suite that ensures consistency across different language implementations:

- **C++ Test**: `tools/c-ramulator2-wrapper/test.cpp`
- **Python Test**: `python/unit-tests/test_ramulator2.py`
- **Rust Test**: `tools/rust-sim-runtime/src/test_ramulator2.rs`

All implementations must produce **identical output** when given the same:
- Configuration file
- Request sequence
- Simulation parameters

This validates that the language bindings correctly interface with the core `libramulator` library (with OS-appropriate extension) and maintain behavioral consistency across different programming languages and operating systems.