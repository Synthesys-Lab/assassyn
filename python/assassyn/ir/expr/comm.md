# Commutative Operations Module

This file provides helper functions for applying commutative operations across a variable number of arguments.

-----

## Exposed Interfaces

```python
def add(*args) -> Value
def mul(*args) -> Value
def and_(*args) -> Value
def or_(*args) -> Value
def xor(*args) -> Value
def concat(*args) -> Value
```

-----

## Functionality

The module provides variadic functions that apply a single operation across all provided arguments.

  * The `add`, `mul`, `and_`, `or_`, and `xor` functions reduce the argument list by repeatedly applying the corresponding Python operator (`+`, `*`, `&`, `|`, `^`).
  * The `concat` function reduces the argument list by repeatedly calling the `.concat()` method on the arguments, effectively chaining them together.
