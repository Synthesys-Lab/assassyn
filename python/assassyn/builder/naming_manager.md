### `NamingManager`

Coordinates the builder naming pipeline by combining the type-oriented namer
with the rewritten assignment hook.

#### Methods

##### `__init__(self)`
Creates a fresh `TypeOrientedNamer` and per-instance module-name cache.

##### `push_value(self, value: Any)`
Best-effort names freshly created IR values. `Expr` instances receive an
immediate semantic name via `TypeOrientedNamer`, stored on
`__assassyn_semantic_name__` when possible, so they remain readable even before
assignments happen.

##### `process_assignment(self, name: str, value: Any) -> Any`
Implements the runtime side of rewritten assignments:
1. The final value receives a name seeded with the Python assignment target.
2. The original value is returned so Python assignment semantics are preserved.

##### `assign_name(self, value: Any, hint: Optional[str] = None) -> str`
Exposes semantic naming for non-expression objects (modules, arrays, etc.).
Applies the hint when provided; otherwise falls back to type-based naming.

##### `_apply_name(self, value: Any, name: str)`
Best-effort helper that writes the semantic name to the value using
`setattr(value, "__assassyn_semantic_name__", name)` while ignoring types that
cannot be annotated.

##### `get_module_name(self, base_name: str) -> str`
Capitalises the supplied base name and feeds it through a `UniqueNameCache` to
guarantee unique module identifiers for the experimental builder front-ends.

##### `get_context_prefix(self) -> Optional[str]`
Returns the current hierarchical naming context based on the active module stack.
When inside a module's `build()` method, this returns the module instance's name
to be used as a prefix for arrays and other entities created within that context.

-----

## Context-aware array naming

When arrays are created without an explicit name inside a module body, a hierarchical hint is applied so textual IR reflects structure:

- arrays declared via `RegArray` receive a hint of the form `<module_name>_array` if no better hint is present
- the hint is applied through `NamingManager.assign_name`, which writes a semantic name on the instance
- explicit names always take precedence; names with underscores that indicate hierarchy are preserved

---

## Global Functions

### `get_naming_manager() -> Optional[NamingManager]`
Returns the process-global naming manager instance if one has been registered.

### `set_naming_manager(manager: Optional[NamingManager])`
Registers or clears the global naming manager reference used by decorators and
assignment hooks.
