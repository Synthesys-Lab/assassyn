# Port Mapper

This module manages compile-time assignment of port indices for multi-port array writes in the simulator code generator. It enables multiple modules to write to the same array by assigning unique port indices to each writer, allowing for efficient multi-ported register array simulation.

## Design Documents

- [Simulator Design](../../../docs/design/internal/simulator.md) - Simulator design and code generation
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system
- [Architecture Overview](../../../docs/design/arch/arch.md) - Overall system architecture

## Related Modules

- [Simulator Generation](./simulator.md) - Core simulator generation logic
- [Simulator Elaboration](./elaborate.md) - Main entry point for simulator generation
- [Module Generation](./modules.md) - Module-to-Rust translation
- [Node Dumper](./node_dumper.md) - IR node reference generation

## Section 0. Summary

The port mapper module implements a global singleton pattern for managing port indices across the entire code generation process. It provides compile-time port allocation for multi-port array writes, ensuring that each module gets a unique port index for writing to arrays.

**DRAM Callback Status:** The port mapper handles DRAM callback port assignment:

1. **DRAM_CALLBACK Port**: Special port type for DRAM callback operations
2. **Port Assignment**: DRAM callbacks are assigned unique port indices
3. **Integration**: DRAM callbacks are integrated with the port mapping system
4. **Status**: DRAM callback port assignment is implemented and functional

**Thread Safety Notes:** The port mapper implements thread safety considerations:

1. **Global Singleton**: PortIndexManager is a global singleton accessed across threads
2. **Thread Safety**: Port assignment operations are thread-safe
3. **Concurrent Access**: Multiple threads can safely access port indices
4. **State Management**: Global state is managed safely across thread boundaries

**Usage Pattern Documentation:** The port mapper follows a three-phase usage pattern:

1. **Reset Phase**: Reset the port manager to start with clean state
2. **Analysis Phase**: Analyze the system and register all port assignments
3. **Code Generation Phase**: Use port indices during code generation

**Global State Management:** The port mapper manages global state through:

1. **Singleton Pattern**: Single instance shared across the entire system
2. **Port Map**: Maps (array_name, module_name) to port indices
3. **Index Tracking**: Tracks next available port index for each array
4. **Port Counts**: Maintains total port count for each array

```python
class PortIndexManager:
    """Manages port index assignment for arrays during code generation."""
    
    def __init__(self):
        # Map: (array_name, module_name) -> port_index
        self.port_map = {}
        # Map: array_name -> next available index
        self.next_index = defaultdict(int)
        # Map: array_name -> total port count
        self.port_counts = defaultdict(int)
```

#### get_or_assign_port

```python
def get_or_assign_port(self, array_name: str, module_name: str) -> int:
    """Get or assign a port index for a module writing to an array.

    Args:
        array_name: Name of the array being written to
        module_name: Name of the module performing the write

    Returns:
        Port index (0, 1, 2, ...) for this array-module combination
    """
```

**Explanation:**
This function implements the core port assignment logic for multi-ported array writes. When a module needs to write to an array, it calls this function to get a unique port index. The function maintains a mapping from `(array_name, module_name)` tuples to port indices, ensuring that each module gets a consistent port index for each array it writes to. If a new combination is encountered, it assigns the next available sequential port index for that array.

The port assignment is deterministic and happens at compile time during the analysis phase of code generation. This allows the generated Rust code to use compile-time constants for port indices, enabling optimal performance in the simulator.

#### get_port_count

```python
def get_port_count(self, array_name: str) -> int:
    """Get the total number of ports needed for an array.

    Args:
        array_name: Name of the array

    Returns:
        Total number of ports needed (minimum 1)
    """
```

**Explanation:**
This function returns the total number of ports that need to be pre-allocated for an array during simulator initialization. It ensures a minimum of 1 port even if no writes were detected during analysis, which is necessary for arrays that might be written to by DRAM callbacks or other runtime mechanisms not visible during static analysis.

### get_port_manager

```python
def get_port_manager():
    """Get the global port manager instance.

    Returns:
        The global PortIndexManager instance
    """
```

**Explanation:**
This function implements the singleton pattern for the port manager. It ensures that there is only one instance of `PortIndexManager` throughout the compilation process, maintaining consistency in port assignments across different phases of code generation (analysis, array initialization, and write operation generation).

### reset_port_manager

```python
def reset_port_manager():
    """Reset the port manager (useful for tests and new compilations)."""
```

**Explanation:**
This function resets the global port manager instance, creating a fresh `PortIndexManager` with empty state. It is called at the beginning of each compilation in `elaborate.py` to ensure a clean slate for port assignments. This is particularly important for testing scenarios where multiple compilations might be performed in the same process.

## Section 2. Internal Helpers

### _port_manager

```python
_port_manager = None  # Global singleton instance
```

**Explanation:**
This module-level variable stores the global singleton instance of `PortIndexManager`. It is initialized to `None` and lazily created when `get_port_manager()` is first called. The singleton pattern ensures consistent port assignments throughout the compilation process.

## Usage in Code Generation Pipeline

The port mapper is used in three distinct phases of the simulator code generation:

### 1. Reset Phase (elaborate.py)
At the start of each compilation, the port manager is reset to ensure clean state:
```python
from .port_mapper import reset_port_manager
reset_port_manager()
```

### 2. Analysis Phase (simulator.py)
During system analysis, a visitor pattern is used to scan all IR nodes and register array writes:
```python
from .port_mapper import get_port_manager
from ...ir.expr.array import ArrayWrite

manager = get_port_manager()
class PortRegistrationVisitor(Visitor):
    def visit_expr(self, node):
        if isinstance(node, ArrayWrite):
            array_name = namify(node.array.name)
            writer_name = namify(node.module.name)
            manager.get_or_assign_port(array_name, writer_name)
```

### 3. Code Generation Phase
**Array Initialization:** Arrays are initialized with the correct number of ports:
```python
num_ports = port_manager.get_port_count(name)
fd.write(f"{name}: Array::new_with_ports({array.size}, {num_ports}),")
```

**Write Operations:** Each write operation uses its assigned port index:
```python
port_idx = manager.get_or_assign_port(array_name, module_writer)
return f"sim.{array_name}.write({port_idx}, write);"
```

## Example: Multi-Port Array Write

Consider a system where multiple modules write to the same array:

```python
array = RegArray(Int(32), 10)

class ModuleA(Module):
    def build(self, arr: Array):
        (arr & self)[0] <= value_a  # Gets port 0
        
class ModuleB(Module):
    def build(self, arr: Array):
        (arr & self)[1] <= value_b  # Gets port 1
        
class ModuleC(Module):
    def build(self, arr: Array):
        (arr & self)[2] <= value_c  # Gets port 2
```

The port mapper assigns:
- `(array, ModuleA)` → port 0
- `(array, ModuleB)` → port 1  
- `(array, ModuleC)` → port 2

Generated Rust code:
```rust
// Array initialization with 3 ports
pub array: Array<i32>,
// In constructor:
array: Array::new_with_ports(10, 3),

// Write operations use assigned port indices
sim.array.write(0, write);  // ModuleA
sim.array.write(1, write);  // ModuleB
sim.array.write(2, write);  // ModuleC
```

## Integration with Simulator Runtime

The port mapper integrates with the Rust simulator runtime's `Array<T>` type, which supports multi-ported writes through the `write(port_index, write_data)` method. Each port index corresponds to a separate write port that can be used concurrently by different modules, enabling true multi-ported register array behavior in the simulator.

The port assignment happens at compile time, allowing the generated code to use compile-time constants for port indices, which provides optimal performance compared to runtime port resolution.

