# DRAM Module

This module simulates an off-chip DRAM module
interacts with the on-chip pipeline.

This module extends `MemoryBase` in [base.py](./base.py)
with a single build.

```python
def DRAM(MemoryBase):

    @downstream.combinational
    def build(self, we, re, addr, wdata);
```

Unlike [SRAM](./sram.py), the data should be handled as soon as response,
we extend several [intrinsics](../../ir/expr/intrinsic.py) to achieve this.
Refer to its memory section for more details.

It calls and `send_read_request(self, addr, re)` and `send_write_request(self, addr, data, we)`.