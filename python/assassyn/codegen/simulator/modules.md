# Module Generation

This module generates the simulation of each Assassyn [module](../../ir/module/),
including [pipeline stage](../../ir/module/module.py) and [downstream](../../ir/module/downstream.py)
in the folder `modules/` of the generated code.

## Design Documents

- [Simulator Design](../../../docs/design/internal/simulator.md) - Simulator design and code generation
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system
- [Module Design](../../../docs/design/internal/module.md) - Module design and lifecycle
- [Architecture Overview](../../../docs/design/arch/arch.md) - Overall system architecture

## Related Modules

- [Simulator Generation](./simulator.md) - Core simulator generation logic
- [Simulator Elaboration](./elaborate.md) - Main entry point for simulator generation
- [Node Dumper](./node_dumper.md) - IR node reference generation
- [Port Mapper](./port_mapper.md) - Multi-port array write support

## Section 0. Summary

**DRAM Callback Implementation Details:** The module generation system implements DRAM callbacks with specific characteristics:

1. **Hardcoded Data Extraction**: DRAM callbacks use hardcoded data extraction from the response buffer
2. **Per-Module Callbacks**: Each DRAM module gets its own callback implementation
3. **Response Buffer Integration**: Callbacks are integrated with the response buffer system
4. **Memory Interface Coordination**: Callbacks coordinate with the memory interface for proper response handling

**Cross-Module Communication Mechanism:** The module generation system implements cross-module communication through:

1. **Expression Exposure Logic**: Complex logic for exposing expressions across module boundaries
2. **FFI Handle Threading**: External FFI handles are threaded through all modules
3. **Port Mapping**: Port mapping system for multi-port array writes
4. **State Coherence**: External wires are kept coherent with simulator state

**Module Context Management:** The module generation system manages module context through:

1. **ElaborateModule Visitor**: Visitor pattern for generating module code
2. **External FFI Specifications**: System-wide external FFI specifications
3. **Module State Tracking**: Tracking of module state and dependencies
4. **Code Generation Pipeline**: Integration with the overall code generation pipeline

## Section 1. Exposed Interfaces

### `dump_modules`

```python
def dump_modules(sys: SysBuilder, modules_dir: Path) -> bool:
```

Generates individual module files in the modules/ directory for simulator code generation.

This function prepares `modules/mod.rs` (with the imports required by generated code, including `sim_runtime` and its `libloading` re-exports alongside `VecDeque` utilities) and then iterates over every module/downstream to create `<module>.rs` implementations.

**Parameters:**
- `sys`: The system builder containing all modules to be generated
- `modules_dir`: Path to the modules directory where files will be created

**Returns:**
- `bool`: Always returns True upon successful completion

**Explanation:** This function is the main entry point for module code generation. It creates the modules directory, writes `mod.rs` with the shared `use` statements, and instantiates an `ElaborateModule` visitor. For each module it writes `<module>.rs`, dumps DRAM callbacks when necessary, and lets the visitor produce the function body. External SystemVerilog modules are emitted as Rust stubs that expose their FFI handles without generating a body, allowing the runtime to call into shared objects. The generated code follows the simulator execution model described in [simulator.md](../../../docs/design/internal/simulator.md), where each module function returns a boolean indicating successful execution or blocking by `wait_until` intrinsics.

## Section 2. Internal Helpers

### `ElaborateModule`

```python
class ElaborateModule(Visitor):
```

Visitor class for elaborating modules with multi-port write support.

**Explanation:** This visitor class implements the core module-to-Rust translation logic. It walks the IR representation of each module, emits Rust code for expressions and control-flow blocks, and cooperates with the `expr_externally_used` analysis to cache values that escape the module. External connections are now handled exclusively through `ExternalIntrinsic` nodes, so the visitor no longer needs bespoke bookkeeping for legacy wire assignments.

#### `__init__`

```python
def __init__(self, sys: SysBuilder):
```

