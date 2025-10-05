# Port Mapper

The port mapper manages compile-time assignment of port indices for multi-port array writes in the simulator code generator.

## Problem Statement

In Assassyn, multiple modules can write to the same array concurrently. To support this safely, each writer needs a dedicated write port. Previously, port IDs were assigned using runtime Python object IDs (via `id(module)`), which resulted in:
- Large, unpredictable hash values as port IDs
- HashMap lookup overhead in the Rust runtime
- No compile-time optimization opportunities

## Solution

The port mapper provides **compile-time port index assignment**:
1. Analyze the entire system during elaboration to find all array writes
2. Assign sequential port indices (0, 1, 2, ...) to each module-array pair
3. Pre-allocate the exact number of ports needed for each array
4. Use Vec-based storage with direct indexing in the Rust runtime

## Architecture

### PortIndexManager

The core class that tracks port assignments:

```python
class PortIndexManager:
    def __init__(self):
        # Map: (array_name, module_name) -> port_index
        self.port_map = {}
        # Map: array_name -> next available index
        self.next_index = defaultdict(int)
        # Map: array_name -> total port count
        self.port_counts = defaultdict(int)
```

**Key methods:**
- `get_or_assign_port(array_name, module_name)`: Returns a port index for a specific module-array combination. Assigns a new sequential index if not already assigned.
- `get_port_count(array_name)`: Returns the total number of ports needed for an array (minimum 1).

### Global Singleton

The port manager uses a global singleton pattern to ensure consistent port assignment throughout the compilation:

```python
_port_manager = None  # Global instance

def get_port_manager():
    """Get or create the global port manager."""
    global _port_manager
    if _port_manager is None:
        _port_manager = PortIndexManager()
    return _port_manager

def reset_port_manager():
    """Reset for a new compilation."""
    global _port_manager
    _port_manager = PortIndexManager()
```

## Usage Flow

### 1. Reset Phase (elaborate.py)

At the start of each compilation:
```python
from .port_mapper import reset_port_manager

def elaborate(sys, **config):
    reset_port_manager()  # Clean slate for new compilation
    # ... continue elaboration
```

### 2. Analysis Phase (simulator.py)

Scan the system to register all array writes:
```python
from .port_mapper import get_port_manager

def analyze_and_register_ports(sys):
    manager = get_port_manager()
    
    class PortRegistrationVisitor(Visitor):
        def visit_expr(self, node):
            if isinstance(node, ArrayWrite):
                array_name = namify(node.array.name)
                writer_name = namify(node.module.name)
                manager.get_or_assign_port(array_name, writer_name)
    
    visitor = PortRegistrationVisitor()
    visitor.visit_system(sys)
```

### 3. Code Generation Phase

**Array initialization (simulator.py):**
```python
manager = get_port_manager()
num_ports = manager.get_port_count(array_name)
# Generate: Array::new_with_ports(size, num_ports)
```

**Write operations (array.py):**
```python
manager = get_port_manager()
port_idx = manager.get_or_assign_port(array_name, module_writer)
# Generate: sim.array.write(port_idx, write)
```

## Example

Given this system:
```python
array = RegArray(Int(32), 10)

module ModuleA:
    array[0] <= value_a  # Gets port 0
    
module ModuleB:
    array[1] <= value_b  # Gets port 1
    
module ModuleC:
    array[2] <= value_c  # Gets port 2
```

The port mapper assigns:
- `(array, ModuleA)` → port 0
- `(array, ModuleB)` → port 1  
- `(array, ModuleC)` → port 2

Generated Rust code:
```rust
// Initialization
pub array: Array<i32>,
// In constructor:
array: Array::new_with_ports(10, 3),  // 3 ports pre-allocated

// Write operations
sim.array.write(0, write);  // ModuleA
sim.array.write(1, write);  // ModuleB
sim.array.write(2, write);  // ModuleC
```

## Benefits

1. **Optimal Performance**
   - Sequential port IDs (0, 1, 2, ...) enable Vec-based storage
   - Direct O(1) indexing instead of HashMap lookup
   - Better cache locality with contiguous memory

2. **Memory Efficiency**
   - Pre-allocate exact number of ports needed
   - No hash table overhead
   - Compact integer indices instead of large hash values

3. **Deterministic**
   - Same system always generates same port assignments
   - Reproducible builds
   - Easier debugging

4. **Compile-Time Optimization**
   - All port assignments known before code generation
   - Enables future optimizations (e.g., conflict detection)

## Special Cases

### DRAM Writes

DRAM callback writes use a reserved port name:
```python
manager.get_or_assign_port(array_name, "DRAM_CALLBACK")
```

This ensures DRAM memory interface callbacks get dedicated ports separate from module writes.

### Minimum Port Count

Arrays always have at least 1 port, even if no writes are detected:
```python
def get_port_count(self, array_name: str) -> int:
    return max(1, self.port_counts[array_name])
```

This maintains backwards compatibility and prevents zero-sized port vectors.

## Integration with Rust Runtime

The port mapper works in conjunction with the Rust runtime's Vec-based port storage (see `xeq.md`):

**Python (compile-time):**
- Assigns sequential port indices
- Determines total port count
- Generates code with correct port IDs

**Rust (runtime):**
- Pre-allocates Vec with exact port count
- Uses direct indexing for writes
- Maintains write ordering and conflict resolution

