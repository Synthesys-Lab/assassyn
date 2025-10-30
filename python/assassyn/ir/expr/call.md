# Async Call Related IR Nodes

This module defines the `FIFOPush`, `Bind`, and `AsyncCall` IR nodes, which represent function call operations in the assassyn AST. These classes implement the [async call mechanism](../../../docs/design/pipeline.md) for inter-stage communication, where modules can asynchronously invoke other modules through FIFO-based parameter passing.

## Design Documents

- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system and async call mechanism
- [Type System Design](../../../docs/design/lang/type.md) - Type system architecture and data type definitions
- [DSL Design](../../../docs/design/lang/dsl.md) - Trace-based DSL system

## Related Modules

- [Expression Base](../expr.md) - Base expression classes and operand system
- [Arithmetic Operations](../arith.md) - Arithmetic and logical operations
- [Intrinsic Operations](../intrinsic.md) - Intrinsic function operations
- [Commutative Operations](../comm.md) - Variadic operations

---

## Section 1. Exposed Interfaces

### class FIFOPush

The IR node class for FIFO push operations, representing the sending of data to a module's input port.

#### Static Constants

- `FIFO_PUSH = 302` - FIFO push operation opcode

#### Attributes

- `fifo: Port` - FIFO port to push to
- `bind: Bind` - Bind reference (set by Bind operations)
- `fifo_depth: int` - Depth of the FIFO (set by Bind operations)

#### Methods

#### `__init__(self, fifo, val, meta_cond=None)`

```python
def __init__(self, fifo, val, meta_cond=None):
    super().__init__(FIFOPush.FIFO_PUSH, [fifo, val], meta_cond=meta_cond)
    self.bind = None
    self.fifo_depth = None
```

