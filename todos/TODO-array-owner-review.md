# TODO: Evaluate Array Owner Classes vs. Simplified `_owner` Model

> **Status 2025-11-01**: Plan executed. `Array.owner` now stores direct module/memory references with identity checks in downstream code (see `dones/DONE-array-owner-simplification.md`). The checklist below documents the investigation pathway that led to the simplification.

## Section 1: Goal

Prepare a comprehensive analysis that explains why `RegisterOwner` and `MemoryOwner` descriptors exist in the current HEAD commit, assess whether they introduce necessary structure beyond storing a raw `MemoryBase`, and determine how best to respond to the concern raised in `dones/DONE-array-owner.md` about potential overengineering.

## Section 2: Action Items

### Analysis & Preparation

1. **Capture the HEAD context**
   - Review `git show --stat HEAD` to enumerate all files affected by the ownership migration.
   - Skim the commit message and metadata to understand the intended scope.

2. **Extract the documented design expectations**
   - Re-read `docs/design/internal/array-ownership.md` to collect the rationale, taxonomy, and lifecycle rules for owners.
   - Check supporting references in `python/assassyn/ir/array.md`, `python/assassyn/ir/memory/{base,sram,dram}.md`, and `docs/design/internal/{pipeline,simulator}.md` for any additional invariants tied to owners.

3. **Summarise prior outcomes**
   - Parse `dones/DONE-array-owner.md` for explicit claims about the benefits and behavioural changes from the migration.
   - Note any promises about simulator, Verilog backend, or tooling relying on structured ownership.

### Code & Test Inspection

4. **Trace owner usage in the IR**
   - Inspect `python/assassyn/ir/array.py` to document how `RegisterOwner` and `MemoryOwner` are defined, validated, and assigned (default resolution vs. explicit memory payload overrides).
   - Examine `Array.assign_owner` and any safeguards around immutability or validation that exceed a plain `_owner = m` pattern.

5. **Inspect memory modules**
   - Review `python/assassyn/ir/memory/base.py`, `sram.py`, and `dram.py` to see how payload and auxiliary arrays rely on `MemoryOwner` roles (`payload`, `dout`) and how they interact with memory subclasses.

6. **Understand downstream consumers**
   - Walk through the simulator (`python/assassyn/codegen/simulator/simulator.py`) and Verilog codegen (`python/assassyn/codegen/verilog/{metadata.py,system.py,top.py,_expr/array.py,cleanup.py}`) to catalogue the precise checks performed on `Array.owner` or its categories.
   - Identify any logic that differentiates between memory payloads, read data latches, or register-owned arrays using structured owners.

7. **Review regression tests**
   - Read `python/unit-tests/test_array_owner.py` to capture the behavioural guarantees enforced by the current owners (immutability, role filtering, default ownership).
   - Note how these tests would behave if the owner were a raw `MemoryBase` reference instead of a typed descriptor.

### Comparative Evaluation

8. **Model the simplified proposal**
   - Define what “set `_owner = m` where `m` is a `MemoryBase`” would look like in code, including how register-owned arrays would represent ownership and how we would check for SRAM vs. DRAM.
   - Determine how the simplified scheme would expose module context, payload roles, and helper metadata (e.g., category strings) currently provided by the dataclasses.

9. **Gap analysis**
   - For each code path documented in steps 4–7, assess whether the simplified approach could replicate the behaviour without additional scaffolding.
   - Highlight scenarios where the owner needs richer data (e.g., module context outside memories, multiple roles per memory, immutability enforcement) and whether those would degrade with a plain reference.

10. **Document trade-offs**
    - Summarise pros/cons: verbosity, type safety, extensibility, downstream clarity, and test coverage.
    - Evaluate maintenance implications (e.g., adding new memory roles or non-memory owners) under both designs.

### Synthesis & Communication

11. **Compile evidence-backed findings**
    - Prepare a structured explanation that references specific files/lines demonstrating why the current design was chosen.
    - Include any contradictions uncovered between documentation and implementation that need reconciliation before responding.

12. **Formulate the response strategy**
    - Decide whether the final write-up should recommend keeping the existing owner descriptors, suggest targeted simplifications, or propose a hybrid.
    - Outline the key points to address in the response (e.g., necessity of role field, simulator expectations, extensibility arguments).

## Section 3: Open Questions

1. Does any subsystem (e.g., metadata exporters) depend on owners being immutable objects rather than mutable references, and how critical is that guarantee?
2. Are there scenarios where arrays must be owned by contexts other than memories or modules (future accelerator integrations), requiring extensible owner descriptors?
3. Would a simplified `_owner` compromise debug tooling or external APIs that expect category/role information?

## Section 4: Potential Follow-ups

1. If the analysis supports the current design, draft a concise FAQ entry or doc addendum clarifying why owners are structured descriptors.
2. If simplifications are viable, propose incremental refactors (with tests) that maintain invariants while reducing perceived overengineering.
3. Evaluate whether helper predicates (e.g., `array.owner.is_memory_payload`) could make the design easier to consume without discarding the dataclass structure.

## Section 5: Completion Checklist

- [ ] HEAD changes and documentation reviewed.
- [ ] Owner usage traced through IR, memory modules, simulator, and Verilog backend.
- [ ] Test expectations summarised.
- [ ] Simplified model evaluated against current behaviour.
- [ ] Evidence-backed response outline prepared.
