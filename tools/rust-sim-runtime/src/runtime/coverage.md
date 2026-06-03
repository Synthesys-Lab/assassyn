# Coverage Runtime

## Summary

`coverage.rs` implements the optional source-level semantic coverage recorder
used by generated Rust simulators. It has no external serialization dependency;
the recorder stores deterministic maps and writes the coverage JSON directly.

## Exposed Interfaces

### `struct CoverageRecorder`

`CoverageRecorder` owns the coverage region, object metadata, counters, and
replayed FIFO occupancy. The public methods record module, wait, async-call,
FIFO, and array events. `flush` writes a versioned JSON artifact.

```rust
pub struct CoverageRecorder { ... }
```

Fields:

- `roi_start` and `roi_end`: optional inclusive cycle bounds.
- `objects`: source-level coverage object metadata keyed by stable IDs.
- `counters`: numeric counters keyed by coverage ID and counter name.
- `fifo_occupancy`: replayed source-level FIFO occupancy.

### `CoverageRecorder::new`

```rust
pub fn new(roi_start: Option<usize>, roi_end: Option<usize>) -> Self
```

Creates a recorder that counts only cycles inside the configured region.

### `CoverageRecorder::record_module`

```rust
pub fn record_module(&mut self, id: &str, module: &str, event: &str, cycle: usize)
```

Records module-level events such as `eligible`, `fire`, or `blocked_wait`.

### `CoverageRecorder::record_wait`

```rust
pub fn record_wait(&mut self, id: &str, module: &str, condition: bool, cycle: usize)
```

Records whether a `wait_until` condition was true or false in the active ROI.

### `CoverageRecorder::record_async_call`

```rust
pub fn record_async_call(&mut self, id: &str, caller: &str, callee: &str, cycle: usize)
```

Records a source-level async event enqueue.

### `CoverageRecorder::record_fifo_push`

```rust
pub fn record_fifo_push(
    &mut self,
    id: &str,
    module: &str,
    port: &str,
    cycle: usize,
    configured_depth: usize,
)
```

Records a FIFO push, updates replayed occupancy even outside the active ROI, and
when the cycle is covered updates max occupancy plus overflow against the
configured RTL depth.

### `CoverageRecorder::record_fifo_pop`

```rust
pub fn record_fifo_pop(&mut self, id: &str, module: &str, port: &str, cycle: usize)
```

Records a FIFO pop and updates replayed occupancy even outside the active ROI.
Covered cycles also update pop counters and empty-pop observations.

### `CoverageRecorder::record_array_read`

```rust
pub fn record_array_read(&mut self, id: &str, array: &str, cycle: usize)
```

Records a register-array or array read.

### `CoverageRecorder::record_array_write`

```rust
pub fn record_array_write(&mut self, id: &str, array: &str, cycle: usize)
```

Records a register-array or array write.

### `CoverageRecorder::flush`

```rust
pub fn flush(&self, path: &str, sim_threshold: usize) -> io::Result<()>
```

Writes the semantic coverage JSON to the given path.

## Internal Helpers

### `struct CoverageObject`

Stores object metadata for one coverage ID.

### `CoverageRecorder::covers`

Checks whether a cycle is inside the inclusive ROI.

### `CoverageRecorder::ensure_object`

Creates object metadata the first time an event ID is seen.

### `CoverageRecorder::bump`

Increments a numeric counter.

### `CoverageRecorder::set_value`

Sets a numeric counter to an exact value.

### `CoverageRecorder::set_max`

Updates a numeric counter only when the new value is larger.

### `CoverageRecorder::set_min`

Updates a numeric counter only when the new value is smaller.

### `CoverageRecorder::to_json`

Renders the full artifact as JSON text.

### `CoverageRecorder::covered_cycles`

Computes the number of cycles covered by the configured ROI.

### `option_json`

Formats optional numeric values for JSON.

### `escape_json`

Escapes strings used in JSON object keys and values.
