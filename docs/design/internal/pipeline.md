# Verilog Pipeline Generation

This document describes how Assassyn generates Verilog code for pipelined architectures.
For the execution model and architectural concepts, see [arch.md](../arch/arch.md).
For module generation details, see [module.md](./module.md).
For simulator implementation, see [simulator.md](./simulator.md).

## Overview

Assassyn generates Verilog code that implements the credit-based pipeline architecture described in [arch.md](../arch/arch.md). The key challenge in Verilog generation is translating the high-level execution model into synthesizable hardware that maintains the same behavior.

### Analysis Pre-pass

Before any Verilog is emitted the backend now performs a dedicated metadata pre-pass:

1. `collect_fifo_metadata` instantiates a lightweight `FIFOAnalysisVisitor` that walks every module body, recording each array read/write and FIFO push/pop interaction’s predicate carry (`expr.meta_cond`), FINISH intrinsics, async calls, and cross-module exposures. Predicates remain raw IR values, so emission reuses the exact same guards.
2. The visitor populates an `InteractionMatrix` plus `ModuleMetadata` records; the matrix owns shared interaction tuples (accessible via module and resource views) and the `AsyncLedger` that groups async calls by callee, while the module metadata captures FINISH sites, async call lists, and value exposures. In parallel, `collect_external_metadata` builds an `ExternalRegistry` holding external classes, instance ownership, and cross-module reads. All of these types live in the split metadata package and remain available via `python.assassyn.codegen.verilog.metadata`. The resulting snapshot (array/FIFO traffic, external metadata, FINISH flags, async trigger lists, value exposures) is handed directly to the dumper constructor.
3. Callers that need a partial refresh can analyse a subset of modules and merge the returned metadata without mutating previously produced registries during code emission.

This separation removes runtime bookkeeping from code emission and guarantees that cleanup, module port generation, and top-level wiring all consult a consistent dataset.

Cleanup now routes both array writes and FIFO pushes through a shared `_emit_predicate_mux_chain` helper, so the metadata-derived predicates feed a single implementation of the reduction-and-mux pattern. The helper collapses single-entry collections to a passthrough assignment and lets callers supply deterministic defaults for the empty case, keeping enable signals and data selection aligned with the prioritisation defined during IR construction without sprinkling ad-hoc guards across call sites.

## Credit-based Flow Control Implementation

The credit system is implemented using counters and control logic. Each pipeline stage has:

- **Credit Counter**: Tracks pending activations
- **Trigger Logic**: Determines when a stage should execute
- **FIFO Interfaces**: Handle data flow between stages

### Trigger Counter Template

The credit-based micro-architecture is implemented using a [trigger counter template](../../python/assassyn/codegen/verilog/trigger_counter.sv):

```verilog
module trigger_counter (
    input clk,
    input rst,
    input trigger_in,      // Credit increment signal
    input wait_until,      // Credit consumption signal
    output reg triggered   // Stage activation signal
);
    reg [WIDTH-1:0] credit_count;
    
    always @(posedge clk) begin
        if (rst) begin
            credit_count <= 0;
            triggered <= 0;
        end else begin
            // Increment credits on trigger
            if (trigger_in) begin
                credit_count <= credit_count + 1;
            end
            
            // Consume credit and trigger stage
            if (wait_until && credit_count > 0) begin
                credit_count <= credit_count - 1;
                triggered <= 1;
            end else begin
                triggered <= 0;
            end
        end
    end
endmodule
```

## Stage FIFO Implementation

Stage registers are implemented as FIFOs to maintain architectural generality. The [FIFO template](../../python/assassyn/codegen/verilog/fifo.sv) provides:

- **Push Interface**: For upstream stages to send data
- **Pop Interface**: For downstream stages to receive data
- **Valid Signal**: Indicates data availability

### FIFO Template

