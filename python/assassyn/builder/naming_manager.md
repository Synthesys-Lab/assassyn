### `NamingManager`

Coordinates the builder naming pipeline by combining the assignment tracker,
type-oriented namer, and the rewritten assignment hook.

#### Methods

##### `__init__(self)`
Creates a fresh `AssignmentTracker`, `TypeOrientedNamer`, and per-instance
module-name cache. It also initialises an empty list for values pending
assignment and a boolean gate controlling whether the assignment hook is active.

##### `push_value(self, value: Any)`
Records a value for later processing when the assignment hook is enabled.
Independently of the hook, `Expr` instances receive an immediate semantic name
via `TypeOrientedNamer`, stored on `__assassyn_semantic_name__` if possible, so
they remain readable even before assignments happen.

##### `process_assignment(self, name: str, value: Any) -> Any`
Implements the runtime side of rewritten assignments:
1. Intermediate pending values are popped in creation order and named using the
   type-oriented namer.
2. The final value receives a name seeded with the Python assignment target.
3. The `AssignmentTracker` is asked to clear its queue so `_name`/`name`
   attributes reflect the generated identifiers.
4. The original value is returned so Python assignment semantics are preserved.

##### `_apply_name(self, value: Any, name: str)`
Best-effort helper that writes the semantic name to the value using
`setattr(value, "__assassyn_semantic_name__", name)` while ignoring types that
cannot be annotated.

##### `reset(self)`
Resets the tracker, the type-oriented namer cache, and clears any pending
values.

##### `enable_assignment_hook(self)` / `disable_assignment_hook(self)`
Flip the internal `_assignment_hook_enabled` flag that controls whether
`push_value` queues values for deferred naming.

##### `get_module_name(self, base_name: str) -> str`
Capitalises the supplied base name and feeds it through a `UniqueNameCache` to
guarantee unique module identifiers for the experimental builder front-ends.

---

## Global Functions

### `get_naming_manager() -> Optional[NamingManager]`
Returns the process-global naming manager instance if one has been registered.

### `set_naming_manager(manager: Optional[NamingManager])`
Registers or clears the global naming manager reference used by decorators and
assignment hooks.

### `__assassyn_assignment__(name: str, value: Any) -> Any`
Entry point invoked by rewritten assignments. Implemented in
`rewrite_assign.py`, it queries the active naming manager (via
`get_naming_manager`) to run `process_assignment` and returns the processed
value. When no manager is active it simply returns the value unchanged.

### `name_ir_node(node: Any, hint: Optional[str] = None, namer: Optional[TypeOrientedNamer] = None) -> str`
Convenience helper that names an IR node using the supplied (or freshly created)
`TypeOrientedNamer`.
