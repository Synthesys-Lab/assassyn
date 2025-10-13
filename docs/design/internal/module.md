# Module Generation

This document describes how Assassyn modules are generated for both simulation and Verilog.
For architectural context and execution model, see [arch.md](../arch/arch.md).
For Verilog pipeline generation, see [pipeline.md](./pipeline.md).
For simulator implementation, see [simulator.md](./simulator.md).

An Assassyn module is similar to but different from a SystemVerilog module.
Just like a SystemVerilog module, it has well-defined inputs.
Unlike a SystemVerilog module, it has no explicit outputs.
It works like a void function in C, and all the outputs are implicitly
applied by side-effect operations, like calling other functions,
and writing to stateful data arrays (registers and memories).

## Simulator

Generating a module is trivial for a simulator,
because hardware is simpler than software --- no back-edges.
We faithfully translate the operations within each module
to their corresponding high-level language operations.

The only differences are the register-writing-related operations:
1. Write to a register: instead of directly writing to the register,
   the value to be written to the register is kept in bookkeeping,
   and committed to the register at the half cycle, as discussed
   in [simulator.md](./simulator.md).
2. Write to a stage register: stage registers are declared as
   port FIFOs, and done by `FIFOPush`. Likewise, they are written
   to bookkeeping, and committed at the half cycle.
3. Async call to a stage: Stage activation is done by an event FIFO,
   every function call may push an event to this FIFO, and every
   successful activation may pop an element from this FIFO.

## Verilog

The key difference of generating Verilog compared to simulator is
that there is no branches. We maintain a stack of conditions on the
path of reaching each statement to predicate on/off the execution.

The register assignment is fundamentally supported by non-blocking
assignment, `<=`, so we just need to make sure the writing condition
is true:

```verilog
always(posedge clk) begin
   if (write_enable) begin
      data[idx] <= value;
   end
end
```

As discussed in [pipeline.md](./pipeline.md), we need to connect to
the FIFOs and the counters to handle the async call.