# Utils Module

This module provides utility functions for the assassyn project, supporting object identification, path management, 
simulation execution, and code generation workflows.

---

## Section 1. Exposed Interfaces

This section describes all the function interfaces and data structures in this source file unit that are exposed 
to the usage for other parts of the project.

### identifierize

```python
def identifierize(obj) -> str
```

The helper function to get the identifier of the given object. You can change `id_slice` to tune the length of the 
identifier. The default is slice(-6:-1).

**Parameters:**
- `obj`: Any Python object to generate an identifier for

**Returns:**
- A hexadecimal string identifier based on the object's memory address

**Explanation:**
This function generates a short hexadecimal identifier for any Python object based on its memory address. The 
identifier length is controlled by `Singleton.id_slice` (default: `slice(-6, -1)` from the builder module). 
This is used throughout the IR system to generate unique names for objects when semantic names are not available.

### unwrap_operand

```python
def unwrap_operand(node) -> Any
```

Unwrap the operand from the node. This is a helper function to get the operand from the node.

**Parameters:**
- `node`: The node to unwrap, can be an `Operand` instance or any other object

**Returns:**
- The unwrapped value if `node` is an `Operand`, otherwise returns `node` unchanged

**Explanation:**
This function extracts the actual value from an `Operand` wrapper in the IR system. If the input is an `Operand` 
instance, it returns the wrapped value; otherwise, it returns the input unchanged. This is used extensively in 
code generation to access the underlying values from IR nodes.

### repo_path

```python
def repo_path() -> str
```

Get the path to assassyn repository.

**Returns:**
- The absolute path to the assassyn repository root directory

**Explanation:**
This function returns the assassyn repository root directory by reading the `ASSASSYN_HOME` environment variable. 
The result is cached in the global `PATH_CACHE` variable to avoid repeated environment variable lookups. This 
function is used throughout the codebase to locate repository resources and generate absolute paths.

### package_path

```python
def package_path() -> str
```

Get the path to this python package.

**Returns:**
- The absolute path to the Python package directory (`/python/assassyn`)

**Explanation:**
This function returns the Python package directory path by appending `/python/assassyn` to the repository root. 
It's used by the builder system to determine package-specific paths for stack inspection and module loading.

### patch_fifo

```python
def patch_fifo(file_path: str) -> None
```

Replaces all occurrences of 'fifo_n #(' with 'fifo #(' in the Top.sv.

**Parameters:**
- `file_path`: Path to the Verilog file to patch

**Explanation:**
This function patches Verilog files by normalizing FIFO instantiations. It uses regex pattern `r'fifo_\d+\s*#\s*\('` 
to find numbered FIFO instantiations and replaces them with the standard `fifo #(` format. This is used in the 
Verilator workflow to ensure consistent FIFO naming in generated Verilog code.

### run_simulator

```python
def run_simulator(manifest_path: str, offline: bool = False, release: bool = True) -> str
```

The helper function to run the simulator.

**Parameters:**
- `manifest_path`: Path to the Cargo.toml manifest file
- `offline`: Whether to run cargo in offline mode (default: False)
- `release`: Whether to build in release mode (default: True)

**Returns:**
- The simulator output as a string

**Explanation:**
This function executes the Rust-based simulator using cargo. It constructs the appropriate cargo command with 
the provided manifest path and optional flags, prints the command being executed, and captures stdout. If the 
initial invocation fails and `offline` was not explicitly requested, it retries automatically with `--offline` 
to cover environments without network access.

### run_verilator

```python
def run_verilator(path: str) -> str
```

The helper function to run the verilator.

**Parameters:**
- `path`: Directory path where the Verilator workflow should be executed

**Returns:**
- The testbench output as a string

**Explanation:**
This function runs the complete Verilator simulation workflow:
1. Changes to the specified directory
2. Executes `design.py` to generate Verilog code
3. Applies `patch_fifo()` to `sv/hw/Top.sv` to normalize FIFO instantiations
4. Executes `tb.py` for the testbench
5. Restores the original working directory

The function ensures proper cleanup by restoring the original working directory even if errors occur.

### parse_verilator_cycle

```python
def parse_verilator_cycle(toks: list) -> int
```

Helper function to parse verilator dumped cycle.

**Parameters:**
- `toks`: List of tokens from the simulation output

**Returns:**
- The parsed cycle count as an integer

**Explanation:**
This function extracts cycle counts from Verilator simulation output tokens. It parses the third token (index 2) 
and removes the first and last 4 characters to extract the cycle number.

### parse_simulator_cycle

```python
def parse_simulator_cycle(toks: list) -> int
```

Helper function to parse rust-simulator dumped cycle.

**Parameters:**
- `toks`: List of tokens from the simulation output

**Returns:**
- The parsed cycle count as an integer

**Explanation:**
This function extracts cycle counts from Rust simulator output tokens. It parses the third token (index 2) 
and removes the first and last 4 characters to extract the cycle number.

### has_verilator

```python
def has_verilator() -> str | None
```

Returns the path to Verilator or None if VERILATOR_ROOT is not set.

**Returns:**
- The string 'verilator' if available, None otherwise

**Explanation:**
This function checks if Verilator is available by verifying that the `VERILATOR_ROOT` environment variable exists 
and points to a valid directory, and by ensuring the `pycde` Python package is importable (a prerequisite for the
code generation flow). Results are cached in `VERILATOR_CACHE`, and the function returns `'verilator'` on success 
or `None` otherwise.

### create_dir

```python
def create_dir(dir_path: str) -> None
```

Create a directory if it doesn't exist.

**Parameters:**
- `dir_path`: Path to the directory to create

**Raises:**
- `OSError`: If directory creation fails due to permissions or disk space

**Explanation:**
This function creates a directory and all necessary parent directories using `os.makedirs(dir_path, exist_ok=True)`. 
If the directory already exists, it does nothing. This is a simple utility for ensuring directory existence.

### namify

```python
def namify(name: str) -> str
```

Convert a name to a valid identifier. This matches the Rust function in src/backend/simulator/utils.rs.

**Parameters:**
- `name`: The string to convert to a valid identifier

**Returns:**
- A valid identifier string with non-alphanumeric characters replaced by underscores

**Explanation:**
This function converts an arbitrary string to a valid identifier by replacing all non-alphanumeric characters 
(except underscore) with underscores. This matches the Rust implementation in `src/backend/simulator/utils.rs` 
and ensures consistency across language boundaries. It's used extensively in code generation to create valid 
variable and module names.

---

## Section 2. Internal Helpers

This section describes internal helper functions and data structures that are implemented in this source code unit.

### PATH_CACHE

```python
PATH_CACHE = None
```

Global variable that caches the repository path to avoid repeated environment variable lookups.

### VERILATOR_CACHE

```python
VERILATOR_CACHE = None
```

Global variable that caches the result of `has_verilator()` to avoid repeating environment and dependency checks.

### _cmd_wrapper

```python
def _cmd_wrapper(cmd) -> str
```

Internal helper function that executes a command and returns its output as a decoded UTF-8 string.

**Parameters:**
- `cmd`: Command to execute as a list of strings

**Returns:**
- The command output as a decoded UTF-8 string

**Explanation:**
This is a simple wrapper around `subprocess.check_output()` that automatically decodes the output to UTF-8. 
It's used by `run_simulator()` and `run_verilator()` to capture command output.
