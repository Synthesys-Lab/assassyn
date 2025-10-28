# Module Port Generation

This module provides utilities for generating Verilog module port declarations, handling the complex interface requirements of the credit-based pipeline architecture including clock/reset signals, execution control, FIFO interfaces, array connections, and external module integration.

## Summary

The module port generation utilities handle the creation of comprehensive module interfaces for Verilog modules. They manage the complex port requirements of the credit-based pipeline architecture, including execution control signals, FIFO handshaking, array read/write interfaces, external module connections, and special handling for downstream modules and SRAM modules.

## Exposed Interfaces

### `generate_module_ports`

```python
def generate_module_ports(dumper, node: Module, is_downstream: bool, is_sram: bool,
                          is_driver: bool, pushes: List, calls: List, pops: List) -> None:
    """Generate port declarations for a module.

    Args:
        dumper: The CIRCTDumper instance
        node: The module to generate ports for
        is_downstream: Whether this is a downstream module
        is_sram: Whether this is an SRAM module
        is_driver: Whether this module is a driver
        pushes: List of FIFOPush expressions
        calls: List of AsyncCall expressions
        pops: List of FIFOPop expressions
    """
```

**Explanation**

This function generates comprehensive port declarations for Verilog modules based on their role in the credit-based pipeline architecture. It performs the following steps:

1. **Standard Ports**: Emits the common Assassyn ports (`clk`, `rst`, `executed`, `cycle_count`, `finish`).

2. **Downstream Module Ports**: For downstream modules, generates:
   - Dependency inputs for each upstream module recorded in `dumper.downstream_dependencies`.
   - SRAM interface wires when the downstream is an SRAM wrapper (`mem_dataout`, `mem_address`, `mem_write_data`, `mem_write_enable`, `mem_read_enable`).

3. **Pipeline Module Ports**: For regular pipeline modules (drivers or async callees), adds the trigger-counter backpressure input (`trigger_counter_pop_valid`).

4. **External Value Inputs**: Declares two categories of inbound external data:
   - Entries from `dumper.cross_module_external_reads` ensure consumers that read another module’s external register output get `producer_value`/`producer_value_valid` inputs even if the value never appears in `node.externals`.
   - Direct externals (`node.externals`) still emit `<producer>_<value>` and `<producer>_<value>_valid` inputs for expressions that originate elsewhere (skipping bindings, constants, and the `ExternalIntrinsic` handles themselves). The implementation now resolves the producer by first checking whether `expr.parent` is already a module—reflecting the block-free IR—before falling back to legacy `.module` lookups.

5. **FIFO Handshake Ports**:
   - For pipeline modules, declares FIFO inputs (`port`, `port_valid`) and optional `port_pop_ready` outputs when the module pops from the FIFO, determined by `pops` metadata (`{p.fifo for p in pops}`).
   - Adds ready inputs for FIFO pushes and trigger counter deltas, skipping handshake ports when the producer/consumer is a pure external stub.

6. **Output Handshakes**: Declares `<callee>_<fifo>_push_valid/data` outputs and `<callee>_trigger` outputs for each async call target, again skipping external-only callees so we do not generate unused ports.

7. **Array Interfaces**: For every array listed in `dumper.array_users[node]`, creates inputs for the array value (`_q_in`) and, when the current module writes to the array, outputs for the per-port write enable/data/index signals. Port indices come from `dumper.array_write_port_mapping`.

8. **Exposed Ports**: Appends any additional port declarations recorded in `dumper.exposed_ports_to_add` during expression traversal (e.g. `expose_*` and `valid_*` ports).

The function accounts for several module categories:

- **Downstream Modules**: Receive upstream execution flags and, if applicable, SRAM memory interfaces.
- **Pipeline Modules**: Get trigger-counter pop-valid signals, FIFO inputs, and handshake ports.
- **SRAM Modules**: Extend downstream behaviour with memory-specific ports.
- **External Stubs**: Skipped from handshake port generation so we do not expose meaningless signals for modules implemented outside of Python.

**Project-specific Knowledge Required**:
- Understanding of [credit-based pipeline architecture](/docs/design/arch/arch.md)
- Knowledge of [module types](/python/assassyn/ir/module/module.md)
- Understanding of [FIFO operations](/python/assassyn/ir/expr/array.md)
- Reference to [array management](/python/assassyn/codegen/verilog/cleanup.md)
- Knowledge of [external module integration](/python/assassyn/ir/module/external.md)

## Internal Helpers

The function uses several utility functions and data structures:

- `dump_type()` from [utils module](/python/assassyn/codegen/verilog/utils.md) for type declarations
- `get_sram_info()` from [utils module](/python/assassyn/codegen/verilog/utils.md) for SRAM information
- `namify()` and `unwrap_operand()` from [utils module](/python/assassyn/utils.md) for name generation
- `get_external_port_name()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for external port naming
- `_walk_expressions()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for expression traversal
- `_is_external_module()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for external module detection

The function integrates with the CIRCTDumper's state management:
- `downstream_dependencies`: Maps downstream modules to their dependencies
- `array_users`: Maps arrays to modules that use them
- `array_write_port_mapping`: Maps arrays to write port assignments
- `exposed_ports_to_add`: Additional ports registered through expose mechanism

**Project-specific Knowledge Required**:
- Understanding of [CIRCTDumper integration](/python/assassyn/codegen/verilog/design.md)
- Knowledge of [multi-port array architecture](/docs/design/arch/arch.md)
- Understanding of [FIFO handshaking protocols](/docs/design/internal/pipeline.md)
- Reference to [external module system](/python/assassyn/ir/module/external.md)
- Knowledge of [SRAM memory interface](/python/assassyn/ir/memory/sram.md)

The module port generation is a critical component of the Verilog code generation process, ensuring that all modules have the proper interfaces to participate in the credit-based pipeline architecture and communicate with other modules through FIFOs, arrays, and external connections.
