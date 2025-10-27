# Builder Module (__init__.py)

## Section 0. Summary

This module implements the system-wide IR builder (SysBuilder) and the global Singleton used to manage building context (modules, blocks, naming, and caches). It also defines the ModuleContext record that holds per-module state, including the per-module predicate stack used by conditional execution.

Starting from this change, the predicate stack is isolated per module via ModuleContext. Both conditional blocks (CondBlock) and predicate intrinsics (push_condition/pop_condition) manipulate the same builder-managed predicate stack of the current module context.

## Section 1. Exposed Interfaces

### class ModuleContext
```python
class ModuleContext:
    module: Module
    cond_stack: list
```

Purpose: Encapsulates per-module builder state requiring stack semantics. Currently contains the owning module and its cond_stack (predicate stack).

- module: The module object associated with this context frame.
- cond_stack: The predicate stack for this module context. The stack holds condition Value objects in LIFO order.

### class SysBuilder
Core builder that also represents a system under construction. Key exposed properties and methods:

```python
class SysBuilder:
    @property
    def current_module(self): ...

    @property
    def current_block(self): ...

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
- current_block: Returns the top block on the block stack; None if not inside a block.
- insert_point: The list of IR nodes where new IR will be appended (the current block body).

- enter_context_of('module' | 'block', entry): Pushes context. For 'module', wraps entry in a new ModuleContext and pushes it on the module stack. For 'block', pushes the block and initializes per-block caches. When entry is a CondBlock, the builder records the condition as external and pushes it to the current module's predicate stack.
- exit_context_of('module' | 'block'): Pops context. For 'module', asserts the module's predicate stack is empty on exit. For 'block', clears per-block caches. When leaving a CondBlock, pops a predicate.

- get_predicate_stack: Returns the current module's predicate stack (empty list if no current module).
- push_predicate(cond): Pushes a predicate onto the current module's predicate stack. Used by both CondBlock entry and the push_condition intrinsic.
- pop_predicate(): Pops a predicate from the current module's predicate stack. Used by both CondBlock exit and the pop_condition intrinsic. Asserts on underflow.

### class Singleton(metaclass=Singleton)
Holds process-wide builder state such as the active builder, indentation for __repr__, and directories excluded from source location capture.

## Section 2. Internal Helpers

### Naming and Caches

SysBuilder initializes and resets:
- naming_manager: Assigns stable names to IR nodes.
- const_cache, array_read_cache: Per-builder caches reset when entering/exiting the system context, and on block enter/exit (for array_read_cache).

### Context Management

- Module context is represented by ModuleContext. Isolation across modules is achieved by keeping a separate cond_stack per module-frame on the module stack. This guarantees no leakage of conditions between modules.
- Conditional blocks are recognized at builder context entry/exit. The builder informs the owning module about external operands and synchronizes the predicate stack via push_predicate/pop_predicate.

### Predicate Semantics

- get_predicate_stack returns the LIFO list of active conditions for the current module. get_pred() from intrinsic.py computes the logical AND of all active conditions (or Bits(1)(1) if empty).
- Intrinsics push_condition/pop_condition mirror block entry/exit to keep the predicate stack consistent when conditions are structured either as blocks or as explicit intrinsics.

### Error Handling

- Module exit asserts the module predicate stack is empty. This prevents condition leakage or imbalance.
- Predicate pop asserts the stack is non-empty.

## Notes

- The builder avoids catching exceptions except where required; assertions document invariants.
- Stack isolation keeps changes low-invasion; codegen (sim/verilog) remains unchanged and operates per-module naturally.
