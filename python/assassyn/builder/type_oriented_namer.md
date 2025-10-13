# Type-Oriented Namer

This module provides the `TypeOrientedNamer` class, which generates semantically meaningful names for IR nodes based on their types and operations. It is a core component of the Assassyn naming system, working in conjunction with `UniqueNameCache` to ensure unique, readable identifiers for all IR values.

## Section 0. Summary

The TypeOrientedNamer implements a sophisticated naming strategy that analyzes IR node types, operations, and operands to generate descriptive identifiers. It uses hardcoded opcode mappings for arithmetic and logic operations, class-based prefixes for specific IR constructs, and operand analysis to create meaningful names like `lhs_add_rhs` for binary operations or `neg_operand` for unary operations. The namer is designed to work with the broader naming system described in [naming_manager.md](naming_manager.md) and ensures compatibility with Rust naming conventions.

## Section 1. Exposed Interfaces

### `TypeOrientedNamer`

```python
class TypeOrientedNamer:
```

The main class that provides type-aware naming for IR nodes.

#### `__init__`

```python
def __init__(self) -> None:
```

Initializes a new `TypeOrientedNamer` instance with a `UniqueNameCache` and lookup tables for operation prefixes.

**Explanation:** This constructor sets up the internal state needed for type-based naming. It creates a `UniqueNameCache` instance for generating unique names and initializes three lookup dictionaries:
- `_binary_ops`: Maps binary operation opcodes to semantic prefixes (e.g., 200→'add', 201→'sub')
- `_unary_ops`: Maps unary operation opcodes to semantic prefixes (e.g., 100→'neg', 101→'not')  
- `_class_prefixes`: Maps IR class names to semantic prefixes (e.g., 'ArrayRead'→'rd', 'FIFOPop'→'pop')

#### `get_prefix_for_type`

```python
def get_prefix_for_type(self, node: Any) -> str:
```

Extracts a descriptive base name for a given IR node based on its type and structure.

**Parameters:**
- `node`: The IR node to generate a prefix for

**Returns:**
- A sanitized string prefix that describes the node's type and operation

**Explanation:** This method implements a hierarchical naming strategy that checks node types in order of specificity:

1. **Module instances**: Nodes with `ModuleBase` in their MRO get "Instance" suffix (e.g., `AdderInstance`)
2. **Pure intrinsics**: Uses the intrinsic's opcode and first argument name if available
3. **Class-based mappings**: Uses predefined prefixes for specific IR classes like `ArrayRead`, `ArrayWrite`, `FIFOPop`, etc.
4. **Binary operations**: Combines operand descriptions with operation tokens (e.g., `lhs_add_rhs`)
5. **Unary operations**: Combines operation token with operand description (e.g., `neg_operand`)
6. **Special operations**: Handles `Cast`, `Slice`, `Concat`, `Select`, `Select1Hot` with descriptive prefixes
7. **Name attribute**: Falls back to the node's `name` attribute if present
8. **Default fallback**: Returns `"val"` for unrecognized types

The method uses `_safe_getattr` to avoid triggering `__getattr__` side effects and applies `_sanitize` to ensure valid identifiers.

#### `name_value`

```python
def name_value(self, value: Any, hint: Optional[str] = None) -> str:
```

Generates a unique name for a value, either using an explicit hint or deriving one from the value's type.

**Parameters:**
- `value`: The IR value to name
- `hint`: Optional explicit name hint to use instead of type-based naming

**Returns:**
- A unique identifier string

**Explanation:** This is the main entry point for generating unique names. If a hint is provided, it sanitizes the hint and uses the `UniqueNameCache` to ensure uniqueness. Otherwise, it calls `get_prefix_for_type` to derive a type-based prefix and then uses the cache to make it unique. The cache ensures that subsequent calls with the same prefix get numbered suffixes (`foo`, `foo_1`, `foo_2`, etc.).

## Section 2. Internal Helpers

### `_sanitize`

```python
@staticmethod
def _sanitize(text: str) -> str:
```

Converts text into a valid identifier-like token by replacing non-alphanumeric characters with underscores.

**Parameters:**
- `text`: The input text to sanitize

**Returns:**
- A sanitized string suitable for use as an identifier

**Explanation:** This static method ensures that generated names are valid identifiers by replacing any sequence of non-alphanumeric characters with a single underscore and trimming leading/trailing underscores. If the result is empty, it returns `'val'` as a fallback.

### `_safe_getattr`

```python
@staticmethod
def _safe_getattr(node: Any, attr: str) -> Optional[Any]:
```

Safely retrieves an attribute from a node without triggering `__getattr__` side effects.

**Parameters:**
- `node`: The object to get the attribute from
- `attr`: The attribute name to retrieve

**Returns:**
- The attribute value or `None` if not found or if an error occurs

