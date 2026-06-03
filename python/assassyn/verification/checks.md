# Validation Checks

## Summary

`checks.py` provides static consistency checks over extracted validation models.

## Exposed Interfaces

### `check_model_consistency`

```python
def check_model_consistency(model: ValidationModel) -> list[str]
```

Returns a list of consistency errors, or an empty list when all referenced
modules and FIFOs exist.

## Internal Helpers

This module has no private helpers.
