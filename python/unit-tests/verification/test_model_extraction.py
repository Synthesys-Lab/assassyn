"""Tests for extracting a normalized validation model from Verilog metadata."""

from assassyn.frontend import Int, Module, Port, RegArray, SysBuilder, UInt, module
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


class ArrayDriver(Module):
    """Driver with one RegArray read and write relation."""

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        storage = RegArray(UInt(8), 4, initializer=[0, 0, 0, 0], name="verify_arr")
        value = storage[UInt(2)(1)]
        (storage & self)[UInt(2)(1)] <= value + UInt(8)(1)


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


def test_validation_model_extracts_array_transitions():
    """The model exposes RegArray write/read port signal mappings."""

    sysb = SysBuilder("array_validation_model")
    with sysb:
        driver = ArrayDriver()
        driver.build()

    module_metadata, interactions = collect_fifo_metadata(sysb)
    model = build_validation_model(
        sysb,
        module_metadata,
        interactions,
        default_fifo_depth=2,
    )

    assert "array:storage" in model.arrays
    transition = model.arrays["array:storage"]
    assert transition.depth == 4
    assert transition.index_width == 2
    assert transition.data_width == 8
    assert len(transition.write_ports) == 1
    assert len(transition.read_ports) == 1
    assert transition.write_ports[0].write_enable_signal == "array_writer_storage.w_port0"
    assert transition.write_ports[0].write_index_signal == "array_writer_storage.widx_port0"
    assert transition.write_ports[0].write_data_signal == "array_writer_storage.wdata_port0"
    assert (
        transition.write_ports[0].next_value_signal
        == "array_writer_storage.mem[array_writer_storage.widx_port0]"
    )
    assert transition.read_ports[0].read_index_signal == "array_writer_storage.ridx_port0"
    assert transition.read_ports[0].read_data_signal == "array_writer_storage.rdata_port0"