**Explanation:** Initializes a FIFO push operation with the target FIFO port, value to push, and optional predicate metadata. The `bind` and `fifo_depth` attributes are initially None and are set by the `Bind` class when managing FIFO configurations. When constructed through the frontend (e.g., `Port.push()` or `Bind._push()`), `meta_cond` defaults to the active predicate returned by [`get_pred()`](../intrinsic.md#get_pred).

#### `fifo` (property)

```python
@property
def fifo(self):
    '''Get the FIFO port'''
    return self._operands[0]
```

**Explanation:** Returns the FIFO port that will receive the pushed data.

#### `val` (property)

```python
@property
def val(self):
    '''Get the value to push'''
    return self._operands[1]
```

**Explanation:** Returns the value to be pushed to the FIFO.

#### `dtype` (property)

```python
@property
def dtype(self):
    '''Get the data type of this operation (Void for side-effect operations)'''
    from ..dtype import void
    return void()
```

**Explanation:** Returns `Void()` type since FIFO push operations are side-effect operations that don't produce a value.

#### `__repr__(self)`

```python
def __repr__(self):
    handle = self.as_operand()
    meta = self.meta_cond
    meta_repr = ''
    if meta is not None:
        meta_repr = f' // meta cond {meta.as_operand() if hasattr(meta, \"as_operand\") else meta}'
    return f'{self.fifo.as_operand()}.push({self.val.as_operand()}) // handle = {handle}{meta_repr}'
```

**Explanation:** Returns a human-readable string representation of the FIFO push operation in the format `fifo.push(value) // handle = handle // meta cond predicate`, surfacing the predicate metadata alongside the handle for debugging.

### class Bind

The IR node class for binding operations. Function bind is a functional programming concept like Python's `functools.partial`, where arguments are bound to a module to create a callable entity.

#### Static Constants

- `BIND = 501` - Bind operation opcode

#### Attributes

- `callee: Module` - Module being bound
- `fifo_depths: dict` - Dictionary of FIFO depths for each port
- `pushes: list[FIFOPush]` - List of FIFOPush operations

#### Methods

#### `__init__(self, callee, **kwargs)`

```python
def __init__(self, callee, **kwargs):
    super().__init__(Bind.BIND, [])
    self.callee = callee
    self._push(**kwargs)
    self.fifo_depths = {}
```

**Explanation:** Initializes a bind operation with the target module and keyword arguments for port bindings. Creates FIFOPush operations for each provided argument and initializes the FIFO depths dictionary.

#### `_push(self, **kwargs)`

```python
def _push(self, **kwargs):
    for k, v in kwargs.items():
        push = getattr(self.callee, k).push(v)
        push.bind = self
        self.pushes.append(push)
```

**Explanation:** Internal method that creates FIFOPush operations for each keyword argument. Each push operation is associated with this bind operation and added to the pushes list. The underlying port helpers automatically populate the push's `meta_cond` with the current predicate, so every bound argument preserves its guard.

#### `bind(self, **kwargs)`

```python
def bind(self, **kwargs):
    '''The exposed frontend function to instantiate a bind operation'''
    self._push(**kwargs)
    return self
```

**Explanation:** Public method for adding more bindings to an existing Bind operation. Returns self to enable method chaining.

#### `is_fully_bound(self)`

```python
def is_fully_bound(self):
    '''The helper function to check if all the ports are bound.'''
    fifo_names = set(push.fifo.name for push in self.pushes)
    ports = self.callee.ports
    cnt = sum(i.name in fifo_names for i in ports)
    return cnt == len(ports)
```

**Explanation:** Checks whether all input ports of the callee module have been bound. Returns True if the number of bound ports equals the total number of ports.

#### `pushes` (property)

```python
@property
def pushes(self):
    '''Get the list of pushes'''
    return self._operands
```

**Explanation:** Returns the list of FIFOPush operations associated with this bind operation.

#### `dtype` (property)

```python
@property
def dtype(self):
    '''Get the data type of this operation (Void for binding operations)'''
    from ..dtype import void
    return void()
```

**Explanation:** Returns `Void()` type since bind operations are side-effect operations that don't produce a value.

#### `async_called(self, **kwargs)`

```python
@ir_builder
def async_called(self, **kwargs):
    '''The exposed frontend function to instantiate an async call operation'''
    self._push(**kwargs)
    return AsyncCall(self)
```

**Explanation:** Creates an AsyncCall operation from this bind operation. This method is decorated with `@ir_builder` to integrate with the [trace-based DSL](../../../docs/design/dsl.md) system.

#### `set_fifo_depth(self, **kwargs)`

```python
def set_fifo_depth(self, **kwargs):
    """Set FIFO depths using keyword arguments."""
    for name, depth in kwargs.items():
        if not isinstance(depth, int):
            raise ValueError(f"Depth for {name} must be an integer")
        matches = 0
        available_fifos = []
        for push in self.pushes:
            available_fifos.append(push.fifo.name)
            if push.fifo.name == name:
                push.fifo_depth = depth
                matches = matches + 1
        if matches == 0:
            raise ValueError(f"No push found for FIFO named {name}. "
                             f"Available FIFO names are: {available_fifos}")
    return self
```

**Explanation:** Sets the FIFO depth for specific ports using keyword arguments. Validates that the depth is an integer and that the FIFO name exists in the pushes list. This is used for [FIFO depth configuration](../../../docs/design/pipeline.md) in the generated hardware.

**Error Conditions:**
- `ValueError`: Raised if depth is not an integer in `set_fifo_depth`
- `ValueError`: Raised if no push is found for the specified FIFO name in `set_fifo_depth`
- FIFO configuration errors: May occur if FIFO depths are not properly configured before async calls

#### `__repr__(self)`

```python
def __repr__(self):
    args = []
    for v in self.pushes:
        depth = self.fifo_depths.get(v.as_operand())
        depth_str = f", depth={depth}" if depth is not None else ""
        operand = v.as_operand()
        operand = f'{operand} /* {v.fifo.as_operand()}={v.val.as_operand()}{depth_str} */'
        args.append(operand)
    args = ', '.join(args)
    callee = self.callee.as_operand()
    lval = self.as_operand()
    fifo_depths_str = ', '\
        .join(f"{k}: {v}" for k, v in self.fifo_depths.items() if v is not None)
    fifo_depths_repr = f" /* fifo_depths={{{fifo_depths_str}}} */" if fifo_depths_str else ""
    return f'{lval} = {callee}.bind([{args}]){fifo_depths_repr}'
```

**Explanation:** Returns a human-readable string representation of the bind operation, including all bound arguments and FIFO depth information.

### class AsyncCall

The IR node class for async call operations, representing the asynchronous invocation of a bound module.

#### Static Constants

- `ASYNC_CALL = 500` - Async call operation opcode

#### Methods

#### `__init__(self, bind: Bind)`

```python
def __init__(self, bind: Bind, meta_cond=None):
    super().__init__(AsyncCall.ASYNC_CALL, [bind], meta_cond=meta_cond)
    bind.callee.users.append(self)
```

**Explanation:** Initializes an async call operation with a bind operation and optional predicate metadata. Adds this call to the callee module's users list to track dependencies for [topological ordering](../../../docs/design/pipeline.md). `Bind.async_called()` automatically captures the active predicate (`get_pred()`) and passes it as `meta_cond`.

#### `bind` (property)

```python
@property
def bind(self) -> Bind:
    '''Get the bind operation'''
    return self._operands[0]
```

**Explanation:** Returns the bind operation that contains the arguments for this async call.

#### `dtype` (property)

```python
@property
def dtype(self):
    '''Get the data type of this operation (Void for call operations)'''
    from ..dtype import void
    return void()
```

**Explanation:** Returns `Void()` type since async call operations are side-effect operations that don't produce a value.

#### `__repr__(self)`

```python
def __repr__(self):
    bind = self.bind.as_operand()
    meta = self.meta_cond
    meta_repr = ''
    if meta is not None:
        meta_repr = f' // meta cond {meta.as_operand() if hasattr(meta, \"as_operand\") else meta}'
    return f'async_call {bind}{meta_repr}'
```

**Explanation:** Returns a human-readable string representation of the async call operation in the format `async_call bind_operation // meta cond predicate`, making the predicate explicit for debugging.

---

## Section 2. Internal Helpers

This module contains no internal helper functions or data structures. All functionality is exposed through the FIFOPush, Bind, and AsyncCall classes.
