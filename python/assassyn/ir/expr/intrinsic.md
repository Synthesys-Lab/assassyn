# intrinsic function description

---

## DRAM related intrinsic functions
1. `send_read_request(addr)`: Send read request with the address argument to memory system.
2. `send_write_request(addr, we)`: Send write request with the address and write enable signal to memory system.
3. `mem_write(payload, addr, wdata)`: If write request sent successfully, the data will be written into payload[addr].
4. `has_mem_resp(memory)`: Check if we have a memory response, this is used to check that we have completed some memory read requests.
5. `mem_resp(memory)`: Get the memory response data. We pass the memory module argument so that we can get the `width` or other information in the module.
6. `use_dram(dram)`: Tell the system that we need to use `DRAM` memory system so that it will generate related modules.

---