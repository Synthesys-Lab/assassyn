# Verilog Cleanup and Signal Generation

This module provides post-generation cleanup utilities for Verilog code generation, handling signal generation for module interconnections, memory interfaces, port management, and the extra bookkeeping required to surface values for external SystemVerilog FFIs.

## Summary

The cleanup module is responsible for generating the final control signals and interconnections after the main Verilog code generation is complete. It handles complex signal routing for arrays, ports, modules, and memory interfaces, ensuring proper connectivity between generated modules according to the credit-based pipeline architecture. As part of the external module flow, it also materialises producer-side `expose_*`/`valid_*` ports for any external register outputs that are consumed by another module, so cross-module reads can be wired up without duplicating logic.  FIFO wiring now consumes the metadata snapshot produced by the pre-pass instead of mutating registries during emission.

Metadata consumed here (`ModuleMetadata`, `ModuleInteractionView`, `ArrayMetadata`,
and `FIFOInteractionView`) is provided by the `python.assassyn.codegen.verilog.metadata`
package.  Implementations live in the `metadata.module`, `metadata.array`, and
`metadata.fifo` submodules but remain accessible via the `metadata` namespace for callers.

## Exposed Interfaces

### `cleanup_post_generation`

```python
def cleanup_post_generation(dumper):
    """generating signals for connecting modules"""
```

**Explanation**

This is the main cleanup function that generates all the necessary control signals and interconnections after the primary Verilog code generation is complete. It performs the following steps:

1. **Execution Signal Generation**: Creates the `executed_wire` signal that determines when a module should execute:
   - For downstream modules: Gathers upstream dependencies with `analysis.get_upstreams(module)` and ORs their `executed` flags via `_format_reduction_expr(..., op="operator.or_", default_literal="Bits(1)(0)")`.
   - For regular modules: ANDs the trigger-counter pop-valid input with any active `wait_until` predicate recorded during expression lowering using the same helper with `op="operator.and_"` and a `Bits(1)(1)` default.

2. **Finish Signal Generation**: Reduces every FINISH site captured in
   `module_metadata.finish_sites`, formatting each intrinsic’s `expr.meta_cond` and gating it with
   `executed_wire` before OR-reducing the terms into `self.finish`.

3. **SRAM Control Signal Generation**: When the current module wraps an SRAM payload (detected via `array.is_payload(sram_instance)`), `generate_sram_control_signals` derives write enables, addresses, and data from the exposed array accesses, producing the handshakes expected by the memory blackbox.

4. **Array Write Signal Generation**: For each array surfaced by
   `module_metadata.interactions.writes`:
   - Filters out arrays whose owner is a memory instance and satisfy `array.is_payload(owner)`, because those are handled by dedicated memory logic.
   - Uses the module view’s `writes(array)` tuples (which mirror the global array view maintained by the `InteractionMatrix`) to map interactions onto the precomputed port indices stored in the `ArrayMetadataRegistry`.
   - Emits write-enable, write-data, and write-index signals per port, formatting each write’s `expr.meta_cond` with `dumper.format_predicate`. Multi-writer modules rely on `_emit_predicate_mux_chain` to collapse predicates and thread prioritised mux chains for data and indices, guaranteeing consistent selection semantics.

5. **FIFO Signal Generation**: Walks `module_metadata.interactions.fifo_ports` to visit each FIFO touched by the module:
   - Pulls the per-port `FIFOInteractionView` directly from the shared matrix so the recorded `FIFOPush` / `FIFOPop` expressions stay in sync across consumers—predicates come from each expression’s `meta_cond`, push data from `expr.val`, and module ownership from the metadata view that registered the expression.
   - Applies backpressure via the parent module's `fifo_*_push_ready` signals and emits valid/data assignments driven purely from metadata captured during the pre-pass.
   - Produces the module-local `*_pop_ready` backpressure signal without consulting dumper internals.
   - Reuses `_emit_predicate_mux_chain` so the push-valid reduction and push-data mux mirror the prioritisation used for array writes.

6. **Module Trigger Signal Generation**: Reads async trigger exposures from `dumper.interactions.async_ledger.calls_for_module(current_module)`, sums all predicates (each taken from the call’s `meta_cond` and converted to an 8-bit increment), and routes the result into `<callee>_trigger`.

