# Builder Module (__init__.py)

## Section 0. Summary

This module implements the system-wide IR builder (SysBuilder) and the global Singleton used to manage building context (modules, module bodies, naming, and caches). It also defines the ModuleContext record that holds per-module state, including the per-module predicate stack used by predicate intrinsics.

Module bodies are plain Python lists; the builder keeps a `body` stack to track the current insertion list. Predicate push/pop intrinsics (emitted by `Condition`) rely on the per-module predicate stack maintained here.

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

    def enter_context_of(self, ty, entry): ...
    def exit_context_of(self, ty): ...

    # Predicate helpers (per current module)
    def get_predicate_stack(self): ...
    def push_predicate(self, cond): ...
    def pop_predicate(self): ...
```

- current_module: Returns the module of the top ModuleContext on the module stack; None if no module is active.
- current_body: Returns the active module body list (top of the body stack); None if no body is active.
- insert_point: Alias for `current_body`—the list where new IR nodes are appended.

- enter_context_of('module' | 'body', entry): Pushes context. For 'module', wraps entry in a new ModuleContext and pushes it on the module stack. For 'body', pushes the list object used as the current insertion body.
- exit_context_of('module' | 'body'): Pops the corresponding context. Module exit asserts the predicate stack is balanced before releasing the frame; body exit simply drops the insertion list.

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
    array_cache: dict[tuple[Array, Value], ArrayRead]
    
    def get_cached_read(self, array: Array, index: Value) -> ArrayRead | None
    def cache_read(self, array: Array, index: Value, read: ArrayRead) -> None
    def has_cached_read(self, array: Array, index: Value) -> bool
```

Purpose: Encapsulates a predicate condition and its associated array-read cache. Each predicate frame stores:
- cond: The condition Value associated with this predicate frame
- array_cache: A dictionary mapping (array, index) tuples to cached ArrayRead operations

The cache management methods provide type-safe access to the frame's cache, abstracting away the direct dictionary access. This ensures proper encapsulation and makes the cache protocol explicit.

**Explanation:**
PredicateFrame pairs a condition with an array-read cache to ensure cache lifetime matches predicate lifetime (push/pop). When a predicate is pushed, a new empty cache is created; when popped, the entire cache is discarded. This prevents array reads created under a predicate from being reused after the predicate expires, which is essential for FSM and other conditional execution patterns.

The cache is keyed by tuples of (array, index), allowing different indices into the same array to be cached separately while deduplicating identical accesses within the same predicate scope.

### Naming and Caches

SysBuilder initializes and resets:
- naming_manager: Assigns stable names to IR nodes.
- const_cache: Per-builder cache reset when entering/exiting the system context.
- Array read caching is handled per predicate frame (see PredicateFrame above).

### Context Management

- Module context is represented by ModuleContext. Isolation across modules is achieved by keeping a separate cond_stack per module-frame on the module stack. This guarantees no leakage of conditions between modules.
- Module bodies are tracked via the `body` stack. Entering a module body pushes the list owned by the module; exiting restores the previous insertion list (if any). Since predicate scopes are encoded via explicit intrinsics, there is no separate block structure to manage.

### Predicate Semantics

- get_predicate_stack returns the LIFO list of active conditions for the current module. get_pred() from intrinsic.py computes the logical AND of all active conditions (or Bits(1)(1) if empty).
- Intrinsics push_condition/pop_condition directly manipulate the predicate stack. They no longer synchronise with structural blocks—the predicate stack is the single source of truth.

### Error Handling

- Module exit asserts the module predicate stack is empty. This prevents condition leakage or imbalance.
- Predicate pop asserts the stack is non-empty.

## Notes

- The builder avoids catching exceptions except where required; assertions document invariants.
- Stack isolation keeps changes low-invasion; codegen (sim/verilog) remains unchanged and operates per-module naturally.
