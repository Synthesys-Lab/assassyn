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


def codegen_async_call(node: AsyncCall, module_ctx):
    """Generate code for async call operations."""
    bind = node.bind
    event_q = f"{namify(bind.callee.name)}_event"
    return f"""{{
              let stamp = sim.stamp - sim.stamp % 100 + 100;
              sim.{event_q}.push_back(stamp)
            }}"""


def codegen_fifo_pop(node: FIFOPop, module_ctx):
    """Generate code for FIFO pop operations."""
    fifo = node.fifo
    fifo_id = fifo_name(fifo)
    module_name = module_ctx.name
    loc_info = str(getattr(node, "loc", "<unknown location>")).replace('"', '\\"')

    return f"""{{
              let stamp = sim.stamp - sim.stamp % 100 + 50;
              sim.{fifo_id}.pop.push(FIFOPop::new(stamp, "{module_name}"));
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

    return f"""{{
              let stamp = sim.stamp;
              sim.{fifo_id}.push.push(
                FIFOPush::new(stamp + 50, {value}.clone(), "{module_name}"));
            }}"""


def codegen_bind(node: Bind, module_ctx):
    """Generate code for bind operations."""
    return "()"
