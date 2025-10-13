# Module Generation

This module generates the simulation of each Assassyn [module](../../ir/module/),
including [pipeline stage](../../ir/module/module.py) and [downstream](../../ir/module/downstream.py)
in the folder `modules/` of the generated code.

## Section 0. Summary

The module generation system translates Assassyn IR modules into Rust simulation code. Each module (both pipeline stages and downstream modules) is converted into a Rust function that can be executed by the simulator host. The system handles different module types, generates appropriate callback functions for DRAM modules, and manages cross-module communication through exposed values in the simulator context.

## Section 1. Exposed Interfaces

### `dump_modules`

```python
def dump_modules(sys: SysBuilder, modules_dir: Path) -> bool:
```

Generates individual module files in the modules/ directory for simulator code generation.

This function creates separate files for each module and a mod.rs file for declarations.
It iterates over all modules and downstreams in the system builder and generates
corresponding Rust implementation files.

**Parameters:**
- `sys`: The system builder containing all modules to be generated
- `modules_dir`: Path to the modules directory where files will be created

**Returns:**
- `bool`: Always returns True upon successful completion

**Explanation:** This function is the main entry point for module code generation. It creates the modules directory, generates a mod.rs file with imports and module declarations, and creates individual .rs files for each module. For DRAM modules, it also generates callback functions for Ramulator2 integration. The function uses the `ElaborateModule` visitor to traverse each module's IR and generate corresponding Rust code. The generated code follows the simulator execution model described in [simulator.md](../../../docs/design/internal/simulator.md), where each module function returns a boolean indicating successful execution or blocking by `wait_until` intrinsics.

## Section 2. Internal Helpers

### `ElaborateModule`

```python
class ElaborateModule(Visitor):
```

Visitor class for elaborating modules with multi-port write support.

**Explanation:** This visitor class implements the core module-to-Rust translation logic. It traverses the IR representation of modules and generates corresponding Rust code that can be executed by the simulator. The visitor handles different IR node types including expressions, blocks, and immediate values, translating them into appropriate Rust constructs.

#### `__init__`

```python
def __init__(self, sys: SysBuilder):
```

Initialize the module elaborator.

**Parameters:**
- `sys`: The system builder containing modules to elaborate

**Explanation:** Sets up the visitor with system context, initializes indentation tracking for code formatting, and prepares module context tracking for proper code generation.

#### `visit_module`

```python
def visit_module(self, node: Module) -> str:
```

Visit a module and generate its Rust implementation.

**Parameters:**
- `node`: The module to visit and generate code for

**Returns:**
- `str`: Complete Rust function implementation for the module

**Explanation:** Generates a Rust function with signature `pub fn <module_name>(sim: &mut Simulator) -> bool`. The function traverses the module body using the visitor pattern and returns true to indicate successful execution. This function is called by `dump_modules` for each module in the system. The generated function follows the simulator execution model where modules return `true` for successful execution or `false` when blocked by `wait_until` intrinsics.

#### `visit_expr`

```python
def visit_expr(self, node: Expr) -> str:
```

Visit an expression and generate its Rust implementation.

**Parameters:**
- `node`: The expression to visit and generate code for

**Returns:**
- `str`: Rust code for the expression with proper indentation

**Explanation:** Delegates expression code generation to the [_expr](./_expr/) module using `codegen_expr`. If the expression is valued and externally used by other modules, it generates code to expose the value in the simulator context as `sim.<expr>_value = Some(value)`. This enables cross-module communication in the generated simulator. The function also adds location comments (`// @<location>`) when available to aid in debugging and tracing generated code back to the original source.

#### `visit_int_imm`

```python
def visit_int_imm(self, int_imm) -> str:
```

Visit an integer immediate value and generate its Rust representation.

**Parameters:**
- `int_imm`: The integer immediate value to convert

**Returns:**
- `str`: Rust code that casts the value to the appropriate type

**Explanation:** Converts Python integer values to Rust with proper type casting using `ValueCastTo`. This handles the conversion from Python's arbitrary precision integers to Rust's typed integer system. The function uses `dump_rval_ref` to determine the appropriate Rust type for the immediate value.

#### `visit_block`

```python
def visit_block(self, node: Block) -> str:
```

Visit a block and generate its Rust implementation.

**Parameters:**
- `node`: The block to visit (can be Block, CondBlock, or CycledBlock)

**Returns:**
- `str`: Rust code for the block with proper control flow

**Explanation:** Handles different block types:
- **CondBlock**: Generates `if <condition> { ... }` statements
- **CycledBlock**: Generates `if sim.stamp / 100 == <cycle> { ... }` for time-based execution
- **Regular Block**: Processes all elements sequentially

The function maintains proper indentation and handles nested blocks correctly. It uses a visited set to avoid processing duplicate elements in the block. The function also handles `RecordValue` nodes by visiting their underlying value expressions.

## Generated Code Structure

### DRAM Module Callbacks

DRAM modules are special because Ramulator2 uses a callback-based response interface.
For each DRAM module, a callback function is generated:

```rust
pub extern "C" fn callback_of_<dram_name>(
    req: *mut Request, ctx: *mut c_void) {
    // Handle read/write responses based on req.type_id
}
```

**Explanation:**
- `req.type_id == 0`: Read response - sets `read_succ = true` and populates data
- `req.type_id == 1`: Write response - sets `write_succ = true`
- The callback updates the corresponding `sim.<dram>_response` fields
- Refer to [ramulator2.md](../../../../tools/rust-sim-runtime/src/ramulator2.md) for Request details

This callback function is dumped in the same file as the DRAM module to minimize its visibility.

### Module Function Structure

Each generated module function follows this pattern:

```rust
pub fn <module_name>(sim: &mut Simulator) -> bool {
    // Generated module body
    true
}
```

**Explanation:** The function returns `true` for successful execution or `false` when blocked by `wait_until` intrinsics. This return value is used by the simulator host to determine whether to pop events from the module's event queue.

### Expression Exposure

When expressions are used by external modules, they are exposed in the simulator context:

```rust
sim.<expr>_value = Some(value);
```

**Explanation:** This mechanism enables cross-module communication by making computed values available to other modules through the shared simulator context. The exposure is determined by the [expr_externally_used](../../analysis/external_usage.py) analysis.

### Debug Support

The module generation includes source location tracking. When available, location comments are added as `// @<location>` to aid in debugging and tracing generated code back to the original source.