7. **External Exposure Generation**: For every value exposure in `module_metadata.value_exposures`:
   - Schedules `expose_<name>`/`valid_<name>` port declarations for the module generator.
   - Emits assignments that drive the value and its validity, converting each expression’s `meta_cond` into bit expressions through `dumper.format_predicate`.
   - Skips raw objects that are bridged through dedicated external wiring handled elsewhere.
   - Emits additional `expose_<instance>_<port>` / `valid_<instance>_<port>` pairs for every external register output that is consumed by another module, using the cross-module metadata recorded earlier in the pipeline.

8. **Bookkeeping**: Records `self.executed = executed_wire` as the last assignment, ensuring downstream consumers and the top-level harness can observe the execution result.

**Project-specific Knowledge Required**:
- Understanding of [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) class structure
- Knowledge of [credit-based pipeline architecture](/docs/design/arch/arch.md)
- Understanding of [array write port mapping](/python/assassyn/codegen/simulator/port_mapper.md)
- Reference to [SRAM memory interface](/python/assassyn/ir/memory/sram.md)

### `generate_sram_control_signals`

```python
def generate_sram_control_signals(dumper, sram_info):
    """Generate control signals for SRAM memory interface."""
```

**Explanation**

This function generates the control signals specifically for SRAM memory interfaces. It performs the following steps:

1. **Write Signal Analysis**: Identifies all array write operations and extracts write address, data, and enable conditions
2. **Read Signal Analysis**: Identifies array read operations and extracts read addresses
3. **Address Selection**: Creates multiplexer logic to select between write and read addresses, prioritizing write addresses when writing
4. **Control Signal Generation**: Generates the following signals:
   - `mem_write_enable`: Write enable signal based on execution and write predicates
   - `mem_address`: Multiplexed address signal
   - `mem_write_data`: Write data signal
   - `mem_read_enable`: Always-enabled read signal

**Project-specific Knowledge Required**:
- Understanding of [SRAM memory model](/python/assassyn/ir/memory/sram.md)
- Knowledge of [array read/write operations](/python/assassyn/ir/expr/array.md)

### `_emit_predicate_mux_chain`

```python
def _emit_predicate_mux_chain(entries, *, render_predicate, render_value, default_value, aggregate_predicates):
    """Return both the mux chain and aggregate predicate for *entries*."""
```

**Explanation**

This helper consolidates the predicate-driven mux logic shared by array writes and FIFO pushes. Callers provide renderers for predicates and values alongside a default expression and reduction strategy; the helper then:

1. Collects predicate literals via `render_predicate` and feeds them into `aggregate_predicates`, allowing array writers to omit a default literal while FIFO pushes supply `Bits(1)(0)`.
2. Threads a nested `Mux` chain seeded with `default_value`, preserving iteration order so later entries win, matching the legacy manual loops. A single-entry list simply returns that entry’s value (no redundant `Mux` is introduced), while an empty list yields the caller-supplied default value.
3. Returns a `(mux_expr, aggregated_predicate)` tuple so enable reductions, data muxes, and index muxes can reuse the same predicate formatting without duplication. When no entries exist, callers receive the reduction produced by `aggregate_predicates([])` (for example, `Bits(1)(0)` in the FIFO case), keeping zero-writer scenarios explicit and consistent across call sites.

**Project-specific Knowledge Required**:
- Understanding of [array write operations](/python/assassyn/ir/expr/array.md)
- Knowledge of [FIFO metadata collection](/python/assassyn/codegen/verilog/analysis.md)
- Familiarity with [type casting utilities](/python/assassyn/codegen/verilog/utils.md)

## Internal Helpers

The module uses several internal helper functions and imports utilities from other modules:

- `dump_type()` and `dump_type_cast()` from [utils](/python/assassyn/codegen/verilog/utils.md) for type handling
- `get_sram_info()` from [utils](/python/assassyn/codegen/verilog/utils.md) for SRAM information extraction
- `namify()` and `unwrap_operand()` from [utils](/python/assassyn/utils.md) for name generation and operand handling
- `_format_reduction_expr(predicates, *, default_literal, op="operator.or_")` canonicalises OR/AND-style predicate reductions, emitting caller-provided defaults for empty sequences while allowing any reducer supported by the dumper runtime. Callers pass `operator.and_` when AND semantics are required, keeping generated code consistent with the `operator` module import in the Verilog header.
- `_emit_predicate_mux_chain()` centralises predicate-driven mux construction so callers reuse ordering and reduction semantics.

The cleanup process is tightly integrated with the [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) class and is called as the final step in module generation to ensure all interconnections are properly established.
