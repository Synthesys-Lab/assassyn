# Intrinsic Functions

This module declares each intrinsic, and implements their frontend builders with `@ir_builder` annotated.

---

## DRAM Intrinsics

1. `send_read_request(mem, addr)`: Send read request with the address argument to the given `mem`ory system.
2. `read_request_succ(mem)`: It returns the combinational pin indicates if the read request sent in this cycle succeeds. If not send, return false.
3. `send_write_request(mem, addr, data)`: Send write request with the address and write enable signal to the given `mem`ory system, and returns if this request is successful in combinational pin.
4. `write_request_succ(mem)`: It returns the combinational pin indicates if the write request sent in this cycle succeeds. If not send, return false.
5. `has_mem_resp(mem)`: This is a purely combinational pin that checks if the given memory has response.
6. `get_mem_resp(mem)`: Get the memory response data. The lsb are the data payload, and the msb are the corresponding request address.

2 & 4 are designed to work against the constraint of "scope". Consider the code below:
```python
with Condition(we):
    x = send_write_request(self)
# We cannot get x here, as x is outside the scope.
```

Thus, we develop separate intrinsics to check if the memory request send is successful.

---