```verilog
module fifo #(
    parameter WIDTH = 32,
    parameter DEPTH = 8
) (
    input clk,
    input rst,
    input push,
    input [WIDTH-1:0] push_data,
    input pop,
    output [WIDTH-1:0] pop_data,
    output full,
    output empty
);
    reg [WIDTH-1:0] memory [DEPTH-1:0];
    reg [DEPTH-1:0] write_ptr, read_ptr;
    reg [DEPTH:0] count;
    
    always @(posedge clk) begin
        if (rst) begin
            write_ptr <= 0;
            read_ptr <= 0;
            count <= 0;
        end else begin
            if (push && !full) begin
                memory[write_ptr] <= push_data;
                write_ptr <= (write_ptr + 1) % DEPTH;
                count <= count + 1;
            end
            
            if (pop && !empty) begin
                read_ptr <= (read_ptr + 1) % DEPTH;
                count <= count - 1;
            end
        end
    end
    
    assign pop_data = memory[read_ptr];
    assign full = (count == DEPTH);
    assign empty = (count == 0);
endmodule
```

### PyCDE Runtime Helpers

The generated `design.py` imports reusable PyCDE helpers from `assassyn.pycde_wrapper`. This module defines the parameterized `FIFO` and `TriggerCounter` classes using `@modparams`, mirroring the handwritten templates above. Centralizing the definitions prevents divergent copies of these primitives between generated designs and user-authored PyCDE code.

## Combinational Downstream Modules

Downstream modules are implemented as pure combinational logic. The key considerations are:

1. **Topological Ordering**: Modules must be instantiated in dependency order
2. **Signal Propagation**: All inputs must be available in the same cycle
3. **No Feedback**: No combinational loops are allowed

### Downstream Module Template

```verilog
module downstream_module (
    input [31:0] input_a,
    input [31:0] input_b,
    input valid_a,
    input valid_b,
    output [31:0] output_c,
    output valid_c
);
    // Pure combinational logic
    assign output_c = input_a + input_b;
    assign valid_c = valid_a & valid_b;
endmodule
```

## Register Array Implementation

Register arrays support multiple concurrent access patterns with port-based arbitration:

### Multi-Port Register Array

```verilog
module reg_array #(
    parameter WIDTH = 32,
    parameter DEPTH = 64,
    parameter NUM_PORTS = 2
) (
    input clk,
    input rst,
    // Port interfaces
    input [NUM_PORTS-1:0] write_enable,
    input [NUM_PORTS-1:0][$clog2(DEPTH)-1:0] write_addr,
    input [NUM_PORTS-1:0][WIDTH-1:0] write_data,
    output [WIDTH-1:0] read_data
);
    reg [WIDTH-1:0] memory [DEPTH-1:0];
    
    // Port arbitration - one-hot selection
    wire [NUM_PORTS-1:0] port_sel;
    assign port_sel[0] = write_enable[0];
    assign port_sel[1] = write_enable[1] & ~write_enable[0];
    // ... additional ports
    
    always @(posedge clk) begin
        if (rst) begin
            // Initialize memory
        end else begin
            for (int i = 0; i < NUM_PORTS; i++) begin
                if (port_sel[i]) begin
                    memory[write_addr[i]] <= write_data[i];
                end
            end
        end
    end
    
    // Read port (currently routes entire array)
    assign read_data = memory[read_addr];
endmodule
```

When an array belongs to a memory instance and `array.is_payload(memory)` returns `True`, the backend bypasses this generic module entirely and instead emits the specialised SRAM/DRAM wrappers that expose memory-specific handshakes. Ownership metadata is now identity-based, enabling the backend to distinguish payload buffers from standard registers without dedicated descriptor classes.

## Clock Domain and Timing

All generated Verilog follows these timing conventions:

- **Clock Edge**: All registers update on positive clock edge
- **Reset**: Synchronous reset with active-high polarity
- **Setup/Hold**: All signals meet timing requirements
- **Clock Gating**: Not implemented in current version

## Test Cases

The following test cases demonstrate the Verilog generation:

- [test_driver.py](../../python/ci-tests/test_driver.py): Basic driver module
- [test_async_call.py](../../python/ci-tests/test_async_call.py): Sequential communication
- [test_downstream.py](../../python/ci-tests/test_downstream.py): Combinational communication
- [test_array_multi_write.py](../../python/ci-tests/test_array_multi_write.py): Register arrays
- [test_toposort.py](../../python/ci-tests/test_toposort.py): Topological ordering
