# Simulator Generation

This module generates Rust-based simulators from Assassyn systems. It implements the credit-based pipeline architecture described in the [simulator design document](../../../docs/design/internal/simulator.md) by translating Assassyn operations into corresponding Rust operations with proper handling of register writes, stage registers, and asynchronous calls.

## Section 0. Summary

The simulator generation process creates a complete Rust project that faithfully executes the high-level execution model of Assassyn hardware designs. The generated simulator implements the credit-based pipeline architecture where pipeline stages communicate through event queues and FIFOs, while downstream modules execute as pure combinational logic driven by upstream stage triggers. The simulator handles register arrays with port-based arbitration, DRAM interfaces for memory simulation, and maintains proper timing semantics through a half-cycle tick mechanism.

## Section 1. Exposed Interfaces

### analyze_and_register_ports

```python
def analyze_and_register_ports(sys):
    """Analyze system and register all array write ports and DRAM modules.

    This function scans the entire system to find all array writes and DRAM modules,
    registers them with the port manager, ensuring each writer gets a unique
    port index for compile-time port allocation.

    Args:
        sys: The Assassyn system builder

    Returns:
        Tuple of (port_manager, dram_modules) where dram_modules is a list of DRAM instances
    """
```

**Explanation:**

This function performs a comprehensive analysis of the Assassyn system to prepare for simulator generation. It uses a visitor pattern to traverse all expressions and modules in the system, identifying two key components:

1. **Array Write Port Registration**: For each `ArrayWrite` expression found, it registers the array-module combination with the global port manager. This ensures that each module writing to an array gets a unique port index, enabling compile-time port allocation for optimal performance in the generated Rust code.

2. **DRAM Module Collection**: It collects all `DRAM` module instances in the system, which will be used later to generate per-DRAM memory interfaces in the simulator.

The function returns both the port manager (which contains the port assignments) and the list of DRAM modules, providing the necessary information for generating the simulator struct with proper port allocation and memory interface setup.

### dump_simulator

```python
def dump_simulator(sys: SysBuilder, config, fd):
    """Generate the simulator module.

    This matches the Rust function in src/backend/simulator/elaborate.rs

    Args:
        sys: The Assassyn system builder
        config: Configuration dictionary with the following keys:
            - idle_threshold: Idle threshold for the simulator
            - sim_threshold: Maximum number of simulation cycles
            - random: Whether to randomize module execution order
            - resource_base: Path to resource files
            - fifo_depth: Default FIFO depth
        fd: File descriptor to write to
    """
```

**Explanation:**

This function generates the complete Rust simulator implementation by writing to the provided file descriptor. The generation process follows these steps:

1. **System Analysis**: First calls `analyze_and_register_ports` to determine port requirements and collect DRAM modules.

2. **Import Generation**: Writes necessary Rust imports including the `sim_runtime` crate, collections, and platform-specific dependencies.

3. **Simulator Struct Generation**: Creates the main `Simulator` struct with fields for:
   - Global timestamp and request mapping table
   - Per-DRAM memory interfaces and response buffers
   - Register arrays with pre-allocated ports
   - Module trigger flags and event queues
   - FIFO fields for stage ports
   - Exposed value tracking for downstream modules

4. **Implementation Generation**: Generates the `Simulator` implementation with methods for:
   - Constructor that initializes all fields and memory interfaces
   - Event validity checking for pipeline stage triggering
   - Downstream reset for combinational logic
   - Register ticking for half-cycle semantics
   - DRAM response reset

5. **Module Simulation Functions**: For each module, generates a `simulate_<module_name>` function that:
   - For pipeline stages: Checks event validity and pops events on successful execution
   - For downstream modules: Checks upstream trigger conditions
   - Calls the corresponding module implementation from the generated modules

6. **Main Simulation Loop**: Generates the `simulate()` function that:
   - Initializes the simulator and DRAM interfaces
   - Sets up initial events for Driver and Testbench modules
   - Runs the main simulation loop with proper timing and idle detection
   - Handles register ticking and memory interface updates

The generated simulator implements the credit-based pipeline architecture described in the [simulator design document](../../../docs/design/internal/simulator.md), with proper handling of asynchronous communication, register arrays, and memory interfaces.

## Section 2. Internal Helpers

### PortRegistrationVisitor

```python
class PortRegistrationVisitor(Visitor):
    """Visitor that registers array write ports and collects DRAM modules."""
```

**Explanation:**

This internal visitor class is used by `analyze_and_register_ports` to traverse the Assassyn system and perform the necessary analysis. It implements the visitor pattern to systematically examine all expressions and modules:

- **Expression Visiting**: When visiting expressions, it specifically looks for `ArrayWrite` instances and registers them with the port manager using the array name and module name as keys.

- **Module Visiting**: When visiting modules, it identifies `DRAM` instances and adds them to the collection for later use in memory interface generation.

The visitor pattern ensures that all array writes and DRAM modules are discovered regardless of their location in the system hierarchy, providing comprehensive coverage for port allocation and memory interface setup.

### Configuration Parameters

The `dump_simulator` function accepts several configuration parameters that control the generated simulator behavior:

- **idle_threshold**: Controls when the simulation stops due to inactivity (default: 5)
- **sim_threshold**: Maximum number of simulation cycles (default: 100)  
- **random**: Whether to randomize module execution order for testing
- **resource_base**: Base path for resource files (SRAM initialization)
- **fifo_depth**: Default depth for FIFO implementations

These parameters allow fine-tuning of the simulator behavior for different testing scenarios and performance requirements.

### Memory Interface Management

The simulator generation creates per-DRAM memory interfaces rather than a single global interface. This approach provides better isolation and callback management for systems with multiple DRAM modules. Each DRAM module gets:

- A dedicated `MemoryInterface` instance (`mi_<dram_name>`)
- A response buffer (`<dram_name>_response`) for handling memory responses
- Proper initialization with configuration files
- Individual ticking in the simulation loop

This design matches the requirements described in the [simulator design document](../../../docs/design/internal/simulator.md) for handling multiple memory interfaces in complex systems.
