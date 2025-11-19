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

### build_simulator

```python
def build_simulator(manifest_path: str, offline: bool = False) -> str
```

Build the simulator binary using cargo build.

**Parameters:**
- `manifest_path`: Path to the Cargo.toml manifest file or a cached binary path
- `offline`: Whether to run cargo in offline mode (default: False)

**Returns:**
- The path to the compiled binary executable

**Explanation:**
This function compiles the Rust-based simulator using `cargo build --release` and manages the build cache. It 
performs the following steps:

1. **Cache Check**: If `manifest_path` is already a binary (from a cache hit in `elaborate()`), it immediately 
   returns the path without compilation
2. **Compilation**: If `manifest_path` is a Cargo.toml file, it constructs the appropriate cargo build command 
   and compiles the simulator. If the initial build fails and `offline` was not explicitly requested, it retries 
   automatically with `--offline` to support environments without network access
3. **Cache Saving**: After successful compilation, if [`backend.elaborate()`](../backend.py) set the global 
   `CACHE_PENDING` variable, this function calls `save_build_cache()` to store the build metadata for future runs

The build cache coordination between `elaborate()` and this function enables significant speedup in development 
workflows by eliminating redundant compilation when the IR and configuration haven't changed. For scenarios 
requiring multiple runs with the same simulator (e.g., different test workloads), build once with this function, 
then use `run_simulator()` with the `binary_path` parameter to run the binary directly without recompiling.

### get_simulator_binary_path

```python
def get_simulator_binary_path(manifest_path: str) -> str
```

Get the path to the compiled simulator binary.

**Parameters:**
- `manifest_path`: Path to the Cargo.toml manifest file

**Returns:**
- The absolute path to the compiled binary executable

**Explanation:**
This function determines the path to the compiled simulator binary without building it. It parses the Cargo.toml
file to extract the package name, then constructs the path to the binary in the cargo target directory. It
respects the `CARGO_TARGET_DIR` environment variable if set (used by the build system), otherwise uses the
default `target/release/` directory relative to the manifest. This function requires Python 3.11+ for the built-in
`tomllib` module, or falls back to the `toml` package for earlier Python versions.

### run_simulator

```python
def run_simulator(manifest_path: str = None, offline: bool = False, release: bool = True, binary_path: str = None) -> str
```

The helper function to run the simulator.

**Parameters:**
- `manifest_path`: Path to the Cargo.toml manifest file (optional, used if `binary_path` is None)
- `offline`: Whether to run cargo in offline mode (default: False)
- `release`: Whether to build in release mode (default: True)
- `binary_path`: Path to a pre-compiled simulator binary (optional, for direct execution)

**Returns:**
- The simulator output as a string

**Explanation:**
This function runs the Rust-based simulator in one of two modes:

1. **Direct Binary Execution** (when `binary_path` is provided): Runs the pre-compiled binary directly without
   invoking cargo. This is significantly faster for scenarios where you need to run the same simulator multiple
   times with different workloads (e.g., testing multiple CPU workloads). Use `build_simulator()` first to
   compile the binary once, then call this function with `binary_path` parameter for each run.

2. **Cargo Run Mode** (when `binary_path` is None): Falls back to the traditional approach of using `cargo run`
   with the provided manifest path. It constructs the appropriate cargo command with the provided flags, prints
   the command being executed, and captures stdout. If the initial invocation fails and `offline` was not
   explicitly requested, it retries automatically with `--offline` to cover environments without network access.

**Performance Optimization:**
For workloads that require running the simulator multiple times (e.g., `minor-cpu` with 30+ test cases), using
the binary_path mode can dramatically reduce total execution time by eliminating redundant compilation overhead.

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

### check_build_cache

```python
def check_build_cache(src_dir: str, cache_key: str) -> tuple[str, str] | None
```

Check if cached build exists and is valid.

**Parameters:**
- `src_dir`: Directory where cache file is stored (typically the test file's directory)
- `cache_key`: Combined IR hash + config hash key

**Returns:**
- A tuple `(binary_path, verilog_path)` if cache exists and is valid, `None` otherwise

**Explanation:**
This function checks for a cached build by reading the `.build_cache.json` file in the source directory. It 
verifies that:
1. The cache file exists
2. The cache key matches the current build
3. The cached binary still exists on disk

If all conditions are met, it returns the paths to the cached binary and Verilog output. Otherwise, it returns 
`None` to indicate a cache miss. This function is called by [`backend.elaborate()`](../backend.py) to check for 
cached builds before performing expensive IR processing and code generation.

### save_build_cache

```python
def save_build_cache(src_dir: str, cache_key: str, binary: str, verilog: str) -> None
```

Save build cache metadata.

**Parameters:**
- `src_dir`: Directory where cache file will be stored
- `cache_key`: Combined IR hash + config hash key
- `binary`: Path to built simulator binary
- `verilog`: Path to generated verilog (optional, can be None)

**Returns:**
- None

**Explanation:**
This function saves cache metadata to a `.build_cache.json` file in the source directory. The cache file stores:
- `key`: The combined IR and configuration hash for cache validation
- `binary`: Absolute path to the compiled simulator binary
- `verilog`: Absolute path to generated Verilog directory (if any)

This function is called by `build_simulator()` after successful compilation to cache the build for future runs. 
The cache enables significant speedup in development workflows by eliminating redundant compilation when the IR 
and configuration haven't changed.

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
