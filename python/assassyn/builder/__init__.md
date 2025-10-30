# Builder Module (__init__.py)

## Section 0. Summary

This module implements the system-wide IR builder (SysBuilder) and the global Singleton used to manage building context (modules, naming, and caches). It also defines the ModuleContext record that holds per-module state, including the per-module predicate stack used by predicate intrinsics. Every expression materialised through the builder now records the active module as its `parent`, relying on the invariant that module contexts are present whenever IR nodes are emitted.

Module bodies are plain Python lists owned by each module instance. The builder derives the active insertion list directly from the current module instead of keeping additional body stacks. Predicate push/pop intrinsics (emitted by `Condition`) rely on the per-module predicate stack maintained here.

## Section 1. Exposed Interfaces

### class ModuleContext
```python
class ModuleContext:
    module: Module
    cond_stack: list[PredicateFrame]
```

Purpose: Encapsulates per-module builder state requiring stack semantics. Currently contains the owning module and its cond_stack (predicate stack).

- module: The module object associated with this context frame.
- cond_stack: The predicate stack for this module context. The stack holds PredicateFrame objects in LIFO order.

### class SysBuilder
Core builder that also represents a system under construction. Key exposed properties and methods:

```python
class SysBuilder:
    @property
    def current_module(self): ...

    @property
    def current_body(self): ...

    @property
    def insert_point(self): ...

    def enter_context_of(self, module): ...
    def exit_context_of(self): ...

    # Predicate helpers (per current module)
    def get_predicate_stack(self): ...
    def push_predicate(self, cond): ...
    def pop_predicate(self): ...
```

- current_module: Returns the module of the top ModuleContext on the module stack. Raises `RuntimeError` if no module is active.
- current_body: Returns the active module body by referencing `current_module.body`.
- insert_point: Alias for `current_body`—the list where new IR nodes are appended.

- enter_context_of(module): Wraps `module` in a new ModuleContext and pushes it on the module stack.
- exit_context_of(): Pops the module context after asserting the predicate stack is balanced and returns the popped ModuleContext.

- get_predicate_stack: Returns the current module's predicate stack (empty list if no current module).
- push_predicate(cond): Pushes a predicate onto the current module's predicate stack. Used by predicate intrinsics (e.g. `Condition`).
- pop_predicate(): Pops a predicate from the current module's predicate stack. Mirrors predicate intrinsics. Asserts on underflow.

### class Singleton(metaclass=Singleton)
Holds process-wide builder state such as the active builder, indentation for __repr__, and directories excluded from source location capture.

## Section 2. Internal Helpers

### class PredicateFrame

```python
class PredicateFrame:
    cond: Value
    carry: Value
    array_cache: dict[tuple[Array, Value], ArrayRead]
    
    def get_cached_read(self, array: Array, index: Value) -> ArrayRead | None
    def cache_read(self, array: Array, index: Value, read: ArrayRead) -> None
    def has_cached_read(self, array: Array, index: Value) -> bool
```

Purpose: Encapsulates a predicate condition and its associated array-read cache. Each predicate frame stores:
- cond: The condition Value associated with this predicate frame
- carry: The cumulative `AND` of all predicate conditions from the bottom of the stack through this frame. Carry values are materialised once at push time, so callers can reuse them without recomputing chained `AND`s.
- array_cache: A dictionary mapping (array, index) tuples to cached ArrayRead operations

The cache management methods provide type-safe access to the frame's cache, abstracting away the direct dictionary access. This ensures proper encapsulation and makes the cache protocol explicit.

**Explanation:**
PredicateFrame pairs a condition with an array-read cache to ensure cache lifetime matches predicate lifetime (push/pop). When a predicate is pushed, a new empty cache is created; when popped, the entire cache is discarded. This prevents array reads created under a predicate from being reused after the predicate expires, which is essential for FSM and other conditional execution patterns. The cached carry mirrors the behaviour of [`get_pred()`](../ir/expr/intrinsic.md#get_pred) and is now the single source of truth for downstream metadata capture.

The cache is keyed by tuples of (array, index), allowing different indices into the same array to be cached separately while deduplicating identical accesses within the same predicate scope.

### Naming and Caches

SysBuilder initializes and resets:
- naming_manager: Assigns stable names to IR nodes.
- const_cache: Per-builder cache reset when entering/exiting the system context.
- Array read caching is handled per predicate frame (see PredicateFrame above).

### Context Management

- Module context is represented by ModuleContext. Isolation across modules is achieved by keeping a separate cond_stack per module-frame on the module stack. This guarantees no leakage of conditions between modules.
- Module bodies live on the module instances. The builder consults `current_module.body` to determine the insertion list, so no separate body stack or body context calls are required.

### Predicate Semantics

- get_predicate_stack returns the LIFO list of active predicate frames for the current module. Each frame exposes both the individual condition (`cond`) and the accumulated carry (`carry`).
- current_predicate_carry() exposes the top frame’s `carry`, defaulting to `Bits(1)(1)` when the stack is empty or no builder is active. IR constructors call this helper so every expression automatically captures the correct predicate metadata.
- push_predicate(cond) computes the new carry by `AND`ing the parent frame's carry with the incoming condition (or using the condition directly when the stack is empty) before pushing a new frame.
- get_pred() from intrinsic.py reuses the top frame's `carry` (or `Bits(1)(1)` if the stack is empty), avoiding repeated recomputation of the predicate chain.
- Intrinsics push_condition/pop_condition directly manipulate the predicate stack. They no longer synchronise with structural blocks—the predicate stack is the single source of truth.

### Error Handling

- Module exit asserts the module predicate stack is empty. This prevents condition leakage or imbalance.
- Predicate pop asserts the stack is non-empty.

## Notes

- The builder avoids catching exceptions except where required; assertions document invariants.
- Stack isolation keeps changes low-invasion; codegen (sim/verilog) remains unchanged and operates per-module naturally.
