# Translation Validation

This document describes translation validation for Assassyn generated RTL. The
design decision is to validate each generated design against a normalized
Assassyn cycle semantics rather than attempting to prove the entire compiler
correct once for all possible programs.

Assassyn already uses one high-level description for simulation and RTL
generation. Translation validation makes this unification checkable by building a
state relation between the normalized Assassyn state and the generated RTL state.
The first validation target is scheduling and architectural state preservation:
module firing, async-call event counts, FIFO data order, predicate gating, and
register-array commit boundaries.

## Validation Decision

The validated source model is a normalized cycle model. It is lower than the
surface DSL but higher than generated RTL. It contains only the semantic objects
needed for pipeline execution:

- module event counts;
- module fire predicates;
- FIFO queues for module ports;
- async-call deltas;
- `wait_until` and condition predicates;
- array payloads and pending writes;
- source-level identifiers and source locations.

The normalized model is extracted from the same analysis facts that drive RTL
generation. The checker then compares this model with generated RTL state and
signals. This is translation validation for a concrete generated design, not a
global compiler-correctness proof.

## Normalized Cycle Semantics

At cycle `t`, a normalized state is:

```text
S_t = {
  event_count[module],
  fifo_queue[module.port],
  array_payload[array],
  pending_array_writes[array],
  predicate_values,
  observed_outputs
}
```

Each module has a `fire[module]` value:

```text
fire[module] =
  event_count[module] > 0
  and wait_predicates[module]
  and required_fifo_values_valid[module]
  and required_backpressure_ready[module]
```

When `fire[module]` is false, the module does not consume its event token and its
side effects are not committed. When it is true:

- one event token is consumed;
- each active FIFO pop consumes the head value of the corresponding source queue;
- each active FIFO push appends data to the target source queue at the cycle
  boundary;
- each active async call increments the callee event count at the cycle boundary;
- active array writes are placed in the pending-write set;
- pending array writes commit at the normal register boundary.

The Rust simulator uses half-cycle bookkeeping for event and FIFO scheduling.
The normalized model folds those implementation timestamps into a single
architectural clock transition.

## RTL State Relation

The RTL state is related to the normalized state by object-specific projections:

```text
event_count[module] == TriggerCounter(module).count
fifo_queue[module.port] == FIFO(module.port).abstract_sequence
array_payload[array] == RegisterFile(array).payload
pending_array_writes[array] == RegisterFile(array).write_ports
fire[module] == generated executed signal for module
```

The FIFO abstract sequence is the ordered payload visible from the current front
pointer for the current count. A single-element FIFO is treated as an abstract
sequence of length zero or one.

The relation intentionally refers to semantic object names. Generated RTL signal
names are recorded in a separate signal map so the source-level model does not
depend on one textual lowering.

## One-Step Obligation

For each generated design, validation checks the following one-step condition:

```text
Rel(S_t, R_t) and SameInputs(S_t, R_t)
  implies
Rel(step_assassyn(S_t), step_rtl(R_t))
  and SameObservations(step_assassyn(S_t), step_rtl(R_t))
```

The first implementation can check this condition through bounded simulation
monitors. Later implementations can reuse the same extracted model for induction,
SMT, or CIRCT Verif lowering.

## Checked Properties

The first checker covers scheduling and state preservation:

- event delta equals the number of firing async calls targeting the module minus
  the consumed event token;
- FIFO push predicates and async trigger predicates are aligned for each bound
  argument group;
- FIFO push data equals the source expression selected by the normalized
  predicate priority;
- FIFO pop readiness equals `fire[module]` and the normalized pop predicate;
- FIFO data order is preserved by the RTL queue;
- array write-enable, write-index, and write-data match the normalized pending
  write set;
- array writes are visible only after the commit boundary.

These checks are stronger than local safety assertions such as "FIFO does not
underflow." A local assertion can prove that a bad state is absent; translation
validation checks that the RTL transition implements the source transition.

## Coverage Reuse

Semantic coverage identifiers are reused as validation identifiers. A coverage
event such as:

```text
async:execute->writeback:0
fifo:writeback.result
```

is also the key used in the validation JSON and monitor output. This gives the
same source-level names to simulator coverage reports and RTL mismatch reports.

The reuse avoids a second naming scheme and allows a counterexample to be
reported at the Assassyn object that caused the mismatch.

## Validation Artifacts

The validation flow emits two artifacts when enabled:

```text
translation_validation.json
translation_validation_monitor.sv
```

The JSON artifact records:

- schema version;
- module transitions;
- FIFO transitions;
- trigger-counter transitions;
- array transitions;
- predicate terms;
- source locations;
- generated RTL signal map.

The SystemVerilog monitor is a bounded-simulation checker. It can be included in
the generated Verilator testbench when validation is enabled. The normal RTL
interface remains unchanged when validation is disabled.

## Exclusions

The first validation target excludes:

- DRAM and Ramulator timing;
- SRAM blackbox internals;
- external SystemVerilog implementation internals;
- ISA-level CPU correctness;
- RVFI generation;
- deadlock and liveness proofs;
- automatic FIFO-depth synthesis.

Those features can build on the same normalized model after scheduling and state
preservation are stable.

## Comparison With Auto Assertions

Automatically generated assertions remain useful for local safety. However, they
are not the primary contribution. The primary contribution is a state relation
and transition check:

```text
Assassyn semantic state
  corresponds to
generated RTL concrete state
```

and:

```text
one normalized Assassyn cycle
  corresponds to
one generated RTL clock transition
```

This framing keeps verification tied to the central Assassyn claim: simulation
and implementation are generated from the same architectural description and
should remain cycle-aligned.
