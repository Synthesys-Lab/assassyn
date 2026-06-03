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

Renders the monitor text, including `bind Top` for RegArray checks,
`bind fifo` for generated FIFO port checks, and `bind trigger_counter` for
trigger count bound checks. The emitted checks cover FIFO port knownness and
valid pop data, RegArray commit boundaries, trigger counter state, and X
checks. Each generated assertion has activation and failure counters, and each
monitor instance prints parseable
`translation_validation_assertion name=... activations=... failures=...` lines
from a SystemVerilog `final` block.

## Internal Helpers

- `_AssertionCounter` records one generated assertion counter pair.
- `_AssertionCounterBuilder` emits assertion statements, counter declarations,
  and final `$display` report lines.
- `_array_ports`: builds RegArray monitor ports.
- `_array_assertions`: emits RegArray read/write and commit-boundary assertions.
- `_array_write_assertions`: checks write-enable knownness, write-index/data
  knownness, and next-cycle payload equality.
- `_array_read_assertions`: checks read-index/read-data knownness.
- `_array_visibility_assertions`: checks same-cycle read/write aliases still
  observe the current selected payload value.
- `_render_bind`: emits the `bind Top` statement for RegArray signals.
- `_render_fifo_monitor`: emits the FIFO monitor module and binds it to every
  generated `fifo` instance.
- `_render_trigger_counter_monitor`: emits the trigger-counter monitor module
  and binds it to every generated `trigger_counter` instance.
- `_array_bind_connections`: emits bind connections for one RegArray.
- `_format_ports`: formats SystemVerilog port declarations.
- `_assert_known`: emits X/Z rejection assertions.
- `_assert_bounded`: emits unsigned bound assertions.
- `_sv_width`: renders packed vector widths.
- `_sv_literal`: renders fixed-width decimal literals.
- `_sv_identifier`: converts source IDs to SystemVerilog identifiers.
- `_sv_string`: escapes diagnostic strings.
