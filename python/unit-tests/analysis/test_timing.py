"""Unit tests for Assassyn pre-synthesis critical-path analysis."""

import json
import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))
os.environ.setdefault(
    "ASSASSYN_HOME",
    os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..', '..')),
)

from assassyn.analysis.timing import (  # type: ignore
    CRITICAL_PATHS_REPORT,
    critical_paths,
    write_critical_paths_report,
)
from assassyn.frontend import (  # type: ignore
    Downstream,
    Module,
    Port,
    RegArray,
    SysBuilder,
    UInt,
    Value,
    downstream,
    module,
)
from assassyn.ir.expr.arith import BinaryOp  # type: ignore


def _build_register_dag_system() -> SysBuilder:
    """Build a two-operation RegArray-to-RegArray combinational DAG."""

    sys_builder = SysBuilder("timing_register_dag")
    with sys_builder:

        class Pipe(Module):
            """Single module with an acyclic register-read to register-write path."""

            def __init__(self):
                super().__init__(ports={})

            @module.combinational
            def build(self):
                source = RegArray(UInt(8), 1, name="timing_source")
                sink = RegArray(UInt(8), 1, name="timing_sink")
                first = source[0] + UInt(8)(1)
                second = first + UInt(8)(2)
                (sink & self)[0] <= second

        Pipe().build()

    return sys_builder


def _build_register_boundary_system() -> SysBuilder:
    """Build a direct RegArray read-to-write boundary path."""

    sys_builder = SysBuilder("timing_register_boundary")
    with sys_builder:

        class Pipe(Module):
            """Single module with no combinational work between register boundaries."""

            def __init__(self):
                super().__init__(ports={})

            @module.combinational
            def build(self):
                source = RegArray(UInt(8), 1, name="boundary_source")
                sink = RegArray(UInt(8), 1, name="boundary_sink")
                (sink & self)[0] <= source[0]

        Pipe().build()

    return sys_builder


def _build_fifo_boundary_system() -> SysBuilder:
    """Build a FIFO pop-to-push path with one combinational adder."""

    sys_builder = SysBuilder("timing_fifo_boundary")
    with sys_builder:

        class Pipe(Module):
            """Single-stage FIFO pipeline module."""

            def __init__(self):
                super().__init__(
                    ports={
                        "in0": Port(UInt(8)),
                        "out0": Port(UInt(8)),
                    }
                )

            @module.combinational
            def build(self):
                data = self.in0.pop()
                incremented = data + UInt(8)(1)
                self.out0.push(incremented)

        Pipe().build()

    return sys_builder


def _build_downstream_topology_system() -> SysBuilder:
    """Build downstream modules in reverse order of their data dependencies."""

    sys_builder = SysBuilder("timing_downstream_topology")
    with sys_builder:

        class Producer(Module):
            """Register source for a downstream chain."""

            def __init__(self):
                super().__init__(ports={})

            @module.combinational
            def build(self):
                source = RegArray(UInt(8), 1, name="downstream_source")
                return source[0]

        class AddOne(Downstream):
            """Combinational downstream stage that adds one."""

            def __init__(self, name: str):
                super().__init__()
                self.name = name

            @downstream.combinational
            def build(self, value: Value):
                return value.optional(UInt(8)(0)) + UInt(8)(1)

        class Store(Downstream):
            """Final downstream stage that writes the chain result."""

            def __init__(self):
                super().__init__()
                self.name = "store"

            @downstream.combinational
            def build(self, value: Value):
                sink = RegArray(UInt(8), 1, name="downstream_sink")
                (sink & self)[0] <= value.optional(UInt(8)(0))

        producer = Producer()
        store = Store()
        second = AddOne("second")
        first = AddOne("first")

        source = producer.build()
        first_result = first.build(source)
        second_result = second.build(first_result)
        store.build(second_result)

    return sys_builder


def _build_weighted_operation_system() -> SysBuilder:
    """Build two competing register paths with different operation weights."""

    sys_builder = SysBuilder("timing_weighted_ops")
    with sys_builder:

        class Pipe(Module):
            """Single module with add and multiply paths from the same source."""

            def __init__(self):
                super().__init__(ports={})

            @module.combinational
            def build(self):
                source = RegArray(UInt(8), 1, name="weighted_source")
                add_sink = RegArray(UInt(8), 1, name="weighted_add_sink")
                mul_sink = RegArray(UInt(16), 1, name="weighted_mul_sink")

                value = source[0]
                add_result = value + UInt(8)(1)
                mul_result = value * UInt(8)(3)

                (add_sink & self)[0] <= add_result
                (mul_sink & self)[0] <= mul_result

        Pipe().build()

    return sys_builder


