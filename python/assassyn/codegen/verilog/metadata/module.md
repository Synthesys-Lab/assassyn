# Module Metadata Helpers

`module.py` contains the module-scoped helpers that interact with the shared
metadata matrix.

## Exposed Interfaces

### `ModuleBundle`

Mutable bucket used by the analysis pass while the matrix remains unfrozen.  It
collects FIFO pushes/pops, per-port FIFO interaction lists, and per-array
read/write expressions.

### `ModuleInteractionView`

Immutable named tuple returned by `InteractionMatrix.module_view(module)`.  It
exposes:

- `pushes` / `pops` – tuples of FIFO expressions recorded for the module.
- `fifo_ports` – ordered tuple of FIFO ports touched by the module.
- `fifo_map` – mapping from FIFO port to the ordered tuple of recorded
  interactions.
- `writes` / `reads` – mappings from arrays to tuples of recorded write/read
  expressions.

### `ModuleMetadata`

Packages module-scoped metadata that is not already embedded in the matrix:

- Value exposures (`record_value`, `value_exposures`)
- FINISH intrinsics (`record_finish`, `finish_sites`)
- Async calls (`record_call`, `calls`)
- Frozen module view (`interactions`)

`freeze()` snapshots all mutable lists to tuples and ensures the matrix itself
is frozen.

## Internal Helpers

- Uses `_ensure_mutable()` to forbid mutation after freezing.
- Relies on `InteractionMatrix` from `core.py` for shared bookkeeping and
  view construction.
