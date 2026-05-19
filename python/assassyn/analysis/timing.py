"""Pre-synthesis timing analysis for Assassyn IR systems."""

from __future__ import annotations

from collections import deque
from collections.abc import Callable, Mapping
from dataclasses import dataclass
import json
from pathlib import Path
import typing

from ..ir.expr import AsyncCall, Expr, FIFOPush
from ..ir.expr.arith import BinaryOp
from ..ir.expr.array import ArrayWrite
from ..ir.expr.call import Bind
from ..ir.expr.intrinsic import ExternalIntrinsic, Intrinsic, PureIntrinsic
from ..utils import identifierize, unwrap_operand
from ..utils.enforce_type import enforce_type
from .topo import topo_downstream_modules

if typing.TYPE_CHECKING:
    from ..builder import SysBuilder
    from ..ir.module import Module
else:
    SysBuilder = typing.Any
    Module = typing.Any

DelayModel = Callable[[Expr], float | int | None] | Mapping[object, float | int]

CRITICAL_PATHS_REPORT = "critical_paths.json"

_SOURCE_ARRAY_READ = "regarray_read"
_SOURCE_FIFO_POP = "fifo_pop"
_SOURCE_EXTERNAL_REG = "external_reg_output"
_SINK_ARRAY_WRITE = "regarray_write"
_SINK_FIFO_PUSH = "fifo_push"
_SINK_ASYNC_CALL = "async_stage_transfer"
_KIND_COMBINATIONAL = "combinational"


@dataclass(frozen=True)
class TimingNode:
    """Serializable description of an expression on a timing path."""

    node_id: str
    module: str
    symbol: str
    kind: str
    delay: float
    opcode: int | None
    loc: str | None
    expr: str

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-serializable dictionary for this node."""

        return {
            "id": self.node_id,
            "module": self.module,
            "symbol": self.symbol,
            "kind": self.kind,
            "delay": self.delay,
            "opcode": self.opcode,
            "loc": self.loc,
            "expr": self.expr,
        }


@dataclass(frozen=True)
class TimingEdge:
    """Serializable edge between two adjacent timing path nodes."""

    source: str
    sink: str

    def to_dict(self) -> dict[str, str]:
        """Return a JSON-serializable dictionary for this edge."""

        return {
            "source": self.source,
            "sink": self.sink,
        }


@dataclass(frozen=True)
class CriticalPath:
    """One ranked source-to-sink timing path."""

    rank: int
    delay: float
    source: TimingNode
    sink: TimingNode
    nodes: tuple[TimingNode, ...]
    edges: tuple[TimingEdge, ...]

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-serializable dictionary for this path."""

        return {
            "rank": self.rank,
            "delay": self.delay,
            "source": self.source.to_dict(),
            "sink": self.sink.to_dict(),
            "nodes": [node.to_dict() for node in self.nodes],
            "edges": [edge.to_dict() for edge in self.edges],
        }


@dataclass(frozen=True)
class _Boundaries:
    sources: frozenset[Expr]
    sinks: frozenset[Expr]
    source_kinds: Mapping[Expr, str]
    sink_kinds: Mapping[Expr, str]


