# Utils Module

This module provides utility functions for the assassyn project, supporting object identification, path management, sim>

---

## Exposed Interfaces

```python
def identifierize(obj) -> str
def unwrap_operand(node) -> Any
def repo_path() -> str
def package_path() -> str
def patch_fifo(file_path: str) -> None
def run_simulator(manifest_path: str, offline: bool = False, release: bool = True) -> str
def run_verilator(path: str) -> str
def parse_verilator_cycle(toks: list) -> int
def parse_simulator_cycle(toks: list) -> int
def has_verilator() -> str | None
def create_and_clean_dir(dir_path: str) -> None
def namify(name: str) -> str
```

---

## Path Management

**`repo_path()`** returns the assassyn repository root directory by reading the `ASSASSYN_HOME` environment variable. R>
**`package_path()`** returns the Python package directory path by appending `/python/assassyn` to the repository root.

---

## Object and IR Utilities

**`identifierize(obj)`** generates a short 4-character hexadecimal identifier for any Python object based on its memory>
**`unwrap_operand(node)`** extracts the value from an `Operand` node in the IR. If the node is an `Operand` instance (f>

---

## Simulation Helpers

**`run_simulator(manifest_path, offline=False, release=True)`** executes the Rust-based simulator using cargo. It const>
**`run_verilator(path)`** runs the complete Verilator simulation workflow:
1. Changes to the specified directory
2. Executes `design.py` to generate Verilog code
3. Applies `patch_fifo()` to `sv/hw/Top.sv`
4. Executes `tb.py` for the testbench
5. Restores the original working directory
Returns the testbench output.

**`parse_verilator_cycle(toks)`** and **`parse_simulator_cycle(toks)`** extract cycle counts from simulation output tok>
**`has_verilator()`** checks if Verilator is available by verifying that the `VERILATOR_ROOT` environment variable exis>

---

## Code Patching

**`patch_fifo(file_path)`** patches Verilog files by normalizing FIFO instantiations. It uses regex pattern `r'fifo_\d+>

---

## File System and Naming Utilities

**`create_and_clean_dir(dir_path)`** creates a directory and all necessary parent directories using `os.makedirs(dir_path, exist_ok=True)`. Note that despite the function name suggesting "clean", it does not clear existing directory contents—it only ensures the directory exists.
**`namify(name)`** converts an arbitrary string to a valid identifier by replacing all non-alphanumeric characters (except underscore) with underscores. This matches the Rust implementation in `src/backend/simulator/utils.rs` and ensures consistency across language boundaries.
