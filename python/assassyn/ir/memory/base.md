# Memroy Base

This module implements the base interface for a memory:

```python
class MemoryBase(Downstream):
    '''Base class for memory modules.'''
    # Builtin property of a memory
    width: int      # Width of the memory in bits
    depth: int      # Depth of the memory in words

    # For simulation purpose only
    init_file: str  # Path to initialization file

    # All the combinational pins into this downstream module.
    we: Value       # Write enable signal
    re: Value       # Read enable signal
    addr: Value     # Address signal
    wdata: Value    # Write data signal

    # Dervied Values

    # Depth is required to be a power of 2
    # Width is the log2 of depth
    addr_width: int # Width of the address in bits

    # The array payload as per the depth and width
    payload: Array  # Array holding the memory contents
```

The module provides a constructor, `__init__()`, which checks
all the assumptions of the un-derived values, and derive the values.
The payload array if from [ir/array.py](../array.py).