def _assert_path_edges_are_adjacent(path) -> None:
    """Assert each edge connects the corresponding adjacent path nodes."""

    assert [(edge.source, edge.sink) for edge in path.edges] == [
        (source.node_id, sink.node_id)
        for source, sink in zip(path.nodes, path.nodes[1:])
    ]


def test_acyclic_combinational_fixture_reports_ordered_edges():
    """The analyzer should emit a topologically ordered acyclic path."""

    [path] = critical_paths(_build_register_dag_system(), top_n=1)

    assert path.delay == 2.0
    assert [node.kind for node in path.nodes] == [
        "regarray_read",
        "combinational",
        "combinational",
        "regarray_write",
    ]
    assert len({node.node_id for node in path.nodes}) == len(path.nodes)
    _assert_path_edges_are_adjacent(path)


def test_register_boundary_fixture_has_zero_delay_boundaries():
    """RegArray reads are sources and RegArray writes are sinks."""

    [path] = critical_paths(_build_register_boundary_system(), top_n=1)

    assert path.delay == 0.0
    assert path.source.kind == "regarray_read"
    assert path.source.delay == 0.0
    assert path.sink.kind == "regarray_write"
    assert path.sink.delay == 0.0
    assert [node.kind for node in path.nodes] == ["regarray_read", "regarray_write"]


def test_critical_paths_reports_fifo_source_to_fifo_sink():
    """Ensure FIFO pop/push metadata becomes a timed source-to-sink path."""

    paths = critical_paths(_build_fifo_boundary_system(), top_n=1)

    assert len(paths) == 1
    path = paths[0]
    assert path.rank == 1
    assert path.delay == 1.0
    assert path.source.kind == "fifo_pop"
    assert path.sink.kind == "fifo_push"
    assert [node.kind for node in path.nodes] == [
        "fifo_pop",
        "combinational",
        "fifo_push",
    ]
    _assert_path_edges_are_adjacent(path)


def test_downstream_topology_fixture_uses_dependency_order():
    """Downstream modules should be analyzed in topological dataflow order."""

    sys_builder = _build_downstream_topology_system()

    assert [module.name for module in sys_builder.downstreams] == [
        "store",
        "second",
        "first",
    ]

    [path] = critical_paths(sys_builder, top_n=1)

    downstream_modules = [
        node.module
        for node in path.nodes
        if node.module in {"first", "second", "store"}
    ]
    assert downstream_modules == [
        "first",
        "first",
        "second",
        "second",
        "store",
        "store",
    ]
    assert path.source.kind == "regarray_read"
    assert path.sink.kind == "regarray_write"


def test_weighted_operation_fixture_ranks_default_and_custom_delays():
    """Operation weights should drive path ranking and accept overrides."""

    sys_builder = _build_weighted_operation_system()

    default_paths = critical_paths(sys_builder, top_n=2)
    assert [path.delay for path in default_paths] == [4.0, 1.0]
    assert any(node.opcode == BinaryOp.MUL for node in default_paths[0].nodes)
    assert any(node.opcode == BinaryOp.ADD for node in default_paths[1].nodes)

    custom_paths = critical_paths(
        sys_builder,
        delay_model={
            BinaryOp.ADD: 7,
            BinaryOp.MUL: 1,
        },
        top_n=2,
    )
    assert [path.delay for path in custom_paths] == [7.0, 1.0]
    assert any(node.opcode == BinaryOp.ADD for node in custom_paths[0].nodes)
    assert any(node.opcode == BinaryOp.MUL for node in custom_paths[1].nodes)


def test_write_critical_paths_report_uses_shared_filename(tmp_path):
    """Check the JSON report schema and shared report filename constant."""

    report_path = tmp_path / CRITICAL_PATHS_REPORT
    returned = write_critical_paths_report(
        _build_fifo_boundary_system(),
        report_path,
        top_n=1,
    )

    assert returned == report_path
    payload = json.loads(report_path.read_text(encoding="utf-8"))
    assert payload["schema"] == "assassyn.critical_paths.v1"
    assert payload["top_n"] == 1
    assert payload["paths"][0]["delay"] == 1.0
    assert payload["paths"][0]["source"]["kind"] == "fifo_pop"
    assert payload["paths"][0]["sink"]["kind"] == "fifo_push"
