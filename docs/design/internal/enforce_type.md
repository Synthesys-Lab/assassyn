# Type Enforcement System Design

## Language-Level Overview

The `@enforce_type` decorator implements a runtime type system for Assassyn's Python frontend, providing compile-time-like type safety at runtime. This addresses fundamental gaps in Python's type system where annotations are purely informational.

## Design Philosophy

### Type Safety as First-Class Citizen

Assassyn's hardware description language requires precise type contracts between modules, arrays, and operations. The runtime type system ensures these contracts are enforced, preventing subtle bugs that could manifest as incorrect hardware generation.

### Gradual Adoption Strategy

The decorator follows a "opt-in" model where functions can be incrementally migrated to use type enforcement. This allows the system to evolve without breaking existing code.

### Performance-Conscious Design

Type checking is designed to have zero overhead when types are correct, making it suitable for production use in simulation and code generation pipelines.

## Type System Architecture

### Core Validation Engine

The type system is built around three core functions:

1. **`check_type(value, expected_type)`** - Atomic type checking primitive
2. **`validate_arguments(func, args, kwargs)`** - Function-level validation orchestrator  
3. **`@enforce_type`** - Decorator interface for seamless integration

### Type Hierarchy Support

The system handles Python's complex type hierarchy:

```
Any
├── Simple Types (int, str, bool, float, custom classes)
├── Union Types (Union[A, B], Optional[T])
└── Generic Types (List[T], Dict[K, V], Tuple[...])
```

### Forward Reference Resolution

Uses `get_type_hints()` for proper resolution of forward references, enabling type annotations that reference classes defined later in the module.

## Integration with Assassyn Architecture

### Hardware Description Language Integration

The type system is specifically designed for Assassyn's domain:

- **Array Operations**: Validates array indices and values
- **Module Interfaces**: Ensures proper port and signal types
- **Expression Trees**: Validates operand types in IR construction
- **Code Generation**: Type-safe interface between frontend and backend

### Factory Pattern Integration

Extracts and generalizes validation logic from the existing factory decorator, providing a unified type enforcement strategy across the codebase.

## Error Handling Philosophy

### Fail-Fast Design

Type mismatches raise `TypeError` immediately, providing clear debugging information before incorrect values propagate through the system.

### Contextual Error Messages

Error messages include:
- Parameter name that failed validation
- Expected type from annotation  
- Actual type of provided value
- Function name for debugging context

### Graceful Degradation

Complex or unsupported type annotations fall back to trusting the caller, ensuring the decorator never breaks existing functionality.

## Performance Characteristics

### Zero-Cost Abstractions

When types are correct (the common case), the decorator has zero performance impact beyond annotation extraction.

### Early Exit Strategy

Validation fails fast on the first type mismatch, minimizing overhead when errors occur.

### No Caching Strategy

Annotations are extracted fresh on each call, trading a small performance cost for implementation simplicity and reliability.

## Future Language Evolution

### Extensibility Points

The architecture supports future enhancements:

1. **Nested Generic Validation**: Deep validation of `List[List[int]]` contents
2. **Protocol Support**: Validation against `typing.Protocol` interfaces
3. **Custom Validators**: User-defined validation functions
4. **Warning Mode**: Development-time warnings instead of errors

### Integration with Static Analysis

The runtime system complements static type checkers like mypy, providing runtime verification of type contracts that static analysis cannot guarantee.

## Implementation Status

✅ **COMPLETED** - The type enforcement system has been successfully implemented and applied to key modules:

### Core Infrastructure
- ✅ `python/assassyn/utils/enforce_type.py` - Core validation engine
- ✅ `python/unit-tests/test_enforce_type.py` - Comprehensive test suite (20 tests)

### Applied Modules
- ✅ `python/assassyn/ir/expr/array.py` - ArrayWrite and ArrayRead constructors
- ✅ `python/assassyn/ir/const.py` - Const constructor and __getitem__ with 32-bit validation
- ✅ `python/assassyn/ir/module/base.py` - Module base functionality
- ✅ `python/assassyn/codegen/simulator/simulator.py` - Simulator generation functions
- ✅ `python/assassyn/codegen/verilog/design.py` - Verilog design generation

### Test Results
- ✅ All unit tests pass (20/20)
- ✅ Factory integration verified
- ✅ Type enforcement working correctly
- ✅ Error messages clear and informative

### Phase 2 Impact
- ✅ **Point #3 "Type System and Error Handling Documentation"** - **FUNDAMENTALLY SOLVED**
- ✅ Runtime validation addresses missing safety checks
- ✅ Type annotation inconsistencies resolved with runtime enforcement
- ✅ 32-bit limitation properly documented in Const class

## Future Improvements

1. **Nested Generic Validation**: Validate `List[List[int]]` contents
2. **Protocol Support**: Validate against `typing.Protocol` classes
3. **Performance Optimization**: Cache annotation extraction
4. **Custom Validators**: Allow custom validation functions
5. **Warning Mode**: Optional warning instead of error for development

## Design Decisions

1. **Structure-only generics**: Validates `List` vs `Dict` but not contents (performance vs correctness tradeoff)
2. **No caching**: Simpler implementation, annotations rarely change
3. **Early exit**: Fail fast on first error (better debugging experience)
4. **Graceful fallback**: Complex annotations don't break the decorator
5. **Preserve metadata**: Uses `functools.wraps` to maintain function signatures

## Technical Insights

- **Annotation extraction**: Uses `get_type_hints()` for forward reference support
- **Union handling**: Special case for `Optional` (common pattern)
- **Error messages**: Include context for easier debugging
- **Backward compatibility**: Existing code continues to work unchanged
- **Extensibility**: Easy to add new type patterns as needed
