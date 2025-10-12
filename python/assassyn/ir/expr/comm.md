# Commutative Operations Module

This module provides helper functions for applying commutative operations across a variable number of arguments. These functions enable variadic operations that can take multiple operands and apply a single operation across all of them, which is useful for simplifying expressions with multiple operands.

---

## Section 1. Exposed Interfaces

### `reduce(op, *args)`

```python
def reduce(op, *args):
    '''Reduce the arguments using the operator'''
    res = args[0]
    for arg in args[1:]:
        res = op(res, arg)
    return res
```

**Explanation:** Generic reduction function that applies a binary operator across all provided arguments. Takes the first argument as the initial result and then applies the operator between the result and each subsequent argument.

### `add(*args)`

```python
def add(*args):
    '''Add all the arguments'''
    return reduce(operator.add, *args)
```

**Explanation:** Variadic addition function that adds all provided arguments together. Uses Python's `operator.add` to perform the addition operations.

### `mul(*args)`

```python
def mul(*args):
    '''Multiply all the arguments'''
    return reduce(operator.mul, *args)
```

**Explanation:** Variadic multiplication function that multiplies all provided arguments together. Uses Python's `operator.mul` to perform the multiplication operations.

### `and_(*args)`

```python
def and_(*args):
    '''Bitwise and on all the arguments'''
    return reduce(operator.and_, *args)
```

**Explanation:** Variadic bitwise AND function that applies bitwise AND across all provided arguments. Uses Python's `operator.and_` to perform the bitwise AND operations.

### `or_(*args)`

```python
def or_(*args):
    '''Bitwise or on all the arguments'''
    return reduce(operator.or_, *args)
```

**Explanation:** Variadic bitwise OR function that applies bitwise OR across all provided arguments. Uses Python's `operator.or_` to perform the bitwise OR operations.

### `xor(*args)`

```python
def xor(*args):
    '''Bitwise xor on all the arguments'''
    return reduce(operator.xor, *args)
```

**Explanation:** Variadic bitwise XOR function that applies bitwise XOR across all provided arguments. Uses Python's `operator.xor` to perform the bitwise XOR operations.

### `concat(*args)`

```python
def concat(*args):
    '''Concatenate multiple values using the concat method'''
    if len(args) < 2:
        raise ValueError("concat requires at least two arguments")
    return reduce(lambda x, y: x.concat(y), *args)
```

**Explanation:** Variadic concatenation function that concatenates all provided arguments using their `.concat()` method. Requires at least two arguments and uses a lambda function to chain the concatenation operations. This is commonly used for [bit concatenation operations](../../../docs/design/pipeline.md) in hardware design.

---

## Section 2. Internal Helpers

This module contains no internal helper functions or data structures beyond the exposed interface functions.
