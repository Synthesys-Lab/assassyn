# Utils Module

This module provides utility functions for the assassyn project, supporting object identification, path management, simulation execution, and code patching operations.

## Section 1. Exposed Interfaces

This section describes all the function interfaces and data structures in this source file unit that are exposed to the usage for other parts of the project.

### identifierize

```python
def identifierize(obj) -> str
```

The helper function to get the identifier of the given object. You can change `id_slice` to tune the length of the identifier. The default is slice(-5:-1).

**Parameters:**
- `obj`: Any Python object to generate an identifier for

**Returns:**
- `str`: A short hexadecimal identifier based on the object's memory address

### unwrap_operand

```python
def unwrap_operand(node) -> Any
```

Unwrap the operand from the node. This is a helper function to get the operand from the node.

**Parameters:**
- `node`: The node to unwrap, can be an Operand instance or any other object

**Returns:**
- `Any`: The value from the Operand node if it's an Operand instance, otherwise returns the node unchanged

### repo_path

```python
def repo_path() -> str
```

Get the path to assassyn repository.

**Returns:**
- `str`: The path to the assassyn repository root directory from the ASSASSYN_HOME environment variable

### package_path

```python
def package_path() -> str
```

Get the path to this python package.

**Returns:**
- `str`: The path to the Python package directory by appending '/python/assassyn' to the repository root

### patch_fifo

```python
def patch_fifo(file_path: str) -> None
```

Replaces all occurrences of 'fifo_n #(' with 'fifo #(' in the Top.sv file.

**Parameters:**
- `file_path`: The path to the file to patch

**Returns:**
- `None`: Function modifies the file in place

**Note:** ⚠️ **CONTEXT NEEDED**: The function uses regex pattern `r'fifo_\d+\s*#\s*\('` to normalize FIFO instantiations, but the exact reason why this patching is needed and the context of the generated Verilog code needs better documentation.

### run_simulator

```python
def run_simulator(manifest_path: str, offline: bool = False, release: bool = True) -> str
```

The helper function to run the simulator.

**Parameters:**
- `manifest_path`: Path to the Cargo manifest file
- `offline`: Whether to run in offline mode (default: False)
- `release`: Whether to run in release mode (default: True)

**Returns:**
- `str`: The output from the simulator execution

### run_verilator

```python
def run_verilator(path: str) -> str
```

The helper function to run the verilator. Changes to the specified directory, executes design.py, patches the Top.sv file, runs the testbench, and restores the original working directory.

**Parameters:**
- `path`: The directory path to run verilator in

**Returns:**
- `str`: The output from the verilator testbench execution

### parse_verilator_cycle

```python
def parse_verilator_cycle(toks: list) -> int
```

Helper function to parse verilator dumped cycle.

**Parameters:**
- `toks`: List of tokens from verilator output

**Returns:**
- `int`: The parsed cycle number

**Note:** ⚠️ **IMPLEMENTATION DETAIL**: The function parses the third token (index 2) and removes the first and last 4 characters. The exact format of the input tokens needs better documentation for future maintainers.

### parse_simulator_cycle

```python
def parse_simulator_cycle(toks: list) -> int
```

Helper function to parse rust-simulator dumped cycle.

**Parameters:**
- `toks`: List of tokens from simulator output

**Returns:**
- `int`: The parsed cycle number

**Note:** ⚠️ **IMPLEMENTATION DETAIL**: The function parses the third token (index 2) and removes the first and last 4 characters. The exact format of the input tokens needs better documentation for future maintainers.

### has_verilator

```python
def has_verilator() -> str | None
```

Returns the path to Verilator or None if VERILATOR_ROOT is not set.

**Returns:**
- `str | None`: 'verilator' if VERILATOR_ROOT environment variable is set and points to a valid directory, None otherwise

### create_and_clean_dir

```python
def create_and_clean_dir(dir_path: str) -> None
```

Create a directory and clear its contents if it already exists.

**Parameters:**
- `dir_path`: The directory path to create

**Returns:**
- `None`: Function creates the directory structure

**Note:** ⚠️ **DESIGN INCONSISTENCY**: Despite the function name suggesting "clean", the current implementation only creates the directory using `os.makedirs(dir_path, exist_ok=True)` without actually clearing existing contents. This needs human decision on whether to fix the implementation or rename the function.

### namify

```python
def namify(name: str) -> str
```

Convert a name to a valid identifier. This matches the Rust function in src/backend/simulator/utils.rs.

**Parameters:**
- `name`: The string to convert to a valid identifier

**Returns:**
- `str`: A valid identifier with non-alphanumeric characters (except underscore) replaced with underscores

## Section 2. Internal Helpers

This section describes all the function interfaces and data structures that are implemented in this source code unit but are not exposed to other parts of the project.

### _cmd_wrapper

```python
def _cmd_wrapper(cmd) -> str
```

Internal helper function to execute shell commands and return their output.

**Parameters:**
- `cmd`: The command to execute as a list of strings

**Returns:**
- `str`: The decoded output from the command execution

### PATH_CACHE

Global variable used to cache the repository path to avoid repeated environment variable lookups.