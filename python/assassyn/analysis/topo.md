# Topological Analysis

This module provides topological analysis functionality for Assassyn systems, implementing the analysis framework described in the [architecture overview](../../../docs/design/arch/arch.md).

## Design Documents

- [Architecture Overview](../../../docs/design/arch/arch.md) - Overall system architecture
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system
- [Module Design](../../../docs/design/internal/module.md) - Module design and lifecycle

## Related Modules

- [Analysis Module](./__init__.py) - Analysis module initialization
- [External Usage Analysis](./external_usage.md) - External usage analysis
- [Module Analysis](../ir/module/) - Module analysis and processing

## Section 0. Summary

The topological analysis module provides functions for analyzing dependencies between modules in Assassyn systems. It implements topological sorting algorithms to determine the correct execution order of modules based on their dependencies.

**Implementation Location:** The topological analysis functions are implemented in `topo.py`:

1. **`topo_downstream_modules`**: Topologically sorts downstream modules based on dependencies
2. **`get_upstreams`**: Identifies upstream modules that a given module depends on
3. **Usage**: Used by the simulator generation process and verilog code generation
4. **Integration**: Integrated with the overall analysis framework

**Function Purpose:** The analysis module provides:

1. **Dependency Analysis**: Analyzes dependencies between downstream modules
2. **Graph Construction**: Builds a dependency graph using adjacency lists
3. **Topological Sort**: Performs topological sorting using Kahn's algorithm
4. **Cycle Detection**: Detects circular dependencies and raises ValueError
5. **External Module Handling**: Rekindles dependencies purely from `module.externals`, so no bespoke wire-assignment heuristics are needed.

## Section 1. Exposed Interfaces

### topo_downstream_modules

```python
def topo_downstream_modules(sys):
    """Topologically sort downstream modules based on their dependencies."""
```

**Explanation:**

This function performs topological sorting of downstream modules based on their dependencies. It:

1. **Dependency Analysis**: Analyzes dependencies between downstream modules
2. **Graph Construction**: Builds a dependency graph
3. **Topological Sort**: Performs topological sorting using Kahn's algorithm
4. **Cycle Detection**: Detects circular dependencies and raises ValueError
5. **Result**: Returns modules in correct execution order

**Parameters:**
- `sys`: The system builder containing downstream modules

**Returns:**
- List of downstream modules in topological order

**Raises:**
- `ValueError`: If circular dependencies are detected

### get_upstreams

```python
def get_upstreams(module):
    """Get upstream modules of a given module."""
```

**Explanation:**

This function identifies upstream modules that a given module depends on. It:

1. **External Analysis**: Analyzes external dependencies of the module
2. **Upstream Identification**: Identifies upstream modules from external references
3. **Dependency Filtering**: Filters out certain types of dependencies
4. **Parent Detection**: Supports both legacy block parents and the new module-level parents introduced by the block removal refactor
5. **External Module Handling**: Treats any expression found in `module.externals` as a potential dependency source
6. **Result**: Returns set of upstream modules

**Parameters:**
- `module`: The module to analyze

**Returns:**
- Set of upstream modules

## Section 2. Internal Helpers

### Dependency Graph Construction

The module constructs dependency graphs using:

1. **Graph Representation**: Uses defaultdict(list) for adjacency list
2. **In-degree Tracking**: Tracks in-degree for each module
3. **Dependency Addition**: Adds dependencies based on external references
4. **Cycle Detection**: Detects cycles during topological sort

### External Module Handling

Dependencies are derived from the expressions stored in `module.externals`. The analysis no longer inspects wire assignments directly; instead it trusts the IR builder to record any cross-module values in the externals list.

## Error Conditions

### Circular Dependencies

The module detects and handles circular dependencies:

1. **Detection**: Detected during topological sort
2. **Error Handling**: Raises ValueError with descriptive message
3. **Prevention**: Prevents infinite loops in module execution
4. **Recovery**: No automatic recovery, requires manual intervention

### Missing Dependencies

The module handles missing dependencies:

1. **Graceful Handling**: Handles missing attributes gracefully
2. **Default Values**: Uses default values for missing dependencies
3. **Error Prevention**: Prevents crashes from missing dependencies
4. **Robustness**: Ensures robust operation with incomplete data
