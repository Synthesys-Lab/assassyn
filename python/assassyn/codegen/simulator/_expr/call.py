"""Call-related code generation helpers for simulator.

This module contains helper functions to generate simulator code for call operations,
including async calls, FIFO operations, and bindings.
"""

# pylint: disable=unused-argument

from ....ir.expr import AsyncCall, FIFOPop, FIFOPush
from ....ir.expr.call import Bind
from ....utils import namify
from ..utils import fifo_name
from ..node_dumper import dump_rval_ref


def _coverage_fifo_id(fifo):
    return f"fifo:{_coverage_module_name(fifo.module)}.{fifo.name}"


def _coverage_module_name(module):
    return module.__class__.__name__


def _coverage_depth(push: FIFOPush):
    depth = push.fifo_depth
    if isinstance(depth, int) and depth >= 0:
        return 1 << depth
    return 0


def codegen_async_call(node: AsyncCall, module_ctx):
    """Generate code for async call operations."""
    bind = node.bind
    event_q = f"{namify(bind.callee.name)}_event"
    caller_name = _coverage_module_name(module_ctx)
    callee_name = _coverage_module_name(bind.callee)
    coverage_id = f"async:{caller_name}->{callee_name}:{namify(node.as_operand())}"
    return f"""{{
              let stamp = sim.stamp - sim.stamp % 100 + 100;
              sim.{event_q}.push_back(stamp);
              if let Some(coverage) = sim.coverage.as_mut() {{
                coverage.record_async_call(
                  "{coverage_id}", "{caller_name}", "{callee_name}", sim.stamp / 100);
              }}
            }}"""


def codegen_fifo_pop(node: FIFOPop, module_ctx):
    """Generate code for FIFO pop operations."""
    fifo = node.fifo
    fifo_id = fifo_name(fifo)
    module_name = module_ctx.name
    coverage_id = _coverage_fifo_id(fifo)
    coverage_module = _coverage_module_name(fifo.module)
    loc_info = str(getattr(node, "loc", "<unknown location>")).replace('"', '\\"')

    return f"""{{
              let stamp = sim.stamp - sim.stamp % 100 + 50;
              sim.{fifo_id}.pop.push(FIFOPop::new(stamp, "{module_name}"));
              if let Some(coverage) = sim.coverage.as_mut() {{
                coverage.record_fifo_pop(
                  "{coverage_id}", "{coverage_module}", "{fifo.name}", sim.stamp / 100);
              }}
              match sim.{fifo_id}.payload.front() {{
                Some(value) => value.clone(),
                None => panic!("{loc_info} is trying to pop an empty FIFO"),
              }}
            }}"""


def codegen_fifo_push(node: FIFOPush, module_ctx):
    """Generate code for FIFO push operations."""
    fifo = node.fifo
    fifo_id = fifo_name(fifo)
    value = dump_rval_ref(module_ctx, node.val)
    module_name = module_ctx.name
    coverage_id = _coverage_fifo_id(fifo)
    coverage_module = _coverage_module_name(fifo.module)
    configured_depth = _coverage_depth(node)

    return f"""{{
              let stamp = sim.stamp;
              sim.{fifo_id}.push.push(
                FIFOPush::new(stamp + 50, {value}.clone(), "{module_name}"));
              if let Some(coverage) = sim.coverage.as_mut() {{
                coverage.record_fifo_push(
                  "{coverage_id}", "{coverage_module}", "{fifo.name}",
                  sim.stamp / 100, {configured_depth});
              }}
            }}"""


def codegen_bind(node: Bind, module_ctx):
    """Generate code for bind operations."""
    return "()"
