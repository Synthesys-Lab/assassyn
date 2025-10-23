# Naming Manager

The NamingManager coordinates the builder naming pipeline by combining the type-oriented namer with the rewritten assignment hook. It provides semantic naming for IR values and manages hierarchical naming contexts.

## Section 1. Exposed Interfaces

### class NamingManager

Central coordinator for the naming system, integrating type-based naming with AST rewriting hooks.

#### NamingManager.__init__

```python
def __init__(self):
```

Creates a fresh `TypeOrientedNamer` and per-instance module-name cache.

#### NamingManager.push_value

```python
def push_value(self, value: Any):
```

Best-effort names freshly created IR values. `Expr` instances receive an immediate semantic name via `TypeOrientedNamer`, stored on the unified `name` attribute when possible, so they remain readable even before assignments happen.

**Explanation**: This method is called by the `ir_builder` decorator when new IR expressions are created. It attempts to name `Expr` objects immediately based on their type, which improves IR readability during debugging and code generation. The method uses a try-catch block to handle cases where the `Expr` import fails or the object cannot be annotated.

#### NamingManager.process_assignment

```python
def process_assignment(self, name: str, value: Any) -> Any:
```

Implements the runtime side of rewritten assignments. The final value receives a name seeded with the Python assignment target, and the original value is returned so Python assignment semantics are preserved.

**Explanation**: This method is called by the AST rewriting system through `__assassyn_assignment__` function in [rewrite_assign.md](rewrite_assign.md). When Python assignments like `x = some_expr` are rewritten to `x = __assassyn_assignment__("x", some_expr)`, this method processes the naming. It uses the assignment target name as a hint for the `TypeOrientedNamer`, then applies the generated name to the value using the unified `name` attribute.

#### NamingManager.assign_name

```python
def assign_name(self, value: Any, hint: Optional[str] = None) -> str:
```

Exposes semantic naming for non-expression objects (modules, arrays, etc.). Applies the hint when provided; otherwise falls back to type-based naming.

**Explanation**: This method is used by modules and arrays to assign semantic names. It's called when modules are created and for context-aware array naming. The method generates a unique name using the `TypeOrientedNamer` and applies it to the object via the unified `name` attribute. This provides a public interface for naming non-expression objects that participate in the IR.

#### NamingManager.get_module_name

```python
def get_module_name(self, base_name: str) -> str:
```

Capitalizes the supplied base name and feeds it through a `UniqueNameCache` to guarantee unique module identifiers for the experimental builder front-ends.

**Explanation**: This method is used by the experimental frontend factory functions to generate unique module names. It ensures that modules created from the same base name (like function names) get unique identifiers to avoid naming conflicts. The method capitalizes the base name and uses a `UniqueNameCache` to guarantee uniqueness.

#### NamingManager.get_context_prefix

```python
def get_context_prefix(self) -> Optional[str]:
```

Returns the current hierarchical naming context based on the active module stack. When inside a module's `build()` method, this returns the module instance's name to be used as a prefix for arrays and other entities created within that context.

**Explanation**: This method accesses the global `Singleton.builder` to get the current module context. It's used to provide hierarchical naming hints for arrays and other entities created within modules. The method gets the module's name directly from the `name` attribute. This enables context-aware naming where entities inherit their parent module's name as a prefix.

### get_naming_manager

```python
def get_naming_manager() -> Optional[NamingManager]:
```

Returns the process-global naming manager instance if one has been registered.

### set_naming_manager

```python
def set_naming_manager(manager: Optional[NamingManager]):
```

Registers or clears the global naming manager reference used by decorators and assignment hooks.

## Section 2. Internal Helpers

### NamingManager._apply_name

```python
def _apply_name(self, value: Any, name: str):
```

Best-effort helper that writes the semantic name to the value using `setattr(value, "name", name)` while ignoring types that cannot be annotated.

**Explanation**: This internal method handles the actual application of semantic names to objects. It uses a try-catch block to handle cases where objects cannot be annotated (like Python builtins). The semantic name is stored in the unified `name` attribute for consistent access across all IR nodes.

## Unified Name Attribute System

The naming system uses a unified `name` attribute on the `Value` base class to store semantic names on IR objects:

- **Purpose**: Provides a standardized way to attach human-readable names to IR values with a consistent interface across all IR nodes
- **Lifecycle**: Names are assigned when objects are created or when assignments are processed
- **Usage**: The attribute is checked by `TypeOrientedNamer._entity_name()` to extract meaningful names for generating descriptive identifiers
- **Fallback**: If the name is not available, the system falls back to type-based naming or generates default identifiers

## Context-aware Array Naming

When arrays are created without an explicit name inside a module body, a hierarchical hint is applied so textual IR reflects structure:

- Arrays declared via `RegArray` receive a hint of the form `<module_name>_array` if no better hint is present
- The hint is applied through `NamingManager.assign_name`, which writes a semantic name on the instance using the unified `name` attribute
- Explicit names always take precedence; names with underscores that indicate hierarchy are preserved
