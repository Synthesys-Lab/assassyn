# integrate.py

Integration layer that wires the naming system into Assassyn's IR builder,
combinational decorators, and the `SysBuilder` context manager.

## Functions

### `ir_builder(func=None, *, node_type=None)`

Drop-in replacement for the original `@ir_builder` decorator.

#### Parameters
- `func`: Function to decorate (or `None` when used with keyword arguments)
- `node_type`: Optional IR node type propagated to the wrapper

#### Behaviour
1. Calls the original builder function and immediately returns when the result
   is `None`.
2. When the result is an `Expr`, pushes it to the active `NamingManager` so its
   eventual assignment can be tracked.
3. Reproduces the legacy insertion logic (setting parents, registering external
   operands, and inserting into the current block) for non-constant results.
4. Walks up the call stack, skipping project-internal frames, to record
   `res.loc` for source location tracking.

The wrapper preserves the original decorator metadata (`_ir_builder_node_type`),
so downstream consumers continue to behave as before.

---

### `combinational_for(module_class)`

Factory returning a combinational decorator that understands the naming system.

#### AST rewriting
- Source is retrieved with `inspect.getsource`, dedented, parsed, and rewritten
  by `rewrite_assign`.
- The generated function replaces simple assignments with
  `__assassyn_assignment__` calls and clears the decorator list to avoid
  stacking combinational wrappers.
- Compilation happens inside the original `func.__globals__` so later
  definitions remain visible; the temporary `__assassyn_assignment__` entry is
  restored to its previous value when one existed beforehand.
- If rewriting fails for any reason, the original function is used and a warning
  is emitted.

#### Wrapper behaviour
- Creates a fresh `Block` rooted at `Block.MODULE_ROOT`, attaches it to the
  module (`module.body.parent = module`, `module.body.module = module`), and
  manages the usual builder contexts.
- Binds the rewritten function's signature so `Array` arguments inherit the
  corresponding parameter names for clearer IR dumps.
- Marks the wrapper with `_is_combinational`, `_module_class`, and stores the
  rewritten function in `__assassyn_original__` for debugging/reference.

---

### `install_decorators()`

Monkey-patches Assassyn so every consumer sees the enhanced decorators.

---

### `sys_builder()`

Augments `SysBuilder` so that entering/leaving its context also manages the
global naming manager.

```python
def new_enter(self):
    result = original_enter(self)
    set_naming_manager(self.naming_manager)
    return result

def new_exit(self, exc_type, exc_value, traceback):
    set_naming_manager(None)
    return original_exit(self, exc_type, exc_value, traceback)
```

This ensures the naming manager is available while the builder is active and
released afterwards, regardless of exceptions.

---

### `initialize_naming_system()`

Convenience entry point that installs the decorators and patches `SysBuilder`
so the naming manager is available while the builder is active.

---

## Context Management Summary

The enhanced combinational decorator coordinates three layers simultaneously:

```python
Singleton.builder.enter_context_of('module', module)
Singleton.builder.enter_context_of('block', module.body)

Singleton.builder.exit_context_of('block')
Singleton.builder.exit_context_of('module')
```
