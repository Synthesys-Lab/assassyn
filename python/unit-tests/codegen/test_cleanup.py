"""Regression coverage for cleanup predicate-driven mux generation."""

import os
import sys
from typing import Dict, Iterable, List, Tuple

sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (  # type: ignore
    Condition,
    Module,
    Port,
    RegArray,
    SysBuilder,
    UInt,
    module,
    push_condition,
    pop_condition,
)
from assassyn.codegen.verilog.cleanup import (  # type: ignore
    _emit_predicate_mux_chain,
    _format_reduction_expr,
)
from assassyn.codegen.verilog.design import CIRCTDumper  # type: ignore
from assassyn.codegen.verilog.analysis import collect_fifo_metadata  # type: ignore
from assassyn.ir.expr.call import FIFOPush
from assassyn.ir.expr.expr import FIFOPop
from assassyn.utils import namify  # type: ignore


def _render_cleanup_lines() -> Tuple[List[str], Dict[str, str]]:
    """Generate cleanup code for a multi-writer module and capture naming context."""
    sys_builder = SysBuilder("cleanup_mux_reference")
    with sys_builder:

        class MultiWriter(Module):  # type: ignore[misc]

            def __init__(self):
                super().__init__(ports={
                    'pred0': Port(UInt(1)),
                    'pred1': Port(UInt(1)),
                    'idx0': Port(UInt(2)),
                    'idx1': Port(UInt(2)),
                    'val0': Port(UInt(8)),
                    'val1': Port(UInt(8)),
                    'fifo': Port(UInt(8)),
                })

            @module.combinational
            def build(self):
                pred0 = self.pred0.pop()
                pred1 = self.pred1.pop()
                idx0 = self.idx0.pop()
                idx1 = self.idx1.pop()
                val0 = self.val0.pop()
                val1 = self.val1.pop()

                array = RegArray(UInt(8), 4, name="target_array")

                with Condition(pred0):
                    write_port = array & self
                    write_port[idx0] <= val0

                with Condition(pred1):
                    write_port = array & self
                    write_port[idx1] <= val1

                push_condition(pred0)
                self.fifo.push(val0)
                pop_condition()

                push_condition(pred1)
                self.fifo.push(val1)
                pop_condition()

        MultiWriter().build()

    cleanup_module = sys_builder.modules[0]

    module_metadata, interactions = collect_fifo_metadata(sys_builder)
    dumper = CIRCTDumper(module_metadata=module_metadata, interactions=interactions)
    dumper.sys = sys_builder
    dumper.array_metadata.collect(sys_builder)
    dumper.visit_module(cleanup_module)

    module_entry = module_metadata[cleanup_module]
    module_view = module_entry.interactions

    array_name = None
    array_port_suffix = ''
    if module_view.writes:
        array_ref = next(iter(module_view.writes))
        array_name = dumper.dump_rval(array_ref, False)
        array_meta = dumper.array_metadata.metadata_for(array_ref)
        if array_meta is not None:
            port_idx = array_meta.write_ports.get(cleanup_module, 0)
            array_port_suffix = f"_port{port_idx}"

    fifo_name = None
    fifo_module_prefix = None
    for fifo_port in module_view.fifo_ports:
        interactions_for_port = module_view.fifo_map[fifo_port]
        pushes = [entry for entry in interactions_for_port if isinstance(entry, FIFOPush)]
        pops = [entry for entry in interactions_for_port if isinstance(entry, FIFOPop)]
        assert all(isinstance(entry, FIFOPush) for entry in pushes)
        assert all(isinstance(entry, FIFOPop) for entry in pops)
        if pushes:
            fifo_name = dumper.dump_rval(fifo_port, False)
            fifo_module_prefix = namify(fifo_port.module.name)
            break

    context = {
        'array_name': array_name or 'array',
        'array_port_suffix': array_port_suffix or '_port0',
        'module_name': cleanup_module.name,
        'fifo_name': fifo_name or 'fifo',
        'fifo_module_prefix': fifo_module_prefix or namify(cleanup_module.name),
    }

    return [line.strip() for line in dumper.code if line.strip()], context


