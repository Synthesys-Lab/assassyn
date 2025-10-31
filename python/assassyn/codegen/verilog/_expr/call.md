# Call Expression Generation

`call.py` provides the expression emitters for Assassyn's call-related IR
nodes. After the removal of legacy wire expressions, the module now focuses on
two responsibilities:

1. Triggering async calls, which drive the credit-based pipeline machinery.
2. Handling `Bind` nodes, which exist purely to structure call operands.

## Exposed Interfaces

### `codegen_async_call`

```python
def codegen_async_call(dumper, expr: AsyncCall) -> Optional[str]:
```

Registers an async call with metadata-driven trigger bookkeeping. The helper
does not emit Verilog immediately; instead it defers generation to the cleanup
phase, where triggers for each callee are aggregated (using the immutable
metadata populated by [`collect_fifo_metadata`](../analysis.md)) and
translated into credit updates. This mirrors the behaviour described in
[`arch.md`](../../../docs/design/arch/arch.md).

### `codegen_bind`

```python
def codegen_bind(_dumper, _expr: Bind) -> Optional[str]:
```

Bind nodes are structural placeholders produced by the frontend when supplying
arguments to call operations. They do not correspond to synthesizable logic, so
the emitter intentionally returns `None`.

## Notes

- Earlier versions also emitted wire assignment/read helpers for external
  SystemVerilog modules. Those responsibilities moved to
  `ExternalIntrinsic`-aware code paths, so `call.py` now contains only the two
  functions above.
- Async call registration now relies entirely on metadata; the pre-pass records
  every `AsyncCall` and its predicate, letting cleanup build trigger sums without
  touching dumper internals.
