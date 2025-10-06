# SRAM Module

This module provides all the needed SRAM implementation and modeling.
SRAM is a module inherited from `MemoryBase` in `base.py`, which extends
one additional field and one method.

```python
def SRAM(MemoryBase):
    dout: Array

    def __init__(self, width: int, depth: int, init_file: str);

    @downstream.combinational
    def build(self);
```

- `dout`: a register buffer that holds the result of read if last cycle `re` is enabled.
  `dout` should be a `UInt(width)`.

- `build(we, re, addr, wdata)`: is the IR builder method that fills the body of SRAM.
    - When `we` is enabled, it writes to `_payload` of the given address.
    - When `re` is enabled, it writes to `dout` of the results.
    - `we` and `re` cannot be both enabled, use `assume` in [intrinsic.py](../expr/intrinsic.py) to enforce it.