def _render_single_writer_cleanup_lines() -> Tuple[List[str], Dict[str, str]]:
    """Generate cleanup code for a single-writer module to validate passthrough cases."""
    sys_builder = SysBuilder("cleanup_mux_single")
    with sys_builder:

        class SingleWriter(Module):  # type: ignore[misc]

            def __init__(self):
                super().__init__(ports={
                    'pred': Port(UInt(1)),
                    'idx': Port(UInt(2)),
                    'val': Port(UInt(8)),
                    'fifo': Port(UInt(8)),
                })

            @module.combinational
            def build(self):
                pred = self.pred.pop()
                idx = self.idx.pop()
                val = self.val.pop()

                array = RegArray(UInt(8), 4, name="target_array_single")

                with Condition(pred):
                    write_port = array & self
                    write_port[idx] <= val

                push_condition(pred)
                self.fifo.push(val)
                pop_condition()

        SingleWriter().build()

    cleanup_module = sys_builder.modules[0]

    module_metadata, interactions = collect_fifo_metadata(sys_builder)
    dumper = CIRCTDumper(module_metadata=module_metadata, interactions=interactions)
    dumper.sys = sys_builder
    dumper.array_metadata.collect(sys_builder)
    dumper.visit_module(cleanup_module)

    module_entry = module_metadata[cleanup_module]
    module_view = module_entry.interactions

    array_ref = next(iter(module_view.writes))
    array_name = dumper.dump_rval(array_ref, False)
    array_meta = dumper.array_metadata.metadata_for(array_ref)
    port_idx = 0 if array_meta is None else array_meta.write_ports.get(cleanup_module, 0)
    fifo_port = None
    for candidate in module_view.fifo_ports:
        interactions_for_port = module_view.fifo_map[candidate]
        pushes = [entry for entry in interactions_for_port if isinstance(entry, FIFOPush)]
        pops = [entry for entry in interactions_for_port if isinstance(entry, FIFOPop)]
        assert all(isinstance(entry, FIFOPush) for entry in pushes)
        assert all(isinstance(entry, FIFOPop) for entry in pops)
        if pushes:
            fifo_port = candidate
            break

    assert fifo_port is not None

    fifo_name = dumper.dump_rval(fifo_port, False)
    fifo_module_prefix = namify(fifo_port.module.name)
    fifo_push_prefix = f"self.{fifo_module_prefix}_{fifo_port.name}"

    context = {
        'array_name': array_name,
        'array_port_suffix': f"_port{port_idx}",
        'module_name': cleanup_module.name,
        'fifo_name': fifo_name,
        'fifo_module_prefix': fifo_module_prefix,
        'fifo_push_prefix': fifo_push_prefix,
    }

    return [line.strip() for line in dumper.code if line.strip()], context


def _extract_assignments(lines: Iterable[str], targets: Iterable[str]) -> Dict[str, str]:
    """Return the assignment lines for *targets* from the rendered code."""
    want = set(targets)
    results: Dict[str, str] = {}
    for line in lines:
        for target in tuple(want):
            if line.startswith(f"{target} ="):
                results[target] = line
                want.remove(target)
        if not want:
            break
    return results


def test_array_write_mux_matches_reference_rendering():
    """Ensure array write mux chains keep their original structure."""
    lines, context = _render_cleanup_lines()
    base = f"self.{context['array_name']}_w{context['array_port_suffix']}"
    expected = {
        base: (
            f"{base} = executed_wire & "
            "(reduce(or_, [self.pred0.as_bits(), self.pred1.as_bits()]))"
        ),
        f"{base.replace('_w', '_wdata')}": (
            f"{base.replace('_w', '_wdata')} = Mux("
            "self.pred1.as_bits(), "
            "Mux(self.pred0.as_bits(), UInt(8)(0), self.val0), "
            "self.val1)"
        ),
        f"{base.replace('_w', '_widx')}": (
            f"{base.replace('_w', '_widx')} = Mux("
            "self.pred1.as_bits(), "
            "Mux(self.pred0.as_bits(), UInt(2)(0), self.idx0), "
            "self.idx1).as_bits()"
        ),
    }
    assignments = _extract_assignments(lines, expected.keys())
    assert assignments == expected


