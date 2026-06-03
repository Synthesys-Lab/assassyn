"""CI test for generated translation-validation artifacts."""

import json

from assassyn import backend
from assassyn.frontend import Int, Module, Port, SysBuilder, module


class Target(Module):
    """Target with one FIFO input."""

    def __init__(self):
        super().__init__(ports={"data": Port(Int(32))})

    @module.combinational
    def build(self):
        """Consume all ports so the target has FIFO pop logic."""

        self.pop_all_ports(True)


class Driver(Module):
    """Driver that sends one async call."""

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, target: Target):
        """Send one deterministic async call to the target."""

        target.async_called(data=Int(32)(13))


def build_system():
    """Build the translation-validation artifact smoke-test system."""

    sysb = SysBuilder("translation_validation_artifacts")
    with sysb:
        target = Target()
        target.build()
        driver = Driver()
        driver.build(target)
    return sysb


def test_translation_validation_json_is_written(tmp_path):
    """Verilog elaboration writes validation JSON and monitor artifacts."""

    _, verilog_path = backend.elaborate(
        build_system(),
        path=str(tmp_path),
        simulator=False,
        verilog=True,
        verbose=False,
        enable_cache=False,
        verification=True,
        sim_threshold=4,
    )

    validation_path = verilog_path / "translation_validation.json"
    monitor_path = verilog_path / "translation_validation_monitor.sv"

    assert validation_path.exists()
    assert monitor_path.exists()

    validation = json.loads(validation_path.read_text(encoding="utf-8"))
    assert validation["schema"] == "assassyn.translation_validation.v1"
    assert "module:Target" in validation["modules"]
    assert "fifo:Target.data" in validation["fifos"]

    monitor = monitor_path.read_text(encoding="utf-8")
    assert "bind Top translation_validation_monitor" in monitor
    assert "FIFO pop without valid" in monitor
    assert "fifo_Target_data_count" in monitor
