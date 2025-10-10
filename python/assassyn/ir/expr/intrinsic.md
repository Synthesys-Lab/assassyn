# Intrinsic Functions

This module declares each intrinsic, and implements their frontend builders with `@ir_builder` annotated.
Each intrinsic should have a corresponding `Intrinsic.UPPER_CASE_INTRINSIC` opcode.

---

## DRAM Intrinsics

1. `send_read_request(mem, re, addr)`: Send read request with the address argument to the given `mem`ory system, and return if this request is successful.
   - If `re` is not enabled, just do not send the `request`.
2. `send_write_request(mem, we, addr, data)`: Send write request with the address and write enable signal to the given `mem`ory system, and returns if this request is successful in combinational pin.
3. `has_mem_resp(mem)`: This is a purely combinational pin that checks if the given memory has response.
4. `get_mem_resp(mem)`: Get the memory response data. The lsb are the data payload, and the msb are the corresponding request address.

The DRAM intrinsics now support per-DRAM-module memory interfaces with proper callback handling and response management, replacing the previous single global memory interface approach.

2 & 4 are designed to work against the constraint of "scope". Consider the code below:
```python
with Condition(we):
    x = send_write_request(self)
# We cannot get x here, as x is outside the scope.
```

Thus, we develop separate intrinsics to check if the memory request send is successful.

---