def test_fifo_push_mux_matches_reference_rendering():
    """Ensure FIFO push mux chains keep their original structure."""
    lines, context = _render_cleanup_lines()
    fifo_prefix = f"self.{context['fifo_module_prefix']}_{context['fifo_name']}"
    ready_signal = (
        f"self.fifo_{context['fifo_module_prefix']}_{context['fifo_name']}_push_ready"
    )
    expected = {
        f"{fifo_prefix}_push_valid": (
            f"{fifo_prefix}_push_valid = executed_wire & "
            "(reduce(or_, [(self.pred0), (self.pred1)], Bits(1)(0))) & "
            f"{ready_signal}"
        ),
        f"{fifo_prefix}_push_data": (
            f"{fifo_prefix}_push_data = Mux("
            "self.pred1, "
            "Mux(self.pred0, UInt(8)(0), self.val0), "
            "self.val1)"
        ),
    }
    assignments = _extract_assignments(lines, expected.keys())
    assert assignments == expected


def test_array_write_single_entry_passthrough():
    """Single writer uses direct assignments instead of redundant muxing."""
    lines, context = _render_single_writer_cleanup_lines()
    base = f"self.{context['array_name']}_w{context['array_port_suffix']}"
    expected = {
        base: (
            f"{base} = executed_wire & "
            "(self.pred.as_bits())"
        ),
        f"{base.replace('_w', '_wdata')}": (
            f"{base.replace('_w', '_wdata')} = self.val"
        ),
        f"{base.replace('_w', '_widx')}": (
            f"{base.replace('_w', '_widx')} = self.idx.as_bits()"
        ),
    }
    assignments = _extract_assignments(lines, expected.keys())
    assert assignments == expected


def test_fifo_push_single_entry_passthrough():
    """Single FIFO push should re-use the predicate and value verbatim."""
    lines, context = _render_single_writer_cleanup_lines()
    fifo_prefix = context['fifo_push_prefix']
    ready_signal = (
        f"self.fifo_{context['fifo_module_prefix']}_{context['fifo_name']}_push_ready"
    )
    expected = {
        f"{fifo_prefix}_push_valid": (
            f"{fifo_prefix}_push_valid = executed_wire & "
            "(reduce(or_, [(self.pred)], Bits(1)(0))) & "
            f"{ready_signal}"
        ),
        f"{fifo_prefix}_push_data": (
            f"{fifo_prefix}_push_data = self.val"
        ),
    }
    assignments = _extract_assignments(lines, expected.keys())
    assert assignments == expected


def test_format_reduction_expr_supports_and_operator_with_defaults():
    """Generalised helper emits AND reductions and surfaces defaults."""
    assert (
        _format_reduction_expr([], default_literal="Bits(1)(1)", op="and_")
        == "Bits(1)(1)"
    )
    assert (
        _format_reduction_expr(["lhs"], default_literal="Bits(1)(1)", op="and_")
        == "reduce(and_, [lhs], Bits(1)(1))"
    )
    assert (
        _format_reduction_expr(["lhs", "rhs"], default_literal="Bits(1)(1)", op="and_")
        == "reduce(and_, [lhs, rhs], Bits(1)(1))"
    )


def test_emit_predicate_mux_chain_preserves_custom_reduce():
    """Aggregated predicate from helper should be forwarded unchanged."""
    entries = ["v0", "v1"]

    def render_predicate(entry):
        return f"{entry}_pred"

    mux_expr, predicate_expr = _emit_predicate_mux_chain(
        entries,
        render_predicate=render_predicate,
        render_value=lambda entry: entry,
        default_value="DEFAULT",
        aggregate_predicates=lambda preds: _format_reduction_expr(
            preds,
            default_literal="Bits(1)(1)",
            op="and_",
        ),
    )

    assert predicate_expr == "reduce(and_, [v0_pred, v1_pred], Bits(1)(1))"
    assert mux_expr == "Mux(v1_pred, Mux(v0_pred, DEFAULT, v0), v1)"


def test_emit_predicate_mux_chain_empty_sequence_defaults():
    """Helper should surface the caller's defaults when no entries are provided."""
    default_value = "UInt(8)(0)"

    def aggregate(predicates):
        return _format_reduction_expr(predicates, default_literal="Bits(1)(0)")

    mux_expr, predicate_expr = _emit_predicate_mux_chain(
        [],
        render_predicate=lambda entry: entry,
        render_value=lambda entry: entry,
        default_value=default_value,
        aggregate_predicates=aggregate,
    )

    assert mux_expr == default_value
    assert predicate_expr == "Bits(1)(0)"
