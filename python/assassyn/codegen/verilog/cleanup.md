# Verilog Cleanup and Signal Generation

This module provides post-generation cleanup utilities for Verilog code generation, handling signal generation for module interconnections, memory interfaces, port management, and the extra bookkeeping required to surface values for external SystemVerilog FFIs.

## Summary

The cleanup module is responsible for generating the final control signals and interconnections after the main Verilog code generation is complete. It handles complex signal routing for arrays, ports, modules, and memory interfaces, ensuring proper connectivity between generated modules according to the credit-based pipeline architecture. As part of the external module flow, it also materialises producer-side `expose_*`/`valid_*` ports for any external register outputs that are consumed by another module, so cross-module reads can be wired up without duplicating logic.

## Exposed Interfaces

### `cleanup_post_generation`

```python
def cleanup_post_generation(dumper):
    """generating signals for connecting modules"""
```

**Explanation**

This is the main cleanup function that generates all the necessary control signals and interconnections after the primary Verilog code generation is complete. It performs the following steps:

1. **Execution Signal Generation**: Creates the `executed_wire` signal that determines when a module should execute:
   - For downstream modules: Gathers upstream dependencies from `dumper.downstream_dependencies` and ORs their `executed` flags.
   - For regular modules: ANDs the trigger-counter pop-valid input with any active `wait_until` predicate recorded during expression lowering.

2. **Finish Signal Generation**: Reduces every `(predicate, exec_signal)` pair queued in `dumper.finish_conditions` into the `self.finish` output.

3. **SRAM Control Signal Generation**: When the current module wraps an SRAM payload (detected via `array.is_payload(sram_instance)`), `generate_sram_control_signals` derives write enables, addresses, and data from the exposed array accesses, producing the handshakes expected by the memory blackbox.

4. **Array Write Signal Generation**: For each array exposed through `dumper._exposes`:
   - Filters out arrays whose owner is a memory instance and satisfy `array.is_payload(owner)`, because those are handled by dedicated memory logic.
   - Groups writes by source module and maps them onto the precomputed port indices stored in the `ArrayMetadataRegistry`.
   - Emits write-enable, write-data, and write-index signals per port. Multi-writer modules use `build_mux_chain` to pick the correct payload.

5. **FIFO Signal Generation**: For every port exposure:
   - Aggregates push predicates, applies backpressure via the parent module's `fifo_*_push_ready` signals, and emits valid/data assignments.
   - Aggregates pop predicates and produces the module-local `*_pop_ready` backpressure signal.

6. **Module Trigger Signal Generation**: When async calls target another module, sums all predicates (each converted to an 8-bit increment) and routes the result into `<callee>_trigger`.

7. **External Exposure Generation**: For every expression that must leave the module:
   - Appends `expose_<name>`/`valid_<name>` port declarations to `dumper.exposed_ports_to_add`.
   - Emits assignments that drive the value and its validity.
   - Skips raw objects that are bridged through dedicated external wiring handled elsewhere.
   - Emits additional `expose_<instance>_<port>` / `valid_<instance>_<port>` pairs for every external register output that is consumed by another module, using the cross-module metadata recorded earlier in the pipeline.

8. **Bookkeeping**: Records `self.executed = executed_wire` as the last assignment, ensuring downstream consumers and the top-level harness can observe the execution result.

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

This helper function builds a multiplexer chain for handling multiple write operations to the same array location from the same module. It creates a cascaded multiplexer structure where each write operation is conditionally selected based on its predicate. Type mismatches are reconciled through `dump_type_cast` so the generated hardware preserves bit widths.

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
