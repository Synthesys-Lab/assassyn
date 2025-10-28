# Predicate Helpers (block.py)

## Design Documents

- [DSL Design](../../../docs/design/lang/dsl.md) – Trace-oriented DSL constructs and predicate semantics.
- [Module Design](../../../docs/design/internal/module.md) – Module representation and body layout.
- [Simulator Design](../../../docs/design/internal/simulator.md) – Predicate-driven control flow during simulation.

## Related Modules

- [Builder Singleton](../../builder/__init__.md) – Maintains module contexts and predicate stacks.
- [Expression Base](../expr.md) – Base expression hierarchy used inside module bodies.
- [Module Base](../module/base.md) – Provides `Module.body` and combinational decorator infrastructure.

## Section 0. Summary

The historical `Block` hierarchy has been removed in favour of a flat module body that directly owns an ordered list of expressions. The `block.py` module now provides only lightweight helpers for predicate management, namely the `Condition` and `Cycle` context managers and the internal `_PredicateScope` wrapper. These helpers emit predicate push/pop intrinsics so that frontend code can continue to guard statements with `with Condition(cond): ...` while the builder records the predicate stack per module context.

## Section 1. Exposed Interfaces

### `Condition(cond)`
```python
def Condition(cond: Value) -> ContextManager
```

**Purpose:** Guard a group of statements with the given predicate by emitting `push_condition(cond)` and `pop_condition()` intrinsics around the enclosed statements.

**Parameters:**
- `cond`: A `Value` describing the predicate that must hold for the guarded statements.

**Returns:** A context manager that integrates with the builder's predicate stack.

**Explanation:** Entering the context pushes the predicate onto the active module's predicate stack (and emits the corresponding intrinsic IR node). All IR created while the context is active is therefore conditionally executed. Exiting the context pops the predicate and emits the matching `pop_condition` intrinsic. This mirrors legacy conditional blocks without maintaining a dedicated block AST node.

**Example:**
```python
with Condition(enable_signal):
    log("Enabled value: {}", enable_signal)
```

### `Cycle(cycle)`
```python
def Cycle(cycle: int) -> ContextManager
```

**Purpose:** Sugar for guarding statements with `current_cycle() == cycle`. Useful for testbench-like scheduling written in the frontend.

**Parameters:**
- `cycle`: Absolute cycle number that should trigger the guarded statements.

**Returns:** A `Condition` context manager equivalent to `Condition(current_cycle() == UInt(64)(cycle))`.

**Example:**
```python
with Cycle(10):
    finish()
```

## Section 2. Internal Helpers

### `_PredicateScope`
```python
class _PredicateScope:
    def __enter__(self)
    def __exit__(self, exc_type, exc_value, traceback)
```

**Purpose:** Minimal context manager used by both `Condition` and `Cycle`. Its body pushes the predicate via `push_condition` on enter and pops it on exit. The scope delegates predicate-stack management to `SysBuilder`, ensuring the array-read cache and other predicate-sensitive data stay aligned with the active predicates.

**Design Notes:**
- The scope is intentionally lightweight; it does not attempt to manage insertion points or additional builder context.
- All mutation is driven by the intrinsic helper functions in `ir.expr.intrinsic`, keeping predicate semantics concentrated in one module.

### Implementation Considerations

- Because module bodies are plain lists, predicate intrinsics appear inline alongside other expressions. Consumers such as visitors and code generators should treat them as structural markers and handle indentation or gating logic accordingly.
- Removing structural blocks simplifies IR traversal: visitors iterate directly over `Module.body` and react to predicate intrinsics instead of recursing into nested block objects.
