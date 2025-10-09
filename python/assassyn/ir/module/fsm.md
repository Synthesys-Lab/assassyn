# FSM (Finite State Machine) Helper

This file provides the `FSM` class, which is a helper for simplifying the creation of Finite State Machines within a module.

-----

## Exposed Interfaces

```python
class FSM:
    def __init__(self, state_reg: Array, transition_table: dict): ...
    def generate(self, func_dict: dict, mux_dict: dict = None): ...
```

-----

## FSM Class

The `FSM` class constructs FSM logic from a declarative transition table and dictionaries of state-specific actions.

### Initialization (`__init__`)

The FSM is initialized with a state register (`Array`) and a transition table (`dict`). It automatically calculates the number of bits needed to encode the states and creates a mapping from state names to their corresponding bit values.

### Logic Generation (`generate`)

The `generate` method builds the FSM's combinational logic based on the provided tables and dictionaries.

  * It creates a main conditional block for each state that activates when the `state_reg` matches that state's value.
  * Within each state's block, it executes the corresponding action function provided in the `func_dict`.
  * It generates nested conditional logic for all state transitions defined in the `transition_table`, assigning the next state's value to the `state_reg` when a condition is met.
  * Optionally, if a `mux_dict` is provided, it also generates multiplexer logic using `select` operations to choose a value based on the current state.
