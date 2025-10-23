# Naming System Design

The Assassyn naming system provides semantic naming for IR values and modules, enabling readable code generation and debugging. The system combines AST rewriting, type-oriented naming, and hierarchical context management to generate meaningful identifiers that reflect the semantic structure of hardware designs.

## Overview

The naming system consists of four core components that work together to provide comprehensive semantic naming:

1. **NamingManager**: Central coordinator that integrates all naming components
2. **TypeOrientedNamer**: Generates semantic names based on IR node types and operations  
3. **UniqueNameCache**: Ensures uniqueness of generated names through counter-based suffixes
4. **AST Rewriting**: Intercepts Python assignments to enable semantic naming

## Architecture

### Component Hierarchy

```
NamingManager (Central Coordinator)
├── TypeOrientedNamer (Type-based naming)
│   └── UniqueNameCache (Uniqueness guarantee)
├── UniqueNameCache (Module naming)
└── AST Rewriting System
    └── AssignmentRewriter (AST transformation)
```

### Data Flow

1. **IR Creation**: `ir_builder` decorator creates IR expressions → `NamingManager.push_value()` → `TypeOrientedNamer.name_value()` → `UniqueNameCache.get_unique_name()`

2. **Assignment Processing**: `@rewrite_assign` decorator transforms assignments → `__assassyn_assignment__()` → `NamingManager.process_assignment()` → `TypeOrientedNamer.name_value()`

3. **Module Naming**: Module creation → `NamingManager.assign_name()` → `TypeOrientedNamer.name_value()` → `UniqueNameCache.get_unique_name()`

## Core Components

### NamingManager

The `NamingManager` serves as the central coordinator for the entire naming system. It integrates type-based naming with AST rewriting hooks and provides semantic naming for IR values and modules.

**Key Responsibilities:**
- Coordinate between `TypeOrientedNamer` and AST rewriting system
- Manage hierarchical naming contexts (module prefixes)
- Provide public interface for naming non-expression objects
- Handle global naming manager state

**Integration Points:**
- Called by `ir_builder` decorator when new IR expressions are created
- Called by AST rewriting system through `__assassyn_assignment__` function
- Used by modules and arrays for semantic naming
- Manages global naming manager state through singleton pattern

### TypeOrientedNamer

The `TypeOrientedNamer` implements sophisticated naming strategies based on IR node types, operations, and operands. It uses hardcoded opcode mappings and class-based prefixes to generate descriptive identifiers.

**Naming Strategy Hierarchy:**
1. **Module instances**: `ModuleBase` → `{ClassName}Instance`
2. **Pure intrinsics**: Uses opcode and operand names (e.g., `fifo_peek`, `port_valid`)
3. **Class-based mappings**: Direct prefixes for specific IR classes (`ArrayRead` → `rd`, `FIFOPop` → `pop`)
4. **Binary operations**: Combines operand descriptions with operation tokens (`lhs_add_rhs`)
5. **Unary operations**: Combines operation token with operand description (`neg_operand`)
6. **Special operations**: Handles `Cast`, `Slice`, `Concat`, `Select` with descriptive prefixes
7. **Fallback**: Uses `name` attribute or defaults to `"val"`

**Operand Analysis:**
- Extracts meaningful names from operands using the unified `name` attribute
- Unwraps operand wrappers when available
- Combines multiple operand descriptions into concise identifiers
- Limits identifier length to 25 characters for readability

### UniqueNameCache

The `UniqueNameCache` provides a lightweight counter-based mechanism for ensuring unique identifiers with common prefixes. It's used by both `TypeOrientedNamer` and `NamingManager` to avoid naming conflicts.

**Uniqueness Strategy:**
- First use of prefix: Returns prefix unchanged (`foo`)
- Subsequent uses: Appends numeric suffix (`foo_1`, `foo_2`, etc.)
- Per-instance counters: Each cache maintains independent counters
- Simple but effective: Avoids complex collision resolution

### AST Rewriting System

