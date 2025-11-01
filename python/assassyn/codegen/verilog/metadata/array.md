# Array Metadata Helpers

`array.py` contains array-centric helpers used by the Verilog metadata pipeline.

## Exposed Interfaces

### `ArrayInteractionView`

Immutable view returned by `InteractionMatrix.array_view(array)` with three
attributes:

- `reads` – tuple of `ArrayRead` expressions recorded for the array.
- `writers` – mapping from module to the tuple of `ArrayWrite` expressions it
  produced.
- `reads_by_module` – mapping from module to the tuple of read expressions it
  issued.

### `ArrayMetadata`

Compatibility container consumed by `ArrayMetadataRegistry` in
`python/assassyn/codegen/verilog/array.py`.  It tracks write port indexes,
read-port numbering by module, overall read order, and the modules that touch a
given array.

## Internal Helpers

The module is intentionally lightweight: it defines only the data holders so
the shared matrix can instantiate the immutable views during `freeze()`, while
the registry manages numbering and user tracking elsewhere.
