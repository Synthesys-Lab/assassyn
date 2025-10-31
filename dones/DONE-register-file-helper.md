# DONE: Register File Helper Integration

## Achievements
- Reused `assassyn.pycde_wrapper.build_register_file` for Verilog array emission, eliminating bespoke class generation and keeping port naming consistent across generated and hand-authored designs.
- Enhanced the helper to handle optional read-index ports, honour explicit initialisers, and preserve reverse write-port priority, ensuring feature parity with the previous inline implementation.
- Added regression tests that verify generated designs call the helper, cover single-entry arrays, and check that the helper still muxes write ports in descending order.

## Follow-up Ideas
1. Extend the helper to support byte-enable style write strobes so downstream optimisations can materialise partitioned memories without custom code.
2. Add simulator-side coverage that instantiates the helper directly, catching potential regressions in the PyCDE runtime without relying on full design generation.
3. Document the helper usage within the tutorials so frontend authors know it is available when writing PyCDE modules manually.

## Technical Insights
- Moving the reset-value coercion into the generator was necessary because PyCDE requires constants to be emitted within a module context; precomputing them would produce detached SSA values and trigger verification failures.
- Array names pass through `namify` before reaching the helper, so tests must derive expectations via the same utility to remain stable under naming-manager changes.
- Omitting read-index ports for width-one arrays simplifies downstream wiring: the helper now emits data-only outputs in that case, matching the historical behaviour while still keeping multi-entry arrays index-addressable.