The AST rewriting system enables semantic naming by transforming Python assignment statements at the AST level. Since Python assignment cannot be overloaded, this system intercepts assignments and delegates them to the naming system.

**Transformation Process:**
1. Parse function source code using `inspect.getsource()` and `ast.parse()`
2. Apply `AssignmentRewriter` transformer to convert simple identifier assignments
3. Inject `__assassyn_assignment__` function into namespace
4. Compile transformed AST and preserve function metadata
5. Graceful fallback to original function if transformation fails

**Assignment Rewriting:**
- `x = some_expr` → `x = __assassyn_assignment__("x", some_expr)`
- Only rewrites simple identifier assignments (not attributes or subscripts)
- Preserves original assignment semantics
- Supports chained assignments

## Unified Name Attribute System

The naming system uses a unified `name` attribute on the `Value` base class to store semantic names on IR objects:

**Purpose**: Provides a standardized way to attach human-readable names to IR values with a consistent interface across all IR nodes

**Lifecycle**: Names are assigned when objects are created or when assignments are processed

**Usage**: The attribute is checked by `TypeOrientedNamer._entity_name()` to extract meaningful names for generating descriptive identifiers

**Fallback**: If the name is not available, the system falls back to type-based naming or generates default identifiers

## Context-Aware Naming

The system provides hierarchical naming contexts to reflect the structure of hardware designs:

**Module Context**: When inside a module's `build()` method, entities inherit their parent module's name as a prefix

**Array Naming**: Arrays created without explicit names receive hierarchical hints like `<module_name>_array`

**Context Prefix**: `NamingManager.get_context_prefix()` returns the current module name for use as a naming hint

## Integration with Builder System

The naming system integrates deeply with the Assassyn builder system:

**Global State Management**: Uses singleton pattern through `Singleton.builder` to access current module context

**IR Builder Integration**: `ir_builder` decorator automatically names new IR expressions

**Module Lifecycle**: Modules receive semantic names during creation and can provide context for child entities

**External Dependencies**: Tracks external operands used by modules for proper naming

## Design Principles

### Low Invasion
The naming system is designed to be minimally invasive to existing code:
- Uses unified `name` attribute to avoid conflicts with existing code
- Graceful fallbacks when naming fails
- Optional decorators that can be applied selectively

### Semantic Clarity
Generated names reflect the semantic meaning of operations:
- Type-based prefixes for different IR constructs
- Operand descriptions for complex operations
- Hierarchical naming for structural relationships

### Uniqueness Guarantee
The system ensures all generated names are unique:
- Counter-based suffixes for duplicate prefixes
- Per-instance caches to avoid global conflicts
- Deterministic naming for reproducible builds

### Rust Compatibility
Names are designed to be compatible with Rust naming conventions:
- PascalCase for module instances (`AdderInstance`)
- snake_case for operations and variables
- Avoids Rust reserved keywords and naming conflicts

## Error Handling and Robustness

The naming system is designed to be robust and fail gracefully:

**Import Safety**: Uses try-catch blocks around imports to handle missing dependencies

**Attribute Safety**: Uses direct attribute access since all `Value` subclasses now have a unified `name` attribute

**Transformation Safety**: AST rewriting falls back to original function if transformation fails

**Type Safety**: Handles cases where objects cannot be annotated (Python builtins)

## Performance Considerations

The naming system is optimized for performance:

**Caching**: Uses `UniqueNameCache` to avoid repeated name generation

**Lazy Evaluation**: Names are generated only when needed

**Minimal Overhead**: AST rewriting is applied only to decorated functions

**Efficient Lookups**: Uses dictionary-based opcode and class mappings

## Future Extensions

The naming system is designed to be extensible:

**Custom Naming Strategies**: New naming strategies can be added to `TypeOrientedNamer`

**Additional Context Types**: New context types can be added to `NamingManager`

**Enhanced AST Rewriting**: More sophisticated AST transformations can be added

**Naming Policies**: Configurable naming policies can be implemented for different target languages
