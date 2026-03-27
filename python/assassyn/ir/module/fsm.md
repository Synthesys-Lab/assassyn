# Finite State Machine Helper

## Summary

The `FSM` class provides a declarative interface for creating finite state machines within Assassyn modules. It simplifies the implementation of state-based control logic by automatically generating the necessary combinational logic from a transition table and state-specific action functions, following Assassyn's credit-based pipeline architecture as described in the [architectural design](../../../docs/design/arch/arch.md).

**Key Benefits:**

- **Declarative**: Separate state transitions from state actions for clearer logic
- **Maintainable**: Easy to visualize and modify state machine behavior
- **Compact**: Reduces boilerplate code compared to manual `Condition` blocks
- **Type-safe**: Automatic state encoding with minimal bit width

## Exposed Interfaces

### FSM Class

```python
class FSM:
    def __init__(self, state_reg: Array, transition_table: dict): ...
    def generate(self, func_dict: dict, mux_dict: dict = None): ...
```

## Usage Example

Here is a complete example showing how to use the FSM class:

```python
from assassyn.frontend import *
from assassyn.module import fsm

class MyModule(Module):
    def __init__(self):
        super().__init__(ports={'input': Port(Int(32))})

    @module.combinational
    def build(self):
        # Get input
        data = self.pop_all_ports(True)

        # Create state register (FSM will auto-encode states)
        state = RegArray(Bits(2), 1, initializer=[0])
        counter = RegArray(Int(32), 1, initializer=[0])
        result = RegArray(Int(32), 1, initializer=[0])

        # Define transition conditions
        default = Bits(1)(1)
        counter_done = counter[0] >= Int(32)(10)
        data_valid = data > Int(32)(0)

        # Define transition table
        # Format: "state_name": {condition: "next_state", ...}
        transition_table = {
            "idle": {data_valid: "process", ~data_valid: "idle"},
            "process": {counter_done: "finish", ~counter_done: "process"},
            "finish": {default: "idle"},
        }

        # Define state-specific actions
        def idle_action():
            counter[0] = Int(32)(0)
            log("State: IDLE, waiting for valid data")

        def process_action():
            result[0] = result[0] + data
            counter[0] = counter[0] + Int(32)(1)
            log("State: PROCESS, counter={}", counter[0])

        def finish_action():
            log("State: FINISH, result={}", result[0])
            result[0] = Int(32)(0)

        action_dict = {
            "idle": idle_action,
            "process": process_action,
            "finish": finish_action,
        }

        # Create and generate FSM
        my_fsm = fsm.FSM(state, transition_table)
        my_fsm.generate(action_dict)
```

**Important Notes:**

- **Transition Evaluation**: Conditions are evaluated in the order they appear in the dictionary. The first matching condition determines the next state.
- **Action Execution**: State actions execute **before** transition evaluation in the same cycle.
- **Default Transitions**: Use `Bits(1)(1)` for unconditional transitions.

## Internal Helpers

### FSM Implementation Details

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
   - Example: 4 states → 2 bits, 8 states → 3 bits
   - **Important**: Make sure your `state_reg` has enough bits! Use `Bits(math.ceil(math.log2(num_states)))`
4. **State Mapping Creation:** Generates a mapping from state names to their corresponding bit values, starting from 0
   - States are encoded in the order they appear in the transition table

The method automatically handles state encoding, ensuring efficient hardware implementation with minimal state bits.

**Parameters:**

- `state_reg: Array` - The state register (must be `RegArray` with `Bits` type)
- `transition_table: dict` - Dictionary mapping state names to transition conditions

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
