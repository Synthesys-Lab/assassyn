# Timing Analysis Unit Tests

`test_timing.py` covers the public pre-synthesis critical-path analysis API
with real Assassyn IR fixtures.

## Interface Exposed

- `test_acyclic_combinational_fixture_reports_ordered_edges()`: Verifies that
  a register-read to register-write DAG reports ordered nodes and adjacent
  edges.
- `test_register_boundary_fixture_has_zero_delay_boundaries()`: Verifies that
  `RegArray` reads are timing sources, `RegArray` writes are timing sinks, and
  both boundary nodes contribute zero local delay.
- `test_critical_paths_reports_fifo_source_to_fifo_sink()`: Verifies that FIFO
  pops and pushes form the source and sink around a combinational adder.
- `test_downstream_topology_fixture_uses_dependency_order()`: Verifies that
  downstream modules are analyzed in data-dependency order even when they are
  constructed in reverse order.
- `test_weighted_operation_fixture_ranks_default_and_custom_delays()`: Verifies
  the default multiply/add weights and caller-supplied delay overrides.
- `test_write_critical_paths_report_uses_shared_filename()`: Verifies report
  writing, schema fields, and the shared `CRITICAL_PATHS_REPORT` filename.

## Internal Helpers

- `_build_register_dag_system()`: Builds a two-addition register-to-register
  path used to check acyclic graph ordering.
- `_build_register_boundary_system()`: Builds a direct register read-to-write
  path used to isolate sequential-boundary classification.
- `_build_fifo_boundary_system()`: Builds a FIFO pop-to-push path with one
  combinational operation.
- `_build_downstream_topology_system()`: Builds a reverse-constructed
  downstream chain that must be sorted before analysis.
- `_build_weighted_operation_system()`: Builds competing add and multiply
  paths from one register source.
- `_assert_path_edges_are_adjacent()`: Checks that serialized path edges match
  the adjacent node ordering in the same report entry.

## Data Structures

The fixtures build `SysBuilder` instances containing local `Module` and
`Downstream` subclasses. Assertions inspect the `CriticalPath`, `TimingNode`,
and `TimingEdge` objects returned by `critical_paths()` and serialized by
`write_critical_paths_report()`, including node roles, opcodes, delays, module
names, edge endpoints, and report schema fields.
