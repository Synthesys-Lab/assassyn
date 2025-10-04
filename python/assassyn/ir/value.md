# Value Module

This module provides the base `Value` class that enables Python operator overloading for IR construction. It allows natural Python syntax to automatically generate corresponding IR nodes.

---

## Module Overview

The value module implements the foundation for operator overloading in assassyn's frontend. All IR value types (expressions, signals, ports) inherit from `Value` to gain automatic support for:

1. **Arithmetic Operations**: `+`, `-`, `*`, `%` generate `BinaryOp` nodes
2. **Bitwise Operations**: `|`, `^`, `&`, `~`, `<<`, `>>` generate `BinaryOp`/`UnaryOp` nodes
3. **Comparison Operations**: `<`, `>`, `<=`, `>=`, `==`, `!=` generate `BinaryOp` nodes
4. **Bit Slicing**: `[start:stop]` generates `Slice` nodes
5. **Type Conversions**: `bitcast()`, `zext()`, `sext()` generate `Cast` nodes
6. **Selection Logic**: `select()`, `case()`, `select1hot()` generate selection nodes

---

## Operator Overloading

All operator methods are decorated with `@ir_builder`, causing them to automatically inject generated IR nodes into the current block. When Python evaluates `a + b` where `a` is a `Value`, it calls `a.__add__(b)`, which creates a `BinaryOp` node representing the addition.

**Arithmetic Operators:**
- `__add__(other) -> BinaryOp`: `+` operator, generates ADD opcode
- `__sub__(other) -> BinaryOp`: `-` operator, generates SUB opcode
- `__mul__(other) -> BinaryOp`: `*` operator, generates MUL opcode
- `__mod__(other) -> BinaryOp`: `%` operator, generates MOD opcode

**Bitwise Operators:**
- `__or__(other) -> BinaryOp`: `|` operator, generates BITWISE_OR opcode
- `__xor__(other) -> BinaryOp`: `^` operator, generates BITWISE_XOR opcode
- `__and__(other) -> BinaryOp`: `&` operator, generates BITWISE_AND opcode
- `__lshift__(other) -> BinaryOp`: `<<` operator, generates SHL opcode
- `__rshift__(other) -> BinaryOp`: `>>` operator, generates SHR opcode
- `__invert__() -> UnaryOp`: `~` operator, generates FLIP opcode

**Comparison Operators:**
- `__lt__(other) -> BinaryOp`: `<` operator, generates ILT opcode
- `__gt__(other) -> BinaryOp`: `>` operator, generates IGT opcode
- `__le__(other) -> BinaryOp`: `<=` operator, generates ILE opcode
- `__ge__(other) -> BinaryOp`: `>=` operator, generates IGE opcode
- `__eq__(other) -> BinaryOp`: `==` operator, generates EQ opcode
- `__ne__(other) -> BinaryOp`: `!=` operator, generates NEQ opcode

---

## Bit Slicing

**`__getitem__(x: slice) -> Slice`** enables bit extraction using slice syntax. Only slice objects are supported (not integer indexing). The slice must have explicit `start` and `stop` values.

---

## Type Conversion Methods

**`bitcast(dtype) -> Cast`** reinterprets the bit representation as a different type without changing bits. Used for type punning between representations.

**`zext(dtype) -> Cast`** zero-extends to a wider type by padding with zeros. Used for unsigned integer widening.

**`sext(dtype) -> Cast`** sign-extends to a wider type by replicating the sign bit. Used for signed integer widening.

All three methods generate `Cast` nodes with opcodes BITCAST, ZEXT, SEXT respectively.

---

## Bit Manipulation

**`concat(other) -> Concat`** concatenates two bit vectors, creating a `Concat` node. The result places `self` in the upper bits and `other` in the lower bits.

---

## Selection Operations

**`select(true_value, false_value) -> Select`** implements ternary selection, creating a `Select` node. Returns `true_value` if `self` evaluates to true, otherwise `false_value`. Equivalent to `self ? true_value : false_value` in C.

**`case(cases: dict[Value, Value]) -> Value`** implements multi-way selection from a dictionary mapping `Value` keys to `Value` results. The `None` key is required as the default case. Internally generates nested `select()` operations.

**`select1hot(*args) -> Select1Hot`** performs one-hot selection, creating a `Select1Hot` node. `self` is a one-hot encoded selector, and `args` are the values to select from.

**`optional(default, predicate=None) -> Select`** creates an optional value that selects between `self` and `default` based on a predicate. If `predicate` is `None`, uses `self.valid()` as the condition. This method calls `select()` internally rather than being decorated with `@ir_builder` to avoid double insertion.

---

## Validity Checking

**`valid() -> PureIntrinsic`** creates a `PureIntrinsic` node with opcode VALUE_VALID to check if a value is valid. This operation is primarily meaningful in downstream modules for checking data flow validity.

---

## Key Implementation Details

- **Base Class Design**: `Value` defines no attributes—all attributes come from derived classes like `Expr`, `Port`, `Signal`
- **Cyclic Import Handling**: Uses `import-outside-toplevel` pattern to import `expr` module types inside methods, avoiding circular dependencies
- **Hash Function**: `__hash__` returns `id(self)`, enabling `Value` instances as dictionary keys based on object identity
- **Decorator Exception**: `optional()` and `case()` are not decorated with `@ir_builder` because they internally call `select()`, which already handles IR injection. Decorating them would cause duplicate node insertion
- **Type Flexibility**: Operator methods accept any type for `other`, relying on downstream type checking in `BinaryOp` construction
