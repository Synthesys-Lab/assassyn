# Coverage Helpers

## Summary

`coverage.py` loads and validates semantic coverage JSON emitted by generated
Rust simulators.

## Exposed Interfaces

### `load_coverage`

```python
def load_coverage(path: str | Path) -> dict[str, Any]
```

Reads a JSON artifact and returns it as a dictionary.

### `validate_coverage_schema`

```python
def validate_coverage_schema(coverage: Mapping[str, Any]) -> None
```

Checks that the artifact uses the supported schema and contains the required
top-level mappings.

### `summarize_fifo_occupancy`

```python
def summarize_fifo_occupancy(coverage: Mapping[str, Any]) -> dict[str, dict[str, int]]
```

Extracts FIFO occupancy counters from a validated artifact.

## Internal Helpers

This module has no private helpers.
