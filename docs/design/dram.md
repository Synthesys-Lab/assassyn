# Memory System

Assassyn relies on [Ramulator2](../../3rd-party/ramulator2/)
to simulate the memory system.
The original Ramulator adopts a memory system with no response
for memory writings, which requires careful on-chip design
to resolve the aliases and enforce the memory ordering.

Currently, we adopt a simple hack to add write response to
Ramulator [as documented](../../scripts/init/patches/ramulator2-patch.md).

## Current Limitations

**No LSQ Memory Order Enforcement**: The current implementation does not enforce memory ordering constraints that would typically be handled by a Load Store Queue (LSQ) in a real processor. This means:

- Memory operations may complete out of order
- Write buffer management uses a simple queue without proper ordering guarantees
- This limitation is acceptable for the first version but should be addressed in future iterations

> RFC: Is it a good long term design? Or later we design a better LSQ?