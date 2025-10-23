# Type-Oriented Namer

This module provides the `TypeOrientedNamer` class, which generates semantically meaningful names for IR nodes based on their types and operations. It is a core component of the Assassyn naming system, working in conjunction with `UniqueNameCache` to ensure unique, readable identifiers for all IR values.

## Section 0. Summary

The TypeOrientedNamer implements a sophisticated naming strategy that analyzes IR node types, operations, and operands to generate descriptive identifiers. It uses a unified strategy pattern with `{class: lambda node: ...}` mappings that dynamically extract naming information from IR expr classes using their OPERATORS dictionaries. This eliminates hard-coded opcode mappings and provides a maintainable, extensible naming system that works with the broader naming system described in [naming_manager.md](naming_manager.md) and ensures compatibility with Rust naming conventions.

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

Initializes a new `TypeOrientedNamer` instance with a `UniqueNameCache` and a unified naming strategies dictionary.

**Explanation:** This constructor sets up the internal state needed for type-based naming. It creates a `UniqueNameCache` instance for generating unique names and initializes a `_naming_strategies` dictionary that maps IR expr classes to lambda functions. Each lambda function extracts naming information dynamically from the node's OPERATORS dictionary and operand analysis, eliminating the need for hard-coded opcode mappings.

#### `get_prefix_for_type`

```python
def get_prefix_for_type(self, node: Any) -> str:
```

Extracts a descriptive base name for a given IR node using a unified strategy pattern.

**Parameters:**
- `node`: The IR node to generate a prefix for

**Returns:**
- A sanitized string prefix that describes the node's type and operation

**Explanation:** This method implements a streamlined naming strategy using the strategy pattern:

1. **Module instances**: Nodes with `ModuleBase` in their MRO get "Instance" suffix (e.g., `AdderInstance`)
2. **Strategy lookup**: Uses the `_naming_strategies` dictionary to find the appropriate lambda function for the node's class
3. **Dynamic extraction**: Each strategy function dynamically extracts naming information from the node's OPERATORS dictionary and operand analysis
4. **Fallback**: Falls back to the node's `name` attribute or `"val"` for unrecognized types

The method uses direct attribute access since all `Value` subclasses now have a unified `name` attribute and applies `_sanitize` to ensure valid identifiers.

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

**Explanation:** This method uses `object.__getattribute__` directly to bypass any custom `__getattr__` methods that might have side effects. It catches `AttributeError` and `TypeError` exceptions and returns `None` in those cases, allowing the naming system to gracefully handle missing attributes. This method is used for dynamic or uncertain attribute access; well-defined IR classes like `BinaryOp`, `UnaryOp`, and `PureIntrinsic` have their attributes accessed directly since they use proper `@property` methods.

### `_entity_name`

```python
def _entity_name(self, entity: Any) -> Optional[str]:
```

Extracts a meaningful name from an entity, checking for semantic names and regular name attributes.

**Parameters:**
- `entity`: The entity to extract a name from

**Returns:**
- A sanitized name string or `None` if no name is found

**Explanation:** This method first unwraps any operand wrappers using `_unwrap_operand`, then checks for the unified `name` attribute. All names are sanitized before being returned.

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

**Explanation:** This method takes multiple optional string parts, filters out `None` and empty strings, and combines them with underscores. It removes duplicate adjacent tokens for clarity and limits the result to 25 characters to avoid unreadable names. The method trusts that callers (via `_describe_operand` and `_entity_name`) have already provided sanitized and segmented inputs, eliminating redundant processing. This is used to create descriptive names like `lhs_add_rhs` for binary operations.

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

### `_symbol_to_name`

```python
@staticmethod
def _symbol_to_name():
```

Converts operator symbols to descriptive names for use in generated identifiers.

**Returns:**
- A dictionary mapping operator symbols to descriptive names

**Explanation:** This static method provides a mapping from operator symbols (like '+', '&', '<') to descriptive names (like 'add', 'and', 'lt') that are suitable for use in generated identifiers. This allows the naming system to convert symbolic operators from the OPERATORS dictionaries into readable identifier components.

### `_binary_op_strategy`

```python
def _binary_op_strategy(self, node: Any) -> str:
```

Strategy function for binary operations that extracts naming information from BinaryOp.OPERATORS.

**Parameters:**
- `node`: The binary operation node to name

**Returns:**
- A descriptive name combining operand descriptions with the operation name

**Explanation:** This strategy function looks up the operation symbol in `BinaryOp.OPERATORS`, converts it to a descriptive name using `_symbol_to_name()`, and combines it with descriptions of the left and right operands to create names like `lhs_add_rhs`. The method accesses `node.opcode`, `node.lhs`, and `node.rhs` directly since these are well-defined `@property` methods in the `BinaryOp` class.

### `_unary_op_strategy`

```python
def _unary_op_strategy(self, node: Any) -> str:
```

Strategy function for unary operations that extracts naming information from UnaryOp.OPERATORS.

**Parameters:**
- `node`: The unary operation node to name

**Returns:**
- A descriptive name combining the operation name with operand description

**Explanation:** This strategy function looks up the operation symbol in `UnaryOp.OPERATORS`, converts it to a descriptive name using `_symbol_to_name()`, and combines it with the operand description to create names like `neg_operand`. The method accesses `node.opcode` and `node.x` directly since these are well-defined `@property` methods in the `UnaryOp` class.

### `_pure_intrinsic_strategy`

```python
def _pure_intrinsic_strategy(self, node: Any) -> str:
```

Strategy function for pure intrinsics that extracts naming information from PureIntrinsic.OPERATORS.

**Parameters:**
- `node`: The pure intrinsic node to name

**Returns:**
- A descriptive name based on the intrinsic operation and its arguments

**Explanation:** This strategy function looks up the operation name in `PureIntrinsic.OPERATORS` and combines it with argument names when appropriate. For FIFO operations like 'peek' and 'valid', it creates names like `fifo_name_peek` or `fifo_name_valid`. The method accesses `node.opcode` and `node.args` directly since these are well-defined `@property` methods in the `PureIntrinsic` class.

## Strategy Pattern System

The `TypeOrientedNamer` uses a unified strategy pattern with `{class: lambda node: str}` mappings to generate semantic prefixes for operations:

- **Dynamic Extraction**: All opcode information is extracted from source classes using their OPERATORS dictionaries
- **Unified Interface**: All naming strategies follow the same `{class: lambda node: str}` pattern
- **Strategy Functions**: Specialized functions like `_binary_op_strategy`, `_unary_op_strategy`, and `_pure_intrinsic_strategy` handle operations with OPERATORS dictionaries
- **Lambda Strategies**: Simple lambda functions handle classes without OPERATORS dictionaries (e.g., ArrayRead, FIFOPop)
- **Symbol Conversion**: The `_symbol_to_name()` method converts operator symbols to descriptive names

This approach eliminates hard-coded opcode mappings and provides a maintainable, extensible naming system where adding new expr types only requires adding one lambda to the strategies dictionary.

## Operand Wrapping System

The naming system handles operand wrappers through the `_unwrap_operand` method:

- **Purpose**: Some IR operands may be wrapped in special wrapper objects for type safety or other purposes
- **Implementation**: The method attempts to import and use `assassyn.utils.unwrap_operand` to extract the underlying operand
- **Fallback**: If the unwrapping utility is not available or fails, the original entity is returned unchanged
- **Usage**: This ensures that naming works consistently whether operands are wrapped or unwrapped

This system allows the naming infrastructure to work with both wrapped and unwrapped operands transparently, ensuring consistent name generation regardless of the operand representation.