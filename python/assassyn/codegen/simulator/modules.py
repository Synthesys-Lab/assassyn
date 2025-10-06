"""Module elaboration for simulator code generation."""

from __future__ import annotations

import typing

from ...ir.visitor import Visitor
from ...ir.block import Block, CondBlock, CycledBlock
from ...ir.dtype import RecordValue
from ...ir.expr import Expr
from ...utils import namify
from .node_dumper import dump_rval_ref
from ...analysis import expr_externally_used
from .callback_collector import collect_callback_intrinsics, CallbackMetadata

if typing.TYPE_CHECKING:
    from ...ir.module import Module
    from ...builder import SysBuilder

class ElaborateModule(Visitor):
    """Visitor for elaborating modules with multi-port write support."""

    def __init__(self, sys, callback_metadata: CallbackMetadata | None = None):
        """Initialize the module elaborator."""
        super().__init__()
        self.sys = sys
        self.indent = 0
        self.module_name = ""
        self.module_ctx = None
        self.callback_metadata = callback_metadata

    def visit_module(self, node: Module):
        """Visit a module and generate its implementation."""
        self.module_name = node.name
        self.module_ctx = node

        # Create function header
        result = [f"\n// Elaborating module {self.module_name}"]
        result.append(f"pub fn {namify(self.module_name)}(sim: &mut Simulator) -> bool {{")

        # Increase indentation for function body
        self.indent += 2

        # Visit the module body
        body = self.visit_block(node.body)
        result.append(body)

        # Decrease indentation and add function closing
        self.indent -= 2
        result.append(" true }")

        return "\n".join(result)

    def visit_expr(self, node: Expr):
        """Visit an expression and generate its implementation."""
        # pylint: disable=import-outside-toplevel
        from ._expr import codegen_expr

        id_and_exposure = None
        if node.is_valued():
            need_exposure = False
            need_exposure = expr_externally_used(node, True)
            id_expr = namify(node.as_operand())
            id_and_exposure = (id_expr, need_exposure)

        # Generate code using the codegen_expr helper
        kwargs = {}
        if (self.callback_metadata and self.callback_metadata.memory and
                self.callback_metadata.store):
            kwargs['modules_for_callback'] = {
                'memory': self.callback_metadata.memory,
                'store': self.callback_metadata.store
            }
        code = codegen_expr(node, self.module_ctx, self.sys, **kwargs)

        # Format the result with proper indentation and variable assignment
        indent_str = " " * self.indent
        result = ""

        if id_and_exposure:
            id_expr, need_exposure = id_and_exposure
            valid_update = ""
            if need_exposure:
                valid_update = f"sim.{id_expr}_value = Some({id_expr}.clone());"

            if code:
                result = f"{indent_str}let {id_expr} = {{ {code} }}; {valid_update}\n"
            else:
                result = ""
        else:
            if code:
                result = f"{indent_str}{code};\n"

        return result

    def visit_int_imm(self, int_imm):
        """Visit an integer immediate value."""
        ty = dump_rval_ref(self.module_ctx, self.sys, int_imm.dtype)
        value = int_imm.value
        return f"ValueCastTo::<{ty}>::cast(&{value})"

    def visit_block(self, node: Block):
        """Visit a block and generate its implementation."""
        result = []
        visited = set()

        # Save current indentation
        restore_indent = self.indent

        if isinstance(node, CondBlock):
            cond = dump_rval_ref(self.module_ctx, self.sys, node.cond)
            result.append(f"if {cond} {{\n")
            self.indent += 2
        elif isinstance(node, CycledBlock):
            result.append(f"if sim.stamp / 100 == {node.cycle} {{\n")
            self.indent += 2

        # Visit each element in the block
        for elem in node.iter():
            elem_id = id(elem)
            if elem_id in visited:
                continue
            visited.add(elem_id)
            if isinstance(elem, Expr):
                result.append(self.visit_expr(elem))
            elif isinstance(elem, Block):
                result.append(self.visit_block(elem))
            elif isinstance(elem, RecordValue):
                result.append(self.visit_expr(elem.value()))
            else:
                raise ValueError(f"Unexpected reference type: {type(elem).__name__}")

        # Restore indentation and close scope if needed
        if restore_indent != self.indent:
            self.indent -= 2
            result.append(f"{' ' * self.indent}}}\n")

        return "".join(result)


def dump_modules(sys: SysBuilder, modules_dir):
    """Generate individual module files in the modules/ directory.

    This creates separate files for each module and a mod.rs file for declarations.
    """
    # Create modules directory
    modules_dir.mkdir(exist_ok=True)

    # Generate each module's implementation
    callback_metadata = collect_callback_intrinsics(sys)
    em = ElaborateModule(sys, callback_metadata)

    # Create mod.rs file with imports and callback function
    mod_rs_path = modules_dir / "mod.rs"
    with open(mod_rs_path, 'w', encoding="utf-8") as mod_fd:
        # Add imports
        mod_fd.write("""use sim_runtime::*;
use super::simulator::Simulator;
use std::collections::VecDeque;
use sim_runtime::num_bigint::{BigInt, BigUint};
use sim_runtime::libloading::{Library, Symbol};
use std::ffi::{CString, c_char, c_float, c_longlong, c_void};
use std::sync::Arc;

""")

        # Add callback function if needed
        if (
            callback_metadata.memory
            and callback_metadata.store
            and callback_metadata.mem_user_rdata
        ):
            mod_fd.write(f"""extern "C" fn rust_callback(req: *mut Request, ctx: *mut c_void) {{
    unsafe {{
        let req = &*req;
        let sim: &mut Simulator = &mut *(ctx as *mut Simulator);
        let cycles = (req.depart - req.arrive) as usize;
        let stamp = sim.request_stamp_map_table
            .remove(&req.addr)
            .unwrap_or_else(|| sim.stamp);
        sim.{callback_metadata.mem_user_rdata}.push.push(FIFOPush::new(
            stamp + 100 * cycles,
            sim.{callback_metadata.store}.payload[req.addr as usize].clone().try_into().unwrap(),
            "{callback_metadata.memory}",
        ));
    }}
}}

""")

        # Generate module declarations and individual files
        for module in sys.modules[:] + sys.downstreams[:]:
            module_name = namify(module.name)

            # Add module declaration to mod.rs
            mod_fd.write(f"pub mod {module_name};\n")

            # Create individual module file
            module_file_path = modules_dir / f"{module_name}.rs"
            with open(module_file_path, 'w', encoding="utf-8") as module_fd:
                # Add imports for the individual module
                module_fd.write("""use sim_runtime::*;
use sim_runtime::num_bigint::{BigInt, BigUint};
use crate::simulator::Simulator;

""")

                # Generate module implementation
                module_code = em.visit_module(module)
                module_fd.write(module_code)

    return True
