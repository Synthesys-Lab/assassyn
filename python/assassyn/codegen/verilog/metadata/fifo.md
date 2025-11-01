# FIFO Metadata Helpers

`fifo.py` defines the FIFO-centric view that complements the module and array
projections.

## Exposed Interfaces

### `FIFOInteractionView`

Immutable named tuple with two attributes:

- `pushes` – tuple of `FIFOPush` expressions recorded for the FIFO port.
- `pops` – tuple of `FIFOPop` expressions recorded for the FIFO port.

The view is constructed by `InteractionMatrix.freeze()` and exposed via
`InteractionMatrix.fifo_view(port)`.  Consumers rely on the tuple order to
match the encounter order preserved during analysis.
