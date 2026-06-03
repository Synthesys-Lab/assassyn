# Validation Artifact Emission

## Summary

`emit.py` writes translation-validation artifacts.

## Exposed Interfaces

### `write_validation_json`

```python
def write_validation_json(model: ValidationModel, path: str | Path) -> None
```

Writes the extracted model as formatted JSON.

### `write_monitor_stub`

```python
def write_monitor_stub(model: ValidationModel, path: str | Path) -> None
```

Writes the initial SystemVerilog monitor scaffold.

## Internal Helpers

This module has no private helpers.
