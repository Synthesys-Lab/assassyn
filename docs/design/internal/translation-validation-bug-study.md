# Translation Validation Bug Study

## Summary

This note records the first bug classes targeted by Assassyn translation
validation. The current implementation emits source-level coverage, normalized
validation JSON, and a bounded-simulation SystemVerilog monitor for RTL-visible
schedule state. The monitor is intentionally local: it checks trigger counters,
FIFO occupancy bounds, FIFO push/pop handshakes, and unknown values on active
signals.

## Injected Bug Classes

### Dropped Trigger Credit

Mutation: remove or zero a callee trigger delta while FIFO pushes still happen.

Expected failure: the validation JSON still maps the async call to the callee
trigger relation, and a follow-up semantic checker should detect divergence
between the source event count and `*_trigger_counter_inst.count`.

Current coverage: static model extraction records trigger count/delta paths;
monitor checks count bounds and X values.

### Missing FIFO Push

Mutation: remove one bound-port FIFO push from an async call.

Expected failure: `check_model_consistency` reports async calls that reference a
missing FIFO relation after model mutation. A future relation checker should
also compare async-call FIFO alignment.

Current coverage: `test_monitor_emission.py` mutates the model by deleting the
target FIFO and confirms the consistency checker catches it.

### Mismatched Push Predicate

Mutation: keep the trigger predicate but drop a FIFO push readiness or source
predicate.

Expected failure: the bounded monitor reports `FIFO push without ready` if the
generated push-valid signal fires under backpressure. Future JSON equations
should include predicate strings to compare trigger and FIFO push conditions.

### Wrong FIFO Depth

Mutation: lower `DEPTH_LOG2` below the depth selected by Assassyn schedule
metadata.

Expected failure: monitor reports `FIFO count overflow` when internal FIFO
occupancy exceeds the configured depth in the validation model.

### Wrong Pop Guard

Mutation: assert pop-ready without valid data.

Expected failure: monitor reports `FIFO pop without valid`.

### Same-Cycle Register Commit

Mutation: make a register/array write visible in the same cycle instead of the
commit boundary.

Expected failure: not covered by the current monitor. The JSON model already
has room for array transitions, but the first monitor slice focuses on FIFO and
trigger schedule state.

## Source-Level Debug Mapping

Coverage IDs are shared between semantic coverage and validation artifacts:

- `module:<ClassName>`;
- `fifo:<ClassName>.<port>`;
- `async:<CallerClass>-><CalleeClass>:<index>`.

The generated monitor emits these IDs in `$error` strings so a simulator failure
can be mapped back to the same source-level objects reported in `coverage.json`
and `translation_validation.json`.

## Remaining Work

- Emit predicate equations in JSON and compare trigger/FIFO predicate alignment.
- Add array/register commit-boundary monitor checks.
- Add formal miter generation after the bounded-simulation monitor stabilizes.
- Run monitor checks through Verilator once the environment has `cocotb`.
