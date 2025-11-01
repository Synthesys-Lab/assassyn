# Metadata Core

`core.py` houses the shared primitives required by the Verilog metadata
pipeline: the interaction kind enum, the async call ledger, the interaction
matrix, and the FIFO expression alias.

## Exposed Interfaces

### `FIFOExpr`

Type alias for `FIFOPush | FIFOPop`, shared by module-level views and the
interaction matrix when storing FIFO traffic.

### `InteractionKind`

Enumerates the roles recorded inside the matrix (`ARRAY_READ`, `ARRAY_WRITE`,
`FIFO_PUSH`, `FIFO_POP`).  Callers use these values when classifying
interactions coming from the analysis pass.

### `AsyncLedger`

Tracks async call relationships by caller and callee.  During analysis, each
`AsyncCall` expression is registered via `record`.  After `freeze()`, the ledger
returns immutable tuples grouped by module or by callee; mutating operations
raise if attempted after freezing.

### `InteractionMatrix`

Central accumulator for metadata.  `record()` distributes interactions into
per-module and per-resource buckets while the matrix is mutable.  `freeze()`
converts the buckets into tuples, instantiates the corresponding view objects
from the specialised submodules, and prevents any further mutation.

## Internal Helpers

- Lazily imports `ModuleBundle`, `ModuleInteractionView`, `ArrayInteractionView`,
  and `FIFOInteractionView` to avoid circular dependencies with the module,
  array, and FIFO submodules.
- Guards all mutation through `_ensure_mutable()` to guarantee consumers only
  observe frozen data.
