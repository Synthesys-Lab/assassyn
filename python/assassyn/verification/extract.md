# Validation Model Extraction

## Summary

`extract.py` builds a normalized validation model from existing Verilog backend
metadata.

## Exposed Interfaces

### `build_validation_model`

```python
def build_validation_model(
    sys: SysBuilder,
    module_metadata: dict[Module, ModuleMetadata],
    interactions: InteractionMatrix,
    default_fifo_depth: int,
) -> ValidationModel
```

Creates module, trigger, FIFO, RegArray, and async-call relations keyed by
source-level coverage identifiers.

## Internal Helpers

- `_add_module_transitions`: adds module fire and trigger-counter relations with
  trigger-counter widths from shared schedule equations.
- `_add_fifo_transitions`: adds FIFO queue relations from shared schedule depth
  equations.
- `_add_array_transitions`: adds RegArray read/write commit-boundary relations
  from the same generated array metadata registry used by Verilog codegen.
- `_array_write_port`: maps one generated array writer port to write-enable,
  write-index, write-data, and selected payload signals.
- `_array_read_port`: maps one generated array read port to read-index and
  read-data signals.
- `_add_async_call_transitions`: adds async-call to FIFO alignment relations.
- `_source_module_name`: converts module instances to source-level class names
  used by coverage and validation IDs.
- `_fifo_count_signal`: resolves the generated FIFO instance occupancy signal
  path for single-element and multi-element FIFOs.
