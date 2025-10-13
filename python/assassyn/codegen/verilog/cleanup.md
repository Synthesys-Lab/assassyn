# Verilog Cleanup and Signal Generation

This module provides post-generation cleanup utilities for Verilog code generation, handling signal generation for module interconnections, memory interfaces, and port management.

## Summary

The cleanup module is responsible for generating the final control signals and interconnections after the main Verilog code generation is complete. It handles complex signal routing for arrays, ports, modules, and memory interfaces, ensuring proper connectivity between generated modules according to the credit-based pipeline architecture.

## Exposed Interfaces

### `cleanup_post_generation`

```python
def cleanup_post_generation(dumper):
    """generating signals for connecting modules"""
```

**Explanation**

This is the main cleanup function that generates all the necessary control signals and interconnections after the primary Verilog code generation is complete. It performs the following steps:

1. **Execution Signal Generation**: Creates the `executed_wire` signal that determines when a module should execute:
   - For downstream modules: Combines execution signals from dependencies
   - For regular modules: Combines trigger conditions and wait_until predicates

2. **Finish Signal Generation**: Generates the `finish` signal by combining all finish conditions with their execution predicates

3. **SRAM Control Signal Generation**: For SRAM modules, generates memory interface signals including write enable, address selection, and data routing

4. **Array Write Signal Generation**: For each array, generates port-based write signals:
   - Groups writes by source module
   - Creates write enable signals for each port
   - Builds multiplexer chains for write data and address selection

5. **Port Signal Generation**: For FIFO ports, generates push/pop control signals:
   - Push signals: Combines push predicates with ready signals
   - Pop signals: Generates pop ready conditions

6. **Module Trigger Signal Generation**: For module calls, generates trigger signals by summing all async call predicates

7. **Exposed Signal Generation**: For exposed values, creates output ports and valid signals

**Project-specific Knowledge Required**:
- Understanding of [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) class structure
- Knowledge of [credit-based pipeline architecture](/docs/design/arch/arch.md)
- Understanding of [array write port mapping](/python/assassyn/codegen/simulator/port_mapper.md)
- Reference to [SRAM memory interface](/python/assassyn/ir/memory/sram.md)

### `generate_sram_control_signals`

```python
def generate_sram_control_signals(dumper, sram_info):
    """Generate control signals for SRAM memory interface."""
```

**Explanation**

This function generates the control signals specifically for SRAM memory interfaces. It performs the following steps:

1. **Write Signal Analysis**: Identifies all array write operations and extracts write address, data, and enable conditions
2. **Read Signal Analysis**: Identifies array read operations and extracts read addresses
3. **Address Selection**: Creates multiplexer logic to select between write and read addresses, prioritizing write addresses when writing
4. **Control Signal Generation**: Generates the following signals:
   - `mem_write_enable`: Write enable signal based on execution and write predicates
   - `mem_address`: Multiplexed address signal
   - `mem_write_data`: Write data signal
   - `mem_read_enable`: Always-enabled read signal

**Project-specific Knowledge Required**:
- Understanding of [SRAM memory model](/python/assassyn/ir/memory/sram.md)
- Knowledge of [array read/write operations](/python/assassyn/ir/expr/array.md)

### `build_mux_chain`

```python
def build_mux_chain(dumper, writes, dtype):
    """Helper to build a mux chain for write data"""
```

**Explanation**

This helper function builds a multiplexer chain for handling multiple write operations to the same array location from the same module. It creates a cascaded multiplexer structure where each write operation is conditionally selected based on its predicate.

The function:
1. Takes the first write value as the base case
2. Iteratively builds multiplexer expressions for each additional write
3. Handles type casting when necessary
4. Returns the final multiplexer expression

**Project-specific Knowledge Required**:
- Understanding of [array write operations](/python/assassyn/ir/expr/array.md)
- Knowledge of [type casting utilities](/python/assassyn/codegen/verilog/utils.md)

## Internal Helpers

The module uses several internal helper functions and imports utilities from other modules:

- `dump_type()` and `dump_type_cast()` from [utils](/python/assassyn/codegen/verilog/utils.md) for type handling
- `get_sram_info()` from [utils](/python/assassyn/codegen/verilog/utils.md) for SRAM information extraction
- `namify()` and `unwrap_operand()` from [utils](/python/assassyn/utils.md) for name generation and operand handling

The cleanup process is tightly integrated with the [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) class and is called as the final step in module generation to ensure all interconnections are properly established.