**Explanation:** This method uses `object.__getattribute__` directly to bypass any custom `__getattr__` methods that might have side effects. It catches `AttributeError` and `TypeError` exceptions and returns `None` in those cases, allowing the naming system to gracefully handle missing attributes.

### `_entity_name`

```python
def _entity_name(self, entity: Any) -> Optional[str]:
```

Extracts a meaningful name from an entity, checking for semantic names and regular name attributes.

**Parameters:**
- `entity`: The entity to extract a name from

**Returns:**
- A sanitized name string or `None` if no name is found

**Explanation:** This method first unwraps any operand wrappers using `_unwrap_operand`, then checks for the special `__assassyn_semantic_name__` attribute (used by the naming system), and falls back to a regular `name` attribute. All names are sanitized before being returned.

### `_module_prefix`

```python
def _module_prefix(self, node: Any) -> str:
```

Generates a prefix for Module-like objects with "Instance" suffix.

**Parameters:**
- `node`: The module node to generate a prefix for

**Returns:**
- A string prefix with "Instance" suffix

**Explanation:** This method takes the class name of the node, sanitizes it, and appends "Instance" to create module instance names. It handles special cases where the base name is 'module' or 'modulebase' by normalizing to 'module'. Module instances follow PascalCase naming with "Instance" suffix to ensure compatibility with Rust naming conventions and eliminate compiler warnings.

### `_unwrap_operand`

```python
def _unwrap_operand(self, entity: Any) -> Any:
```

Unwraps Operand wrappers when the `unwrap_operand` utility is available.

**Parameters:**
- `entity`: The entity to potentially unwrap

**Returns:**
- The unwrapped entity or the original entity if unwrapping fails

**Explanation:** This method attempts to import and use the `unwrap_operand` function from `assassyn.utils`. If the import fails or the function is not available, it returns the original entity unchanged. This allows the naming system to work with both wrapped and unwrapped operands.

### `_describe_operand`

```python
def _describe_operand(self, operand: Any) -> Optional[str]:
```

Provides a descriptive token for an operand by extracting its name and taking the first two segments.

**Parameters:**
- `operand`: The operand to describe

**Returns:**
- A descriptive token or `None` if no name is found

**Explanation:** This method calls `_entity_name` to get the operand's name, then uses `_head_token_segment` to keep only the first two underscore-separated segments for brevity. This helps keep generated names concise while maintaining meaningful information.

### `_combine_parts`

```python
def _combine_parts(self, *parts: Optional[str]) -> Optional[str]:
```

Combines multiple name parts into a single sanitized identifier.

**Parameters:**
- `*parts`: Variable number of optional string parts to combine

**Returns:**
- A combined identifier string or `None` if no valid parts are provided

**Explanation:** This method takes multiple optional string parts, filters out `None` and empty strings, sanitizes each part, and combines them with underscores. It removes duplicate adjacent tokens for clarity and limits the result to 25 characters to avoid unreadable names. This is used to create descriptive names like `lhs_add_rhs` for binary operations.

### `_head_token_segment`

```python
@staticmethod
def _head_token_segment(token: str) -> str:
```

Keeps at most the first two underscore-separated segments of a token for brevity.

**Parameters:**
- `token`: The token to truncate

**Returns:**
- The first two segments of the token, joined by underscores

**Explanation:** This static method splits the input token by underscores and keeps only the first two segments, rejoining them with underscores. This helps keep generated names concise while preserving the most important identifying information.

## Opcode Mapping System

The `TypeOrientedNamer` uses hardcoded opcode mappings to generate semantic prefixes for operations:

- **Binary Operations**: Mapped to descriptive tokens (e.g., 200→'add', 201→'sub', 202→'mul')
- **Unary Operations**: Mapped to operation tokens (e.g., 100→'neg', 101→'not')
- **Class-based Prefixes**: Direct mappings for specific IR classes (e.g., 'ArrayRead'→'rd', 'FIFOPop'→'pop')

These opcodes are specific to the IR expression system and are used to generate meaningful names like `lhs_add_rhs` for binary operations or `neg_operand` for unary operations. The mappings ensure that generated names reflect the semantic meaning of operations rather than using generic identifiers.

## Operand Wrapping System

The naming system handles operand wrappers through the `_unwrap_operand` method:

- **Purpose**: Some IR operands may be wrapped in special wrapper objects for type safety or other purposes
- **Implementation**: The method attempts to import and use `assassyn.utils.unwrap_operand` to extract the underlying operand
- **Fallback**: If the unwrapping utility is not available or fails, the original entity is returned unchanged
- **Usage**: This ensures that naming works consistently whether operands are wrapped or unwrapped

This system allows the naming infrastructure to work with both wrapped and unwrapped operands transparently, ensuring consistent name generation regardless of the operand representation.