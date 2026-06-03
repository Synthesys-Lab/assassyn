# Verilog Schedule Helpers

## Summary

`schedule.py` centralizes schedule equations shared by Verilog lowering,
cleanup backpressure generation, top-harness construction, and
translation-validation extraction.

## Exposed Interfaces

### `compute_fifo_depths`

```python
def compute_fifo_depths(
    sys: SysBuilder,
    module_metadata: dict[Module, ModuleMetadata],
    default_fifo_depth: int,
) -> dict[Module, dict[Port, int]]
```

Computes FIFO depth log2 values from explicit push metadata and defaults.

### `compute_trigger_widths`

```python
def compute_trigger_widths(
    sys: SysBuilder,
    fifo_depths: dict[Module, dict[Port, int]],
    default_fifo_depth: int,
) -> dict[Module, int]
```

Computes trigger-counter widths from the FIFO depth equations.

### `group_fifo_pushes`

```python
def group_fifo_pushes(pushes) -> dict[tuple[Module, Port], tuple[FIFOPush, ...]]
```

Groups pushes by target FIFO.

### `group_async_triggers`

```python
def group_async_triggers(
    async_ledger: AsyncLedger,
    module: Module,
) -> dict[Module, tuple[AsyncCall, ...]]
```

Returns async calls issued by one module, grouped by callee.

## Internal Helpers

This module has no private helpers.