Initialize the module elaborator.

**Parameters:**
- `sys`: The system builder containing modules to elaborate

**Explanation:** Sets up the visitor with system context and initializes indentation tracking for code formatting. Exposure tracking relies on `expr_externally_used`, so no extra precomputation of external assignments is required.

#### `visit_module`

```python
def visit_module(self, node: Module) -> str:
```

Visit a module and generate its Rust implementation.

**Parameters:**
- `node`: The module to visit and generate code for

**Returns:**
- `str`: Complete Rust function implementation for the module

**Explanation:** Generates a Rust function with signature `pub fn <module_name>(sim: &mut Simulator) -> bool`. External SystemVerilog modules that do not have a Python body are short-circuited to `visit_external_module`, producing a stub that simply returns `true` (the FFI handle drives the real behaviour). For internal modules the visitor traverses the body and returns `true` on success, mirroring the simulator execution model where `false` indicates the module was blocked by `wait_until`.

#### `visit_expr`

```python
def visit_expr(self, node: Expr) -> str:
```

Visit an expression and generate its Rust implementation.

**Parameters:**
- `node`: The expression to visit and generate code for

**Returns:**
- `str`: Rust code for the expression with proper indentation

**Explanation:** Delegates expression code generation to the [_expr](./_expr/) module using `codegen_expr`. When an expression is valued and flagged by `expr_externally_used`, the visitor emits a `let` binding and caches the value into `sim.<id>_value = Some(...)`. External inputs are now driven through `ExternalIntrinsic` intrinsics, so the visitor no longer synthesizes ad-hoc setter calls—everything flows through the intrinsic-specific code paths.

Location comments (`// @<location>`) are preserved for easier debugging. Expressions that do not need custom handling fall back to the standard `_expr` codegen.

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

#### Module Body Traversal

Module bodies are flat `list[Expr]` sequences. `visit_module()` iterates this list and feeds each element to `visit_expr()`. Predicate push/pop intrinsics are intercepted inside `visit_expr()` to emit `if { ... }` indentation in the generated Rust. Other values, such as `RecordValue`, delegate to their contained expression before code generation, so no additional structural traversal helper is required.

#### `visit_external_module`

```python
def visit_external_module(self, node: ExternalSV):
    """Emit a stub implementation for an external module."""
```

**Explanation:** Generates a minimal Rust function for external SystemVerilog modules. Because the real behaviour lives in dynamically loaded shared libraries, the stub simply marks that the module is driven externally (`// External module ...`) and returns `true` after silencing the unused `sim` parameter. This keeps the simulator buildable even when external modules have no Python-visible body.

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
- `req.type_id == 0`: Read response - sets `read_succ = true`, populates the data buffer with placeholder bytes, and records that the response is a read.
- `req.type_id == 1`: Write response - sets `write_succ = true` and marks the response as a write.
- Both paths update `sim.request_stamp_map_table`, ensuring the simulator can translate DRAM responses back to the stamp that issued the request.
- Refer to [ramulator2.md](../../../../tools/rust-sim-runtime/src/ramulator2.md) for `Request` details.

This callback function is dumped in the same file as the DRAM module to minimize its visibility while keeping linkage straightforward.

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

When expressions are used by other modules or external SV bindings, they are exposed in the simulator context:

```rust
let foo = { /* expression */ };
sim.foo_value = Some(foo.clone());
sim.external_handle.set_bar(ValueCastTo::<_>::cast(&foo));
```

**Explanation:** This mechanism enables cross-module communication by making computed values available to other modules through the shared simulator context. Exposure requirements are determined by the [expr_externally_used](../../analysis/external_usage.py) analysis, and external connections rely on the intrinsic-based pipeline rather than bespoke wire-assignment bookkeeping.

### Debug Support

The module generation includes source location tracking. When available, location comments are added as `// @<location>` to aid in debugging and tracing generated code back to the original source.
