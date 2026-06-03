# Validation Model

## Summary

`model.py` defines dataclasses used to represent normalized translation
validation relations.

## Exposed Interfaces

### `struct RTLSignalMap`

Maps one semantic object to generated RTL signal names, FIFO handshake signals,
and signal widths used by generated monitors.

### `struct ModuleTransition`

Describes a module fire relation.

### `struct FIFOTransition`

Describes a FIFO queue relation and configured depth.

### `struct TriggerTransition`

Describes a trigger-counter relation, including RTL count/delta signal names and
the generated trigger-counter width.

### `struct AsyncCallTransition`

Describes an async call and the bound FIFO IDs it should align with.

### `struct ValidationModel`

Owns the dictionaries of module, FIFO, trigger, and async-call transitions.

#### `to_json_dict`

```python
def to_json_dict(self) -> dict[str, Any]
```

Returns a JSON-serializable model representation.

## Internal Helpers

### `_fifo_to_dict`

```python
def _fifo_to_dict(value: FIFOTransition) -> dict[str, Any]
```

Converts a FIFO transition and nested RTL signal map to JSON data.
