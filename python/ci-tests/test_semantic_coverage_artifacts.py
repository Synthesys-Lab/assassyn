"""CI test for generated simulator semantic coverage artifacts."""

import json
from pathlib import Path

from assassyn import backend, utils
from assassyn.frontend import Int, Module, Port, SysBuilder, finish, log, module


class Sink(Module):
    """Consumer that logs one value and terminates the run."""

    def __init__(self):
        super().__init__(ports={"data": Port(Int(32))})

    @module.combinational
    def build(self):
        data = self.pop_all_ports(True)
        log("coverage sink {}", data)
        finish()


class Driver(Module):
    """Producer that issues one async call."""

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, sink: Sink):
        sink.async_called(data=Int(32)(5))


def build_system():
    """Build the coverage smoke-test system."""

    sysb = SysBuilder("semantic_coverage_artifacts")
    with sysb:
        sink = Sink()
        sink.build()
        driver = Driver()
        driver.build(sink)
    return sysb


def test_semantic_coverage_json_is_written(tmp_path):
    """Running the generated simulator writes source-level coverage JSON."""

    coverage_path = tmp_path / "coverage.json"
    simulator_path, _ = backend.elaborate(
        build_system(),
        path=str(tmp_path),
        simulator=True,
        verilog=False,
        verbose=False,
        enable_cache=False,
        coverage=True,
        coverage_path=str(coverage_path),
        coverage_roi=(0, 8),
        sim_threshold=8,
        idle_threshold=8,
    )

    utils.run_simulator(simulator_path)

    assert coverage_path.exists()
    coverage = json.loads(coverage_path.read_text(encoding="utf-8"))
    assert coverage["schema"] == "assassyn.semantic_coverage.v1"
    assert "module:Sink" in coverage["objects"]
    assert "fifo:Sink.data" in coverage["counters"]
    assert coverage["counters"]["fifo:Sink.data"]["push"] == 1
    assert coverage["counters"]["fifo:Sink.data"]["pop"] == 1
