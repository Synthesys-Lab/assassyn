# Module Port Generation

This module provides utilities for generating Verilog module port declarations, handling the complex interface requirements of the credit-based pipeline architecture including clock/reset signals, execution control, FIFO interfaces, array connections, and external module integration.

## Summary

The module port generation utilities handle the creation of comprehensive module interfaces for Verilog modules. They manage the complex port requirements of the credit-based pipeline architecture, including execution control signals, FIFO handshaking, array read/write interfaces, external module connections, and special handling for downstream modules and SRAM modules.

## Exposed Interfaces

### `generate_module_ports`

```python
def generate_module_ports(dumper, node: Module, is_downstream: bool, is_sram: bool,
                          is_driver: bool, pushes: List, calls: List) -> None:
    """Generate port declarations for a module.

    Args:
        dumper: The CIRCTDumper instance
        node: The module to generate ports for
        is_downstream: Whether this is a downstream module
        is_sram: Whether this is an SRAM module
        is_driver: Whether this module is a driver
        pushes: List of FIFOPush expressions
        calls: List of AsyncCall expressions
    """
```

**Explanation**

This function generates comprehensive port declarations for Verilog modules based on their role in the credit-based pipeline architecture. It performs the following steps:

1. **Standard Ports**: Generates common ports for all modules:
   - `clk = Clock()`: System clock
   - `rst = Reset()`: System reset
   - `executed = Output(Bits(1))`: Module execution status
   - `cycle_count = Input(UInt(64))`: Global cycle counter
   - `finish = Output(Bits(1))`: Module finish signal

2. **Downstream Module Ports**: For downstream modules, generates:
   - **Dependency Signals**: Input ports for each dependency module's execution status
   - **External Value Ports**: Input ports for external values with valid signals
   - **SRAM Interface**: Special memory interface ports for SRAM modules

3. **Pipeline Module Ports**: For regular pipeline modules, generates:
   - **Trigger Counter Interface**: Input port for trigger counter validation
   - **Port Interfaces**: Input ports for FIFO ports with valid signals and pop ready outputs

4. **FIFO Handshake Ports**: Generates handshake interfaces for:
   - **Push Operations**: Input ready signals and output valid/data signals
   - **Call Operations**: Input ready signals and output trigger signals

5. **Array Interface Ports**: Generates multi-port array interfaces:
   - **Read Ports**: Input ports for array data
   - **Write Ports**: Output ports for write enable, data, and address signals

6. **Exposed Ports**: Adds any additional ports registered through the expose mechanism

The function handles different module types with specific logic:

- **Downstream Modules**: Generate dependency and external value ports
- **SRAM Modules**: Generate memory interface ports
- **Driver Modules**: Generate trigger counter interfaces
- **Regular Modules**: Generate FIFO port and trigger interfaces

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
