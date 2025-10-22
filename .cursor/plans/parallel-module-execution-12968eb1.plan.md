<!-- 12968eb1-60be-4320-9fca-111edc74665b 86607a8f-af10-47b4-b259-9a8cd7f8540b -->
# Parallel Module Execution with Minimal Lock Contention

## Overview

Parallelize simulator module execution by:

1. Wrapping shared state (Arrays, FIFOs, event queues) with `Arc<Mutex<_>>` or `Arc<RwLock<_>>`
2. Using thread pools (via `rayon` or `std::thread`) to execute modules in parallel
3. Acquiring locks once per module to commit batched operations
4. Synchronizing after module iteration before calling `tick_registers()`

## Key Changes

### 1. Thread-Safe Data Structures (`tools/rust-sim-runtime/src/runtime/xeq.rs`)

**Current Pattern:**

- `Array<T>`, `FIFO<T>`, `XEQ<T>` are not thread-safe
- Direct field access: `sim.array_name.write()`, `sim.fifo.push.push()`

**Parallel Pattern:**

- Wrap in `Arc<Mutex<_>>` or `Arc<RwLock<_>>`
- Collections: Use `Mutex` since concurrent reads don't help (modules commit writes)
- Event queues: Use `Mutex<VecDeque<usize>>`

**Strategy:**

- Keep API similar, but add thread-safe variants
- `Array<T>` → stays as-is for single-threaded path
- Add `ThreadSafeArray<T>` wrapper with `Arc<Mutex<Array<T>>>`
- Same for `FIFO`, event queues

### 2. Simulator Struct Changes (`python/assassyn/codegen/simulator/simulator.py`)

**Current:**

```rust
pub struct Simulator {
    pub stamp: usize,
    pub array_name: Array<u32>,
    pub fifo_name: FIFO<u32>,
    pub Module_event: VecDeque<usize>,
    ...
}
```

**Parallel (add config flag `parallel: bool`):**

```rust
pub struct Simulator {
    pub stamp: usize,  // Read-only during module execution, no sync needed
    pub array_name: Arc<Mutex<Array<u32>>>,
    pub fifo_name: Arc<Mutex<FIFO<u32>>>,
    pub Module_event: Arc<Mutex<VecDeque<usize>>>,
    pub Module_triggered: Arc<AtomicBool>,  // Atomic for lock-free updates
    ...
}
```

### 3. Module Execution Changes

**Current (sequential):**

```rust
for simulate in simulators.iter() {
    simulate(&mut sim);
}
```

**Parallel (using rayon or scoped threads):**

```rust
use std::thread;

thread::scope(|s| {
    for simulate_fn in simulators.iter() {
        let sim_ref = &sim;  // All threads share Simulator via Arc fields
        s.spawn(move || {
            simulate_fn(sim_ref);
        });
    }
    // Implicit join at scope end - synchronization barrier
});
```

### 4. Generated Module Functions

**Current:**

```rust
fn simulate_Module(&mut self) {
    if self.event_valid(&self.Module_event) {
        let succ = modules::Module::Module(self);
        if succ { self.Module_event.pop_front(); }
        self.Module_triggered = succ;
    }
}
```

**Parallel:**

```rust
fn simulate_Module(&self) {  // &self instead of &mut self
    let mut event_guard = self.Module_event.lock().unwrap();
    if Self::event_valid_locked(&event_guard, self.stamp.load(Ordering::Relaxed)) {
        drop(event_guard);  // Release lock before calling module
        
        let succ = modules::Module::Module(self);
        
        // Re-acquire to update state
        let mut event_guard = self.Module_event.lock().unwrap();
        if succ { event_guard.pop_front(); }
        drop(event_guard);
        
        self.Module_triggered.store(succ, Ordering::Relaxed);
    }
}
```

### 5. Operations Inside Modules

Modules call operations like:

- `sim.array.write(port_idx, write)` → `sim.array.lock().unwrap().write(port_idx, write)`
- `sim.fifo.push.push(...)` → `sim.fifo.lock().unwrap().push.push(...)`
- `sim.Module_event.push_back(...)` → `sim.Module_event.lock().unwrap().push_back(...)`

Each operation acquires lock, performs action, releases immediately.

### 6. Configuration

Add `parallel` flag to config:

- `config["parallel"] = True` → generate parallel simulator
- `config["parallel"] = False` → generate sequential simulator (default)

## Implementation Steps

1. **Add `rayon` dependency** to `tools/rust-sim-runtime/Cargo.toml`
2. **Update simulator.py** to detect `config.get("parallel", False)`
3. **Modify struct generation** to wrap fields in `Arc<Mutex<_>>` when parallel
4. **Update operation codegen** in `_expr/array.py`, `_expr/call.py` to add `.lock().unwrap()` calls
5. **Change module invoker signature** from `&mut Simulator` to `&Simulator`
6. **Replace sequential loop** with `thread::scope()` or `rayon::scope()`
7. **Use `AtomicBool` for triggered flags** and `AtomicUsize` for stamp
8. **Test with existing examples** to verify correctness

## Files to Modify

- `tools/rust-sim-runtime/Cargo.toml` - add rayon
- `tools/rust-sim-runtime/src/runtime/xeq.rs` - document thread-safety
- `python/assassyn/codegen/simulator/simulator.py` - main logic (~200 lines)
- `python/assassyn/codegen/simulator/_expr/array.py` - lock on array ops
- `python/assassyn/codegen/simulator/_expr/call.py` - lock on FIFO/event ops
- `python/assassyn/codegen/simulator/modules.py` - change signature

## Testing

Use existing examples like:

- `examples/async_axbplusc.py` - basic pipeline
- `examples/memory_engine/` - array writes
- Run with `config["parallel"] = True` vs `False` and compare outputs

### To-dos

- [ ] Add rayon dependency to rust-sim-runtime/Cargo.toml for parallel execution
- [ ] Modify simulator.py struct generation to wrap fields in Arc<Mutex<_>> when parallel=True
- [ ] Update _expr/array.py to generate .lock().unwrap() calls for parallel mode
- [ ] Update _expr/call.py to generate .lock().unwrap() calls for FIFO/event operations
- [ ] Change module invoker signatures from &mut Simulator to &Simulator for parallel mode
- [ ] Replace sequential iteration with thread::scope() for parallel module execution
- [ ] Use AtomicBool for triggered flags and AtomicUsize for stamp counter
- [ ] Test with existing examples comparing parallel vs sequential output