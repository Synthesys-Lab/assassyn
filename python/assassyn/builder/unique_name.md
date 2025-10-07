# Unique Name Cache

Lightweight counter-based helper for generating unique identifiers that share a
common prefix.

## Exposed Class

```python
class UniqueNameCache:
    def __init__(self):
        ...
    def get_unique_name(self, prefix: str) -> str:
        ...
```

- `__init__` sets up an empty dictionary mapping each prefix to the last number
  that was assigned.
- `get_unique_name` returns the bare prefix the first time it is requested, then
  increments and appends a numeric suffix on subsequent calls (`foo`, `foo_1`,
  `foo_2`, ...).