@enforce_type
def critical_paths(
    sys: SysBuilder,
    delay_model: DelayModel | None = None,
    top_n: int = 10,
) -> list[CriticalPath]:
    """Return the longest combinational paths between sequential boundaries.

    @param sys The Assassyn system builder to analyse.
    @param delay_model Optional callable or mapping that returns per-expression delays.
    @param top_n Maximum number of paths to return.
    @return Ranked critical paths sorted from longest to shortest delay.
    """

    if top_n <= 0:
        return []

    modules = _ordered_modules(sys)
    if not modules:
        return []

    from ..codegen.verilog.analysis import collect_fifo_metadata

    module_metadata, interactions = collect_fifo_metadata(sys)
    boundaries = _collect_boundaries(modules, module_metadata, interactions)
    nodes = _collect_nodes(modules, boundaries)
    if not nodes or not boundaries.sinks:
        return []

    successors, predecessors = _build_graph(modules, nodes, boundaries)
    ordered_nodes = _topological_nodes(nodes, successors)
    delay_for = {
        node: _resolve_delay(node, boundaries, delay_model)
        for node in ordered_nodes
    }

    best_delay: dict[Expr, float] = {}
    best_path: dict[Expr, tuple[Expr, ...]] = {}
    order_index = {node: idx for idx, node in enumerate(nodes)}

    for node in ordered_nodes:
        preds = predecessors[node]
        if not preds:
            best_delay[node] = delay_for[node]
            best_path[node] = (node,)
            continue

        chosen = max(
            preds,
            key=lambda pred: (best_delay[pred], -order_index[pred]),
        )
        best_delay[node] = best_delay[chosen] + delay_for[node]
        best_path[node] = best_path[chosen] + (node,)

    sink_nodes = [sink for sink in boundaries.sinks if sink in best_delay]
    sink_nodes.sort(key=lambda sink: (-best_delay[sink], _node_id(sink)))

    paths: list[CriticalPath] = []
    for rank, sink in enumerate(sink_nodes[:top_n], start=1):
        expr_path = best_path[sink]
        timing_nodes = tuple(
            _describe_node(expr, boundaries, delay_for[expr])
            for expr in expr_path
        )
        timing_edges = tuple(
            TimingEdge(_node_id(lhs), _node_id(rhs))
            for lhs, rhs in zip(expr_path, expr_path[1:])
        )
        paths.append(
            CriticalPath(
                rank=rank,
                delay=best_delay[sink],
                source=timing_nodes[0],
                sink=timing_nodes[-1],
                nodes=timing_nodes,
                edges=timing_edges,
            )
        )

    return paths


