# Triple Language Ramulator2 Cross Validation

The `test_trilang_ramulator2_valid.py` script runs three different Ramulator2 wrapper implementations and compares their outputs to ensure behavioral consistency.

## Core Functionality

The script runs these three implementations with the same configuration and request sequence:

- **C++ Implementation**: `tools/c-ramulator2-wrapper/test.cpp` (executable: `build/bin/test`)
- **Rust Implementation**: `tools/rust-sim-runtime/tests/test_ramulator2.rs` (via `cargo test`)
- **Python Implementation**: Runs directly via the Ramulator2 Python wrapper API

## Implementation Details

### Python Direct Execution

Unlike C++ and Rust which are executed as subprocesses, the Python implementation is executed directly within the validation script using the `PyRamulator` class. This approach:

- Avoids overhead from subprocess invocation
- Allows direct stdout suppression during statistics collection
- Captures output via Python list collection rather than stdout parsing

### Statistics Output Handling

Ramulator2's `finalize()` method prints detailed statistics in YAML format. To ensure fair comparison of only the simulation output (cycle messages), the script:

1. **Python**: Suppresses stdout by redirecting to `StringIO` during `finish()` call
2. **C++/Rust**: Filters out statistics lines (starting with `Frontend:` or `MemorySystem:`) from subprocess output

## Usage

### Basic Usage
```bash
python python/unit-tests/test_trilang_ramulator2_valid.py
```

### Command Line Options

- `--skip <language>`: Skip running a specific language implementation (`cpp`, `rust`, or `python`)
- `--debug`: Enable verbose debugging output with command/env details
- `--show-outputs`: Display filtered outputs from all languages before comparison

## Output Processing

The script processes and normalizes outputs for fair comparison:

1. **Statistics Filtering**: Removes Ramulator2 statistics output (lines starting with `Frontend:` or `MemorySystem:`) from all implementations
2. **Python Output Suppression**: Suppresses stdout from Python implementation's `finish()` call to avoid statistics output
3. **Rust Test Harness Filtering**: Removes Rust test harness noise (`running 1 test`, `test result: ok...`)
4. **Blank Line Removal**: Strips all blank lines to eliminate formatting differences
5. **Whitespace Normalization**: Strips trailing whitespace from all outputs

## Return Codes

- **0**: All outputs are identical
- **1**: Outputs differ (shows unified diff)
- **2**: Command execution failed

## Example Output

### Success
```bash
$ python python/unit-tests/test_trilang_ramulator2_valid.py
All outputs are identical across implementations.
```

### Failure with Differences
```bash
$ python python/unit-tests/test_trilang_ramulator2_valid.py
[DIFF] cpp vs rust:
--- cpp
+++ rust
@@ -1,3 +1,2 @@
 Cycle 3: Write request sent for address 2, success or not (true or false)true
 Cycle 9: Request completed: 2 the data is: 1
-Cycle 5: Write request sent for address 4, success or not (true or false)true
+Cycle 5: Write request sent for address 4, success or not (true or false)false
```

### Viewing Filtered Outputs
```bash
$ python python/unit-tests/test_trilang_ramulator2_valid.py --show-outputs

=== CPP OUTPUT (FILTERED) ===
Cycle 2: Write request sent for address 1, success or not (true or false)true
Cycle 3: Request completed: 0 the data is: -1
...

=== RUST OUTPUT (FILTERED) ===
Cycle 2: Write request sent for address 1, success or not (true or false)true
Cycle 3: Request completed: 0 the data is: -1
...

=== PYTHON OUTPUT (FILTERED) ===
Cycle 2: Write request sent for address 1, success or not (true or false)true
Cycle 3: Request completed: 0 the data is: -1
...
```

## Related Files

- `python/unit-tests/test_trilang_ramulator2_valid.py`: Main validation script
- `python/assassyn/ramulator2.py`: Python Ramulator2 wrapper implementation
- `tools/c-ramulator2-wrapper/test.cpp`: C++ implementation  
- `tools/rust-sim-runtime/tests/test_ramulator2.rs`: Rust implementation
