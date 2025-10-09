# Memory Base Module

This file provides the `MemoryBase` class, which contains common functionality to serve as a base for different memory types like SRAM and DRAM.

-----

## Exposed Interfaces

```python
class MemoryBase:
    def __init__(self, width: int, depth: int, init_file: str): ...
```

-----

## MemoryBase Class

`MemoryBase` is a foundational class for memory modules, defining their common attributes and initialization logic.

### Initialization (`__init__`)

The constructor initializes the memory's core properties. It takes the memory's `width` and `depth` as arguments and uses them to create an underlying `RegArray` for storage. Standard interface signals like `we` (write enable), `re` (read enable), and `addr` (address) are initialized to `None`.

### Attributes

The class defines the standard interface for a memory module:

  * `width`: The width of the memory in bits.
  * `depth`: The number of words the memory can store.
  * `init_file`: An optional path to a file used to initialize the memory's contents.
  * `payload`: The `RegArray` instance that holds the memory data.
  * `we`, `re`, `addr`, `wdata`: The standard memory port signals for write enable, read enable, address, and write data.
