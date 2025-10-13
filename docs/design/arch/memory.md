# Memory System

This document describes the memory system architecture in Assassyn.
For the broader architectural context and execution model, see [arch.md](./arch.md).
For Verilog implementation details, see [simulator.md](../internal/simulator.md).

## SRAM

Assassyn emulates the behavior of an exclusive-single-read-write SRAM bank
from ASAP7 as shown below:

````verilog
// synchronous SRAM verilog

module srambank_128x4x16_6t122 (
			    input clk
			  , input [8:0] ADDRESS          // address
			  , input [15:0] wd                // data to write
		       	  , input banksel                    // access enable
			  , input read                       // read enable
			  , input write                      // write enable
			  , output reg [15:0] dataout      // latched data output (only updated on read)
			    );

   reg [15:0] 				      mem [511:0];

   always @ (posedge(clk))
     begin                            // should have an error assert on read & write at once...
	if (write & banksel)
	  mem[ADDRESS] <= wd;
	else if (read & banksel)
	  dataout <= mem[ADDRESS];    // output is latched until next read, independent of writes
     end
endmodule // 
````

Where all the inputs, `ADDRESS`, `wd`, `banksel`, `read`, and `write`, are combinational signals
and the output `dataout` is a register. Thus, a key difference between an SRAM and a register file
is having 1 cycle read latency. Further, to fit in the
[Assassyn architecture template](./arch.md),
an SRAM is treated as a downstream module.

## DRAM

Assassyn relies on [Ramulator2](../../3rd-party/ramulator2/)
to simulate the memory system.
The original Ramulator adopts a memory system with no response
for memory writings, which requires careful on-chip design
to resolve the aliases and enforce the memory ordering.

Similar to SRAM, DRAM is also treated as a downstream module.
The key difference is that DRAM has a variable latency,
thus we provide two intrinsics, which will later be lowered to
combinational signals, to check if the memory request is back:
- `has_mem_resp(mem)`: returns true if there is a memory response back from the given DRAM `mem`.
- `get_mem_resp(mem)`: returns the memory response from the given DRAM `mem`. The MSB is the address, and the LSB is the data.

Currently, we adopt a simple hack to add write response to
Ramulator [as documented](../../scripts/init/patches/ramulator2-patch.md).

### Current Limitations

**No LSQ Memory Order Enforcement**: The current implementation does not
enforce memory ordering constraints that would typically be handled by a
Load Store Queue (LSQ) in a real processor. This means:

- Memory operations may complete out of order
- Write buffer management uses a simple queue without proper ordering guarantees
- This limitation is acceptable for the first version but should be addressed in future iterations

> RFC: Is this a good long-term design? Or should we design a better LSQ later?
