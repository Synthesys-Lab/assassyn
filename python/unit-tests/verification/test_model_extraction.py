"""Tests for extracting a normalized validation model from Verilog metadata."""

from assassyn.frontend import Int, Module, Port, SysBuilder, module
from assassyn.codegen.verilog.analysis import collect_fifo_metadata
from assassyn.verification.extract import build_validation_model  # type: ignore


class Consumer(Module):
    """Consumer with one FIFO pop."""

    def __init__(self):
        super().__init__(ports={"data": Port(Int(32))})

    @module.combinational
    def build(self):
        self.pop_all_ports(True)


class Producer(Module):
    """Producer with one async call into the consumer."""

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, consumer: Consumer):
        consumer.async_called(data=Int(32)(11))


def test_validation_model_extracts_async_fifo_and_trigger_relations():
    """The model exposes source-level transitions and RTL signal mappings."""

    sysb = SysBuilder("validation_model")
    with sysb:
        consumer = Consumer()
        consumer.build()
        producer = Producer()
        producer.build(consumer)

    module_metadata, interactions = collect_fifo_metadata(sysb)
    model = build_validation_model(
        sysb,
        module_metadata,
        interactions,
        default_fifo_depth=2,
    )

    assert model.schema == "assassyn.translation_validation.v1"
    assert "module:Consumer" in model.modules
    assert "fifo:Consumer.data" in model.fifos
    assert "async:Producer->Consumer:0" in model.async_calls
    assert model.fifos["fifo:Consumer.data"].configured_depth == 4
    assert model.modules["module:Consumer"].event_count_signal.endswith(".count")
    assert model.triggers["module:Consumer"].rtl_count_signal.endswith(".count")
    assert model.triggers["module:Consumer"].width == 3
    assert model.fifos["fifo:Consumer.data"].rtl.count_signal.endswith(".count")
    assert model.fifos["fifo:Consumer.data"].rtl.data_width == 32
