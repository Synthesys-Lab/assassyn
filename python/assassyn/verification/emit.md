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

Writes the bounded-simulation SystemVerilog monitor.

### `render_monitor`

```python
def render_monitor(model: ValidationModel) -> str
```

Renders the monitor text, including `bind Top`, trigger count bound checks, FIFO
count bound checks, FIFO push/pop handshake checks, RegArray commit-boundary
checks, and X checks.

## Internal Helpers

- `_trigger_ports`: builds trigger-counter monitor ports.
- `_fifo_ports`: builds FIFO monitor ports.
- `_array_ports`: builds RegArray monitor ports.
- `_trigger_assertions`: emits trigger-counter assertions.
- `_fifo_assertions`: emits FIFO safety assertions.
- `_array_assertions`: emits RegArray read/write and commit-boundary assertions.
- `_array_write_assertions`: checks write-enable knownness, write-index/data
  knownness, and next-cycle payload equality.
- `_array_read_assertions`: checks read-index/read-data knownness.
- `_array_visibility_assertions`: checks same-cycle read/write aliases still
  observe the current selected payload value.
- `_render_bind`: emits the `bind Top` statement.
- `_fifo_bind_connections`: emits bind connections for one FIFO.
- `_array_bind_connections`: emits bind connections for one RegArray.
- `_format_ports`: formats SystemVerilog port declarations.
- `_assert_known`: emits X/Z rejection assertions.
- `_assert_bounded`: emits unsigned bound assertions.
- `_sv_width`: renders packed vector widths.
- `_sv_literal`: renders fixed-width decimal literals.
- `_sv_identifier`: converts source IDs to SystemVerilog identifiers.
- `_sv_string`: escapes diagnostic strings.
