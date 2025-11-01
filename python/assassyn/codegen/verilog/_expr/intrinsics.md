# Intrinsic Expression Generation

This module provides Verilog code generation for intrinsic operations, including logging, pure intrinsics (FIFO operations, value validation, external output reads), and block intrinsics (finish, assert, wait_until, external instantiation).

## Summary

The intrinsic expression generation module handles the conversion of Assassyn intrinsic operations into Verilog code. It manages logging operations for debugging, pure intrinsics for FIFO and value operations, and block intrinsics for control flow operations in the credit-based pipeline architecture.

## Exposed Interfaces

### `codegen_log`

```python
def codegen_log(dumper, expr: Log) -> Optional[str]:
    """Generate code for log operations."""
```

**Explanation**

This function generates Python testbench code for logging operations, which are used for debugging and monitoring during simulation. It performs the following steps:

1. **Format String Processing**: Extracts the format string from the first operand and processes it using Python's `Formatter` class. Placeholder conversions such as `:?` are mapped to Python's `!r` conversions to match the DSL semantics.
2. **Argument Processing**: For each argument after the format string:
   - Assumes the metadata pre-pass has already recorded any non-constant operands that need to surface as module outputs
   - Generates sanitized testbench signal references (removing `self.` prefixes and replacing punctuation) for those values
   - Handles signed integer conversion for proper display
3. **Condition Generation**: Builds complex conditions based on:
   - Current execution predicate
   - Condition stack (conditional blocks), translating them into DUT-visible signals; cycle-based predicates come from `current_cycle()` within these conditions
   - Valid signals for operands tracked by metadata
4. **Log Generation**: Creates Python print statements annotated with line information, module names, and formatted cycle counts so the Cocotb testbench can produce readable diagnostics.

The function generates testbench code that:
- Accesses module signals through the DUT (Device Under Test) hierarchy
- Handles signed integer display by checking the sign bit
- Creates conditional logging based on execution context
- Includes line information, cycle count, and module name in the output

**Project-specific Knowledge Required**:
- Understanding of [log operations](/python/assassyn/ir/expr/intrinsic.md)
- Knowledge of [testbench generation](/python/assassyn/codegen/verilog/testbench.md)
- Understanding of [metadata-driven exposure handling](/python/assassyn/codegen/verilog/metadata.md)
- Reference to [condition stack handling](/python/assassyn/codegen/verilog/design.md)

### `codegen_pure_intrinsic`

```python
def codegen_pure_intrinsic(dumper, expr: PureIntrinsic) -> Optional[str]:
    """Generate code for pure intrinsic operations."""
```

**Explanation**

This function generates Verilog code for pure intrinsic operations, which are side-effect-free operations that can be used in expressions. It handles the following intrinsic types:

1. **FIFO_VALID**: Returns the valid signal of a FIFO
   - Generates `self.fifo_name_valid` signal reference
   - Used to check if FIFO has valid data available

2. **FIFO_PEEK**: Returns the data at the head of a FIFO without consuming it
   - Generates `self.fifo_name` signal reference
   - Used to examine FIFO data without popping

3. **VALUE_VALID**: Returns the valid signal for a value expression
   - For external values: generates external port valid signal
   - For internal values: generates `self.executed` signal
   - Used to check if a value is valid in the current execution context

4. **EXTERNAL_OUTPUT_READ**: Reads a port from an `ExternalIntrinsic`
   - Unwraps the intrinsic operand so the dumper can associate it with its owning module
   - Normalises cross-module accesses into a stable `(instance, port, index)` key that later passes use to declare shared wires exactly once
   - For cross-module reads, records the consumer/producer relationship and returns the exposed input (`self.<producer>_<value>`)
   - For local reads, ensures the external wrapper is instantiated and cached in `external_instance_names`, then emits either the raw signal or an indexed access (with index-0 treated as the scalar case)

The function handles FIFO operations by generating appropriate signal references; metadata collected during analysis ensures any required values are surfaced.

**Project-specific Knowledge Required**:
- Understanding of [pure intrinsic operations](/python/assassyn/ir/expr/intrinsic.md)
- Knowledge of [FIFO operations](/python/assassyn/ir/expr/array.md)
- Understanding of [external port handling](/python/assassyn/codegen/verilog/design.md)
- Awareness of the cross-module wiring pipeline documented in [system generation](/python/assassyn/codegen/verilog/system.md) and [cleanup](/python/assassyn/codegen/verilog/cleanup.md)
- Reference to [right-hand value generation](/python/assassyn/codegen/verilog/rval.md)

### `codegen_intrinsic`

```python
def codegen_intrinsic(dumper, expr: Intrinsic) -> Optional[str]:
    """Generate code for intrinsic operations."""
```

**Explanation**

This function generates Verilog code for block intrinsic operations, which are control flow operations that affect module execution. It handles the following intrinsic types:

1. **FINISH**: Signals that the module should finish execution
   - Emits no Verilog directly; metadata analysis has already recorded the intrinsic and its predicate in `ModuleMetadata.finish_sites`
   - Used to implement early termination of module execution
   - The cleanup phase formats each stored predicate, gates it with `executed_wire`, and OR-reduces the results

2. **ASSERT**: Generates assertion code for verification
   - Metadata analysis has already marked the expression for exposure when required
   - Used for formal verification and simulation debugging

3. **WAIT_UNTIL**: Implements the credit-based pipeline wait mechanism
   - Sets `dumper.wait_until` to the condition expression
   - Used to control module execution timing in the credit-based architecture
   - The cleanup phase incorporates this into the execution signal

4. **EXTERNAL_INSTANTIATE / ExternalIntrinsic**: Creates and wires external modules in-line
   - `ExternalIntrinsic` instances are handled before the opcode switch, generating calls to `<wrapper>::new()` and wiring all inputs
   - Updates the dumper's bookkeeping (`external_instance_names`, `external_wrapper_names`, `external_output_exposures`) while consulting the shared `ExternalRegistry` for instance owners and cross-module consumers

The function integrates with the credit-based pipeline architecture by managing execution conditions and finish signals.

**Project-specific Knowledge Required**:
- Understanding of [intrinsic operations](/python/assassyn/ir/expr/intrinsic.md)
- Knowledge of [credit-based pipeline architecture](/docs/design/arch/arch.md)
- Understanding of [execution control](/python/assassyn/codegen/verilog/cleanup.md)
- Reference to [condition handling](/python/assassyn/codegen/verilog/design.md)

## Internal Helpers

The module uses several utility functions:

- `dump_rval()` from [rval module](/python/assassyn/codegen/verilog/rval.md) for generating signal references
- `unwrap_operand()` and `namify()` from [utils module](/python/assassyn/utils.md) for operand processing and name generation
- `get_pred()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for getting current execution predicate
- `external_instance_names` / `external_wrapper_names` maps on the dumper to coordinate `ExternalIntrinsic` handling across passes

The intrinsic expression generation is integrated into the main expression dispatch system through the [__init__.py](/python/assassyn/codegen/verilog/_expr/__init__.md) module, which routes different expression types to their appropriate code generation functions.

**Project-specific Knowledge Required**:
- Understanding of [expression dispatch system](/python/assassyn/codegen/verilog/_expr/__init__.md)
- Knowledge of [CIRCTDumper integration](/python/assassyn/codegen/verilog/design.md)
- Reference to [intrinsic expression types](/python/assassyn/ir/expr/intrinsic.md)
- Understanding of [testbench integration](/python/assassyn/codegen/verilog/testbench.md)
