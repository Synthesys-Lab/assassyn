# Finite State Machine Helper

## Summary

The `FSM` class provides a declarative interface for creating finite state machines within Assassyn modules. It simplifies the implementation of state-based control logic by automatically generating the necessary combinational logic from a transition table and state-specific action functions, following Assassyn's credit-based pipeline architecture as described in the [architectural design](../../../docs/design/arch/arch.md).

## Exposed Interfaces

### FSM Class

```python
class FSM:
    def __init__(self, state_reg: Array, transition_table: dict): ...
    def generate(self, func_dict: dict, mux_dict: dict = None): ...
```

## Internal Helpers

### FSM Class

The `FSM` class constructs finite state machine logic from declarative specifications.

**Purpose:** Provides a high-level interface for implementing state-based control logic within modules, automatically handling state encoding, transition logic, and state-specific actions.

**Member Fields:**
- `state_reg: Array` - The state register storing the current state
- `transition_table: dict` - Dictionary mapping states to their possible transitions
- `state_bits: int` - Number of bits needed to encode all states
- `state_map: dict` - Dictionary mapping state names to their bit representations

**Methods:**

#### `__init__(self, state_reg, transition_table)`

**Explanation:**
Initializes a finite state machine with a state register and transition table. The constructor:

1. **State Register Validation:** Ensures the provided state register is an `Array` object
2. **Transition Table Storage:** Stores the transition table for later use in logic generation
3. **State Bit Calculation:** Computes the minimum number of bits needed to encode all states using `math.floor(math.log2(len(transition_table)))`
4. **State Mapping Creation:** Generates a mapping from state names to their corresponding bit values, starting from 0

The method automatically handles state encoding, ensuring efficient hardware implementation with minimal state bits.

#### `generate(self, func_dict, mux_dict=None)`

**Explanation:**
Generates the combinational logic for the finite state machine. This method:

1. **State-Specific Logic:** For each state in the transition table, creates a conditional block that executes when `state_reg[0]` matches the state's encoded value
2. **Action Execution:** Within each state's block, calls the corresponding action function from `func_dict` if provided
3. **Transition Logic:** Generates nested conditional blocks for each possible transition from the current state, updating `state_reg[0]` to the next state's encoded value when the transition condition is met
4. **Multiplexer Generation:** If `mux_dict` is provided, generates multiplexer logic using `select` operations to choose values based on the current state

The method uses Assassyn's `Condition` context manager to create the necessary combinational logic blocks, ensuring proper integration with the IR system.

**Parameters:**
- `func_dict: dict` - Dictionary mapping state names to action functions
- `mux_dict: dict, optional` - Dictionary for generating state-dependent multiplexer logic

**Design Decisions:**
- Uses `math.log2` for optimal state encoding, ensuring minimal hardware overhead
- Employs `Condition` blocks for clean separation of state-specific logic
- Supports optional multiplexer generation for state-dependent value selection
- Prints debug information during construction and generation for development assistance
