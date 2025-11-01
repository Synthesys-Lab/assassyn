"""Tests for external metadata aggregation."""

from types import SimpleNamespace

from assassyn.builder import SysBuilder
from assassyn.codegen.verilog.analysis import collect_external_metadata
from assassyn.codegen.verilog.metadata import ExternalRead, ExternalRegistry
from assassyn.ir.dtype import UInt
from assassyn.ir.expr.intrinsic import ExternalIntrinsic, PureIntrinsic
from assassyn.ir.module.external import ExternalSV, WireSpec


class DummyExternal(ExternalSV):  # type: ignore[misc]
    """Simple external module used for metadata tests."""


DummyExternal.set_metadata({
    "source": "dummy.sv",
    "module_name": "DummyExternal",
})
DummyExternal.set_port_specs(
    {
        "value": WireSpec(
            name="value",
            dtype=UInt(8),
            direction="out",
            kind="wire",
        ),
    }
)


def _make_external_instance(owner):
    """Create an ExternalIntrinsic owned by *owner*."""
    instance = ExternalIntrinsic(DummyExternal)
    instance.parent = owner
    owner.body.append(instance)
    return instance


def _make_system_with_cross_read():
    """Construct a minimalist system containing a cross-module external read."""

    builder = SysBuilder("external-metadata-test")
    with builder:
        producer = ModuleStub("producer")
        consumer = ModuleStub("consumer")
        downstream = ModuleStub("downstream")

        builder.modules.extend([producer])
        builder.downstreams.extend([consumer, downstream])

        producer_ctx = SimpleNamespace(module=producer, cond_stack=[])
        builder._module_stack.append(producer_ctx)  # pylint: disable=protected-access
        instance = _make_external_instance(producer)
        builder._module_stack.pop()  # pylint: disable=protected-access

        consumer_ctx = SimpleNamespace(module=consumer, cond_stack=[])
        builder._module_stack.append(consumer_ctx)  # pylint: disable=protected-access
        read_expr = PureIntrinsic(PureIntrinsic.EXTERNAL_OUTPUT_READ, instance, "value")
        builder._module_stack.pop()  # pylint: disable=protected-access
        read_expr.parent = consumer
        consumer.body.append(read_expr)

    return builder, producer, consumer, instance, read_expr


def test_collect_external_metadata_records_cross_module_reads():
    """Ensure collect_external_metadata captures producers, consumers, and classes."""

    system, producer, consumer, instance, read_expr = _make_system_with_cross_read()

    registry = collect_external_metadata(system)
    assert isinstance(registry, ExternalRegistry)
    assert registry.frozen

    assert registry.classes == (DummyExternal,)
    assert registry.owner_for(instance) is producer
    reads = registry.cross_module_reads
    assert len(reads) == 1
    record = reads[0]
    assert isinstance(record, ExternalRead)
    assert record.consumer is consumer
    assert record.producer is producer
    assert record.instance is instance
    assert record.expr is read_expr
    assert record.port_name == "value"
    assert registry.reads_for_consumer(consumer) == (record,)
    assert registry.reads_for_instance(instance) == (record,)
    assert registry.reads_for_producer(producer) == (record,)
class ModuleStub(SimpleNamespace):
    """Minimal module representation needed for external metadata tests."""

    def __init__(self, name: str):
        super().__init__(name=name, body=[], externals=[], cond_stack=[])

    def __hash__(self) -> int:  # pragma: no cover - trivial hash helper
        return hash(self.name)

    def __eq__(self, other) -> bool:  # pragma: no cover - trivial equality helper
        if not isinstance(other, ModuleStub):
            return NotImplemented
        return self.name == other.name
