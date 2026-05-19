# Timing Analysis

This module implements a pre-synthesis critical-path estimate over the
Assassyn IR. It operates before PyCDE or Verilog emission, so it reports paths
through IR expressions rather than technology-mapped gates.

## Section 1. Exposed Interfaces

### `TimingNode`

```python
@dataclass(frozen=True)
class TimingNode:
    node_id: str
    module: str
    symbol: str
    kind: str
    delay: float
    opcode: int | None
    loc: str | None
    expr: str
```

`TimingNode` is the serialized form of one expression on a path. The fields
record the report identifier, owning module, IR symbol, timing role, local
delay, opcode, source location, and printable IR expression. `to_dict()`
returns the JSON schema used by `critical_paths.json`.

### `CRITICAL_PATHS_REPORT`

```python
CRITICAL_PATHS_REPORT = "critical_paths.json"
```

Shared report filename used by the backend and code-generation entrypoints.

### `TimingEdge`

```python
@dataclass(frozen=True)
class TimingEdge:
    source: str
    sink: str
```

`TimingEdge` records one adjacent dependency in a reported path. `source` and
`sink` are `TimingNode.node_id` values.

### `CriticalPath`

```python
@dataclass(frozen=True)
class CriticalPath:
    rank: int
    delay: float
    source: TimingNode
    sink: TimingNode
    nodes: tuple[TimingNode, ...]
    edges: tuple[TimingEdge, ...]
```

`CriticalPath` is one ranked source-to-sink result. The path delay is the sum
of expression delays along `nodes`, and `edges` preserves the same ordering for
graph-oriented consumers.

### `critical_paths`

```python
def critical_paths(
    sys: SysBuilder,
    delay_model: DelayModel | None = None,
    top_n: int = 10,
) -> list[CriticalPath]:
```

Returns the longest combinational paths in `sys`, sorted from longest to
shortest. The analysis builds a DAG over `Expr.operands` and `Expr.users`, while
`topo_downstream_modules()` provides deterministic ordering for downstream
combinational modules.

Sequential boundaries are derived from the same Verilog metadata pass used by
code generation:

- `ArrayRead` from a `RegArray` is a source.
- `ArrayWrite` into a `RegArray` is a sink.
- `FIFOPop` is a source.
- `FIFOPush` is a sink.
- `AsyncCall` is a sink for stage-transfer control.
- `PureIntrinsic.EXTERNAL_OUTPUT_READ` from an `ExternalSV.RegOut` is a source.

`collect_fifo_metadata()` provides frozen module, array, FIFO, and async-call
interaction views. These views prevent the timing pass from rediscovering
shared-resource roles with separate heuristics.

The default delay model is a coarse IR-level estimate: sources and sinks have
zero delay, most valued expressions have unit delay, multiplication has delay
`4`, division and modulo have delay `8`, and pure FIFO status reads have zero
delay. A custom model may be a callable `delay_model(expr)` or a mapping keyed
by expression object, expression class, class name, or opcode.

### `write_critical_paths_report`

```python
def write_critical_paths_report(
    sys: SysBuilder,
    path: Path,
    delay_model: DelayModel | None = None,
    top_n: int = 10,
) -> Path:
```

Writes a JSON report with this top-level schema:

```json
{
  "schema": "assassyn.critical_paths.v1",
  "top_n": 10,
  "paths": []
}
```

The backend uses this helper when `backend.config(timing_report=True)` is
enabled and writes `critical_paths.json` in the system output directory.

## Section 2. Internal Helpers

### `_Boundaries`

`_Boundaries` stores the source and sink sets plus their report labels. It is
private because boundary construction is intentionally tied to the current
metadata model.

### `_ordered_modules`

```python
def _ordered_modules(sys: SysBuilder) -> list[Module]:
```

Returns regular pipeline modules followed by downstream modules sorted with
`topo_downstream_modules()`. This ordering keeps cross-downstream expression
edges deterministic.

### `_collect_boundaries`

```python
def _collect_boundaries(
    modules: list[Module],
    module_metadata: Mapping[Module, object],
    interactions: object,
) -> _Boundaries:
```

Reads each module's frozen `ModuleMetadata.interactions` view plus the async
ledger to classify expressions as timing sources or sinks. The module view is
already scoped to the current module, so the helper does not need to re-filter
global array or FIFO views. It also scans module bodies for ExternalSV
registered output reads.

### `_collect_nodes`

```python
def _collect_nodes(modules: list[Module], boundaries: _Boundaries) -> list[Expr]:
```

Keeps hardware-relevant valued expressions plus explicit boundary nodes. Bind,
predicate-stack, logging, wait, finish, and assert-only nodes are omitted unless
they are part of a requested timing boundary.

### `_build_graph`

```python
def _build_graph(
    modules: list[Module],
    nodes: list[Expr],
    boundaries: _Boundaries,
) -> tuple[dict[Expr, set[Expr]], dict[Expr, set[Expr]]]:
```

Constructs the combinational DAG. Operand edges come from `Expr.operands`,
back-edge validation and cross-module uses come from `Expr.users`, predicate
guards come from `Expr.meta_cond`, and wait predicates are threaded onto later
statements in the same way as the Verilog dumper.

### `_topological_nodes`

```python
def _topological_nodes(
    nodes: list[Expr],
    successors: Mapping[Expr, set[Expr]],
) -> list[Expr]:
```

Performs Kahn topological sorting over the expression DAG. A remaining cycle is
reported as a `ValueError` because the analyzer cannot assign a finite
combinational path delay to cyclic logic.

### `_resolve_delay` and `_default_delay`

`_resolve_delay()` checks the caller-supplied delay model before falling back to
`_default_delay()`. `_default_delay()` keeps the built-in model deliberately
coarse so reports remain useful before synthesis without claiming gate-level
accuracy.

### `_describe_node`

Converts an IR expression and resolved delay into a `TimingNode`. The helper
centralizes report naming so path objects and JSON output use identical node
identifiers.