@enforce_type
def write_critical_paths_report(
    sys: SysBuilder,
    path: Path,
    delay_model: DelayModel | None = None,
    top_n: int = 10,
) -> Path:
    """Write a ``critical_paths.json`` report and return the output path.

    @param sys The Assassyn system builder to analyse.
    @param path Destination JSON path.
    @param delay_model Optional callable or mapping passed to ``critical_paths``.
    @param top_n Maximum number of paths to include.
    @return The destination path.
    """

    paths = critical_paths(sys, delay_model=delay_model, top_n=top_n)
    payload = {
        "schema": "assassyn.critical_paths.v1",
        "top_n": top_n,
        "paths": [path_entry.to_dict() for path_entry in paths],
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return path


def _ordered_modules(sys: SysBuilder) -> list[Module]:
    """Return pipeline modules followed by downstreams in topological order."""

    modules = list(getattr(sys, "modules", ()))
    downstreams = topo_downstream_modules(sys)
    return modules + downstreams


def _collect_boundaries(
    modules: list[Module],
    module_metadata: Mapping[Module, object],
    interactions: object,
) -> _Boundaries:
    """Collect timing sources and sinks from Verilog interaction metadata."""

    source_kinds: dict[Expr, str] = {}
    sink_kinds: dict[Expr, str] = {}

    for module in modules:
        metadata = module_metadata.get(module)
        if metadata is None:
            continue
        view = metadata.interactions
        for pop in view.pops:
            source_kinds[pop] = _SOURCE_FIFO_POP
        for push in view.pushes:
            sink_kinds[push] = _SINK_FIFO_PUSH
        for reads in view.reads.values():
            for read in reads:
                source_kinds[read] = _SOURCE_ARRAY_READ
        for writes in view.writes.values():
            for write in writes:
                sink_kinds[write] = _SINK_ARRAY_WRITE
        # Async calls are not resource interactions, so they stay in the
        # ledger rather than the module-scoped FIFO/array view.
        for calls in interactions.async_ledger.calls_for_module(module).values():
            for call in calls:
                sink_kinds[call] = _SINK_ASYNC_CALL

    for module in modules:
        body = getattr(module, "body", None)
        if not isinstance(body, list):
            continue
        for expr in body:
            if _is_external_reg_output(expr):
                source_kinds[expr] = _SOURCE_EXTERNAL_REG

    return _Boundaries(
        sources=frozenset(source_kinds),
        sinks=frozenset(sink_kinds),
        source_kinds=source_kinds,
        sink_kinds=sink_kinds,
    )


def _collect_nodes(modules: list[Module], boundaries: _Boundaries) -> list[Expr]:
    """Collect expressions that participate in the combinational timing DAG."""

    nodes: list[Expr] = []
    seen: set[Expr] = set()
    for module in modules:
        body = getattr(module, "body", None)
        if not isinstance(body, list):
            continue
        for expr in body:
            if not _include_expr(expr, boundaries):
                continue
            if expr in seen:
                continue
            nodes.append(expr)
            seen.add(expr)
    return nodes


def _include_expr(expr: object, boundaries: _Boundaries) -> bool:
    """Return whether an expression should appear in the timing graph."""

    if not isinstance(expr, Expr):
        return False
    if expr in boundaries.sources or expr in boundaries.sinks:
        return True
    if isinstance(expr, (Bind, ArrayWrite, FIFOPush, AsyncCall)):
        return False
    if isinstance(expr, Intrinsic) and not isinstance(expr, ExternalIntrinsic):
        return False
    try:
        return expr.is_valued()
    except (AttributeError, NotImplementedError):
        return False


def _build_graph(
    modules: list[Module],
    nodes: list[Expr],
    boundaries: _Boundaries,
) -> tuple[dict[Expr, set[Expr]], dict[Expr, set[Expr]]]:
    """Build expression dependency edges for the combinational timing DAG."""

    node_set = set(nodes)
    successors: dict[Expr, set[Expr]] = {node: set() for node in nodes}
    predecessors: dict[Expr, set[Expr]] = {node: set() for node in nodes}
    wait_dependencies = _collect_wait_dependencies(modules)

    def add_edge(source: Expr, sink: Expr) -> None:
        if source is sink:
            return
        if source not in node_set or sink not in node_set:
            return
        if sink in boundaries.sources:
            return
        successors[source].add(sink)
        predecessors[sink].add(source)

    for node in nodes:
        if node not in boundaries.sources:
            for dep in _operand_dependencies(node):
                add_edge(dep, node)
            for dep in _metadata_dependencies(node):
                add_edge(dep, node)
            for dep in wait_dependencies.get(node, ()):
                add_edge(dep, node)
        for user_expr in _user_dependencies(node):
            add_edge(node, user_expr)

    return successors, predecessors


def _collect_wait_dependencies(modules: list[Module]) -> dict[Expr, tuple[Expr, ...]]:
    """Map each expression to the active wait predicates that gate it."""

    dependencies: dict[Expr, tuple[Expr, ...]] = {}
    for module in modules:
        active_waits: list[Expr] = []
        body = getattr(module, "body", None)
        if not isinstance(body, list):
            continue
        for expr in body:
            # wait_until gates every later expression in the module trace; this
            # mirrors CIRCTDumper.wait_conditions during Verilog generation.
            if (
                isinstance(expr, Intrinsic)
                and not isinstance(expr, ExternalIntrinsic)
                and expr.opcode == Intrinsic.WAIT_UNTIL
            ):
                if expr.args:
                    dep = _as_expr(expr.args[0])
                    if dep is not None:
                        active_waits.append(dep)
                continue
            if isinstance(expr, Expr) and active_waits:
                dependencies[expr] = tuple(active_waits)
    return dependencies


def _operand_dependencies(expr: Expr) -> tuple[Expr, ...]:
    """Return expression operands that feed *expr*."""

    if isinstance(expr, AsyncCall):
        return ()
    return tuple(
        dep
        for dep in (_as_expr(operand) for operand in expr.operands)
        if dep is not None
    )


def _metadata_dependencies(expr: Expr) -> tuple[Expr, ...]:
    """Return predicate metadata dependencies for *expr*."""

    dep = _as_expr(getattr(expr, "meta_cond", None))
    if dep is None or dep is expr:
        return ()
    return (dep,)


def _user_dependencies(expr: Expr) -> tuple[Expr, ...]:
    """Return expression users recorded by the IR back-edge lists."""

    users: list[Expr] = []
    for user in getattr(expr, "users", ()):
        user_expr = getattr(user, "user", user)
        if isinstance(user_expr, Expr):
            users.append(user_expr)
    return tuple(users)


def _topological_nodes(
    nodes: list[Expr],
    successors: Mapping[Expr, set[Expr]],
) -> list[Expr]:
    """Topologically order expression nodes or raise on combinational cycles."""

    index = {node: position for position, node in enumerate(nodes)}
    in_degree = {node: 0 for node in nodes}
    for source in nodes:
        for sink in successors[source]:
            in_degree[sink] += 1

    queue = deque(node for node in nodes if in_degree[node] == 0)
    ordered: list[Expr] = []
    while queue:
        node = queue.popleft()
        ordered.append(node)
        for sink in sorted(successors[node], key=index.__getitem__):
            in_degree[sink] -= 1
            if in_degree[sink] == 0:
                queue.append(sink)

    if len(ordered) != len(nodes):
        cycle_nodes = [node for node in nodes if in_degree[node] > 0]
        names = ", ".join(_node_id(node) for node in cycle_nodes[:8])
        raise ValueError(f"Combinational timing cycle detected: {names}")

    return ordered


def _resolve_delay(
    expr: Expr,
    boundaries: _Boundaries,
    delay_model: DelayModel | None,
) -> float:
    """Resolve an expression delay from an override model or defaults."""

    if callable(delay_model):
        value = delay_model(expr)
        if value is not None:
            return float(value)
    elif isinstance(delay_model, Mapping):
        for key in (expr, type(expr), type(expr).__name__, getattr(expr, "opcode", None)):
            if key is not None and key in delay_model:
                return float(delay_model[key])
    return _default_delay(expr, boundaries)


def _default_delay(expr: Expr, boundaries: _Boundaries) -> float:
    """Return conservative unit-style delay weights for IR expressions."""

    if expr in boundaries.sources or expr in boundaries.sinks:
        return 0.0
    if isinstance(expr, BinaryOp):
        if expr.opcode == BinaryOp.MUL:
            return 4.0
        if expr.opcode in (BinaryOp.DIV, BinaryOp.MOD):
            return 8.0
        return 1.0
    if isinstance(expr, PureIntrinsic):
        if expr.opcode == PureIntrinsic.EXTERNAL_OUTPUT_READ:
            return 1.0
        return 0.0
    if isinstance(expr, ExternalIntrinsic):
        return 1.0
    if isinstance(expr, Intrinsic):
        return 0.0
    try:
        return 1.0 if expr.is_valued() else 0.0
    except (AttributeError, NotImplementedError):
        return 0.0


def _describe_node(expr: Expr, boundaries: _Boundaries, delay: float) -> TimingNode:
    """Create the serialized node description for *expr*."""

    return TimingNode(
        node_id=_node_id(expr),
        module=_module_name(getattr(expr, "parent", None)),
        symbol=_symbol(expr),
        kind=_node_kind(expr, boundaries),
        delay=delay,
        opcode=getattr(expr, "opcode", None),
        loc=getattr(expr, "loc", None),
        expr=repr(expr),
    )


def _node_kind(expr: Expr, boundaries: _Boundaries) -> str:
    """Return the timing role for *expr*."""

    if expr in boundaries.source_kinds:
        return boundaries.source_kinds[expr]
    if expr in boundaries.sink_kinds:
        return boundaries.sink_kinds[expr]
    return _KIND_COMBINATIONAL


def _node_id(expr: Expr) -> str:
    """Return a stable-enough identifier for a node within one elaboration run."""

    module = _module_name(getattr(expr, "parent", None))
    return f"{module}.{_symbol(expr)}"


def _symbol(expr: Expr) -> str:
    """Return the IR symbol for an expression."""

    try:
        return expr.as_operand()
    except AttributeError:
        return f"expr_{identifierize(expr)}"


def _module_name(module: object) -> str:
    """Return a printable module name for report entries."""

    name = getattr(module, "name", None)
    if name is not None:
        return str(name)
    return "<unknown>"


def _as_expr(value: object) -> Expr | None:
    """Unwrap operands and return expression values only."""

    unwrapped = unwrap_operand(value)
    if isinstance(unwrapped, Expr):
        return unwrapped
    return None


def _is_external_reg_output(expr: object) -> bool:
    """Return whether *expr* reads a registered ExternalSV output."""

    if not isinstance(expr, PureIntrinsic):
        return False
    if expr.opcode != PureIntrinsic.EXTERNAL_OUTPUT_READ:
        return False
    if len(expr.args) < 2:
        return False

    instance = unwrap_operand(expr.args[0])
    port_name = unwrap_operand(expr.args[1])
    if not isinstance(instance, ExternalIntrinsic):
        return False
    spec = instance.external_class.port_specs().get(port_name)
    return spec is not None and spec.kind == "reg"


__all__ = [
    "CriticalPath",
    "CRITICAL_PATHS_REPORT",
    "TimingEdge",
    "TimingNode",
    "critical_paths",
    "write_critical_paths_report",
]
