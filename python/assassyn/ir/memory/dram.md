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

1. When `re` is enabled it calls `send_read_request(self)`.
2. When `we` is enabled it calls `send_write_request(self)`.
3. It returns `read_request_succ(self)` and `write_request_succ(self)`
   for further downstreams to check if it succeeds. It is developers'
   duty to resend the unsuccessful requests.