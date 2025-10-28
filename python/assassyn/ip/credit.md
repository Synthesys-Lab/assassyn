# Credit Counter IP Module

This module provides a credit counter for credit-based flow control in pipeline architectures, equivalent to the SystemVerilog `trigger_counter.sv` module.

## Design Documents

- [trigger_counter.sv](../codegen/verilog/trigger_counter.sv) - Original SystemVerilog implementation
- [Downstream Tutorial](../../tutorials/downstream.qmd) - Downstream module architecture and usage patterns

## Summary

The `CreditCounter` is a downstream module that maintains an internal counter for tracking credits in a pipeline system. Unlike a FIFO which can only push/pop once per cycle, this counter can handle multiple event increments (via `delta`) in a single cycle while supporting single pop operations.

## Exposed Interfaces

### CreditCounter Class

```python
class CreditCounter(Downstream):
    def __init__(self, width: int = 8, debug: bool = False)
    def build(self, delta: Value, pop_ready: Value) -> Tuple[Value, Value]
```

**Purpose**: Maintains a credit counter with increment/decrement operations and overflow/underflow protection.

**Constructor Parameters**:
- `width`: Bit width of the counter (default: 8, max value = 2^width - 1)
- `debug`: Enable debug logging for verification (default: False)

**Build Parameters**:
- `delta`: Value to add to the counter each cycle (UInt(width))
- `pop_ready`: Signal indicating whether to decrement counter by 1 (UInt(1))

**Returns**: Tuple of (delta_ready, pop_valid)
- `delta_ready`: High when counter != max_value (can accept more credits)
- `pop_valid`: High when counter != 0 or delta != 0 (has credits available)

**Internal State**:
- `count_reg`: RegArray maintaining the current counter value

**Logic**:
1. Compute `temp = count + delta`
2. Compute `new_count = temp - (pop_ready ? 1 : 0)` with underflow protection
3. Update counter register
4. Assert `delta_ready` if not at maximum capacity
5. Assert `pop_valid` if credits available

## Usage Example

```python
from assassyn.frontend import *
from assassyn.ip.credit import CreditCounter

# Create counter with 8-bit width
counter = CreditCounter(width=8, debug=False)

# In a Module's build method:
delta = UInt(8)(2)  # Add 2 credits
pop_ready = UInt(1)(1)  # Pop 1 credit

delta_ready, pop_valid = counter.build(delta, pop_ready)

# delta_ready indicates if counter can accept more credits
# pop_valid indicates if counter has credits to pop
```

## Test Case

See `test_ip_credit.py` for a complete test demonstrating:
- Counter increment via delta values
- Counter decrement via pop_ready
- Overflow protection verification
- Strict correctness checking of counter arithmetic
