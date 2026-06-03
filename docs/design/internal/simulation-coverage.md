# Semantic Coverage for Simulation

This document describes source-level semantic coverage for the Assassyn simulator.
The design decision is to collect coverage at the architectural event level instead
of relying on host-language line coverage or generated RTL coverage. The coverage
data is emitted by the simulator as a structured artifact that can be compared
with RTL execution and used to size buffers, inspect blocked stages, and guide
translation validation.

Traditional simulator coverage can count which Rust or Python lines executed, and
traditional RTL coverage can count which Verilog branches toggled. Neither view
directly answers whether an Assassyn program exercised a stage firing, a
`wait_until` stall, a FIFO occupancy corner, or an async-call/bind alignment. The
semantic coverage model therefore records events in terms of Assassyn modules,
ports, predicates, arrays, and source locations.

## Coverage Decision

Coverage is recorded as an optional simulator feature. The default simulator
behavior remains unchanged when semantic coverage is disabled. When enabled, each
recorded event uses a stable coverage identifier that refers to the Assassyn
semantic object rather than a generated implementation detail.

The covered objects are:

- pipeline modules and downstream modules;
- module activation events;
- `wait_until` conditions;
- FIFO pushes, pops, and occupancy;
- async calls and bind argument pushes;
- predicate true/false outcomes;
- array and register-array reads and writes;
- commit-boundary observations such as same-cycle read-after-write.

The coverage artifact is a JSON document. It is intended for automated checking,
human debugging, and later translation-validation correlation.

## Runtime Scope

Coverage collection is scoped by an explicit region of interest. A region can be
expressed as an inclusive cycle interval or by generated entry and exit markers.
Only cycles inside the active region contribute to coverage counters. If no region
is configured, the full simulator run is covered.

The region model separates three costs:

- simulator setup and compilation;
- simulator execution outside the region;
- covered execution inside the region.

Performance reports must state which region is measured. Full-program profiling
is not considered meaningful for comparing architecture behavior because setup,
initialization, and shutdown can dominate small designs.

## Coverage Identifiers

Each coverage identifier has a stable textual path:

```text
module:<module>
module:<module>:wait:<ordinal>
fifo:<module>.<port>
async:<caller>-><callee>:<ordinal>
bind:<caller>-><callee>:<port>:<ordinal>
array:<array>
predicate:<module>:<ordinal>
```

The identifier is paired with metadata:

- kind of semantic object;
- module name;
- port or array name when applicable;
- source location when available;
- printable Assassyn IR expression;
- optional parent identifier for grouping.

The identifier format is not tied to a generated Rust function name or RTL wire
name. Translation validation can therefore reuse the same identifiers while
mapping them to RTL signals separately.

## Event Counters

A module records:

- `eligible`: an event or upstream trigger was present;
- `fire`: the module executed successfully;
- `blocked_wait`: execution was stopped by `wait_until`;
- `blocked_empty_fifo`: execution attempted to consume a missing FIFO value;
- `downstream_triggered`: a downstream module was activated by an upstream fire.

A `wait_until` condition records:

- `true`;
- `false`;
- `first_false_cycle`;
- `last_false_cycle`.

A FIFO records:

- `push`;
- `pop`;
- `simultaneous_push_pop`;
- `empty_pop_attempt`;
- `max_occupancy`;
- `final_occupancy`;
- `configured_rtl_depth`;
- `overflow_under_configured_depth`.

An async call records:

- `call_fire`;
- `event_enqueue`;
- `callee_module`;
- `bound_fifo_pushes`;
- `argument_alignment_error`.

An array records:

- `read`;
- `write`;
- `same_cycle_read_after_write`;
- `multi_write_conflict`;
- `commit_count`.

## FIFO Occupancy Model

The simulator queue is semantically unbounded unless a design rule adds an
explicit finite capacity. Generated RTL FIFOs are bounded. Semantic coverage
therefore tracks both the actual simulator occupancy and a replay against the
configured RTL depth.

For a FIFO with configured depth `D`, each covered cycle applies:

```text
occupancy' = occupancy + push_fire - pop_fire
overflow_under_configured_depth = occupancy' > D
```

This is a workload-specific buffer-sizing signal. It does not prove the design is
deadlock free for all inputs, but it reports whether the observed source-level
execution exceeds the generated RTL buffering.

## JSON Schema

The top-level coverage artifact contains:

```json
{
  "schema": "assassyn.semantic_coverage.v1",
  "roi": {
    "start_cycle": 0,
    "end_cycle": 100
  },
  "run": {
    "sim_threshold": 100,
    "covered_cycles": 100
  },
  "objects": {
    "fifo:consumer.data": {
      "kind": "fifo",
      "module": "consumer",
      "port": "data",
      "loc": "example.py:42",
      "expr": "data.pop()"
    }
  },
  "counters": {
    "fifo:consumer.data": {
      "push": 10,
      "pop": 9,
      "max_occupancy": 2
    }
  }
}
```

Unknown counter keys are ignored by older readers. Required keys are validated by
schema version.

## Relationship to Verilator

Semantic coverage and Verilator coverage answer different questions. Verilator
can expose RTL signal activity and assertion failures after lowering. Semantic
coverage reports which Assassyn-level transitions were exercised before RTL
lowering.

The performance comparison must therefore include both runtime and flexibility:

- Rust simulator without coverage;
- Rust simulator with semantic coverage;
- generated RTL through Verilator;
- Verilator compile time separated from Verilator runtime;
- covered event throughput over the same region of interest;
- source-level filters that enable only selected modules, FIFOs, or predicates.

Assassyn semantic coverage is expected to be more flexible for source-level
architectural questions because instrumentation can be selected by semantic
object. Verilator remains the reference for RTL execution cost and implementation
behavior after lowering.

## Use in Translation Validation

Semantic coverage is a precursor to translation validation. It provides stable
source-level identifiers for transitions that later become normalized semantic
equations and RTL monitor checks.

For example, a FIFO coverage entry:

```text
fifo:decode.inst
```

can be reused by the validation model to relate:

```text
source queue decode.inst
RTL fifo_decode_inst count/front/back/payload
```

This allows a monitor failure to be reported as a source-level FIFO or async call
instead of only as a generated RTL signal mismatch.
