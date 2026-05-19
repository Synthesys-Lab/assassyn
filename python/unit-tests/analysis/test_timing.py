"""Unit tests for pre-synthesis timing analysis."""

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
from assassyn.frontend import Module, Port, SysBuilder, UInt, module  # type: ignore


def _build_fifo_adder_system() -> SysBuilder:
    """Build a minimal FIFO-to-FIFO path with one combinational adder."""

    sysb = SysBuilder("timing_fifo_adder")
    with sysb:

        class Pipe(Module):
            """Single-stage module used to exercise timing boundaries."""

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

    return sysb


def test_critical_paths_reports_fifo_source_to_fifo_sink():
    """Ensure FIFO pop/push metadata becomes a timed source-to-sink path."""

    paths = critical_paths(_build_fifo_adder_system(), top_n=1)

    assert len(paths) == 1
    path = paths[0]
    assert path.rank == 1
    assert path.delay == 1.0
    assert path.source.kind == "fifo_pop"
    assert path.sink.kind == "fifo_push"
    assert [edge.source for edge in path.edges] == [
        node.node_id for node in path.nodes[:-1]
    ]
    assert [edge.sink for edge in path.edges] == [
        node.node_id for node in path.nodes[1:]
    ]


def test_write_critical_paths_report_uses_shared_filename(tmp_path):
    """Check the JSON report schema and shared report filename constant."""

    report_path = tmp_path / CRITICAL_PATHS_REPORT
    returned = write_critical_paths_report(
        _build_fifo_adder_system(),
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
