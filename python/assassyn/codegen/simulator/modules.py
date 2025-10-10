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

if typing.TYPE_CHECKING:
    from ...ir.module import Module
    from ...builder import SysBuilder

class ElaborateModule(Visitor):
    """Visitor for elaborating modules with multi-port write support."""

    def __init__(self, sys):
        """Initialize the module elaborator."""
        super().__init__()
        self.sys = sys
        self.indent = 0
        self.module_name = ""
        self.module_ctx = None

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
            need_exposure = expr_externally_used(  # noqa: E501
                node, True)  # noqa: E501
            id_expr = namify(node.as_operand())
            id_and_exposure = (id_expr, need_exposure)

        # Generate code using the codegen_expr helper
        kwargs = {}
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
            # Handle condition generation properly for intrinsics
            # pylint: disable=import-outside-toplevel
            from ._expr import codegen_expr
            cond_code = codegen_expr(node.cond, self.module_ctx, self.sys)
            if cond_code:
                cond = cond_code
            else:
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
    em = ElaborateModule(sys)

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
use std::ffi::c_void;

""")

                # Add inline callback function for DRAM modules
                if module_name.startswith('DRAM_'):
                    module_fd.write(f"""pub extern "C" fn callback_of_{module_name}(
    req: *mut Request, ctx: *mut c_void) {{
    unsafe {{
        let req = &*req;
        let sim: &mut Simulator = &mut *(ctx as *mut Simulator);
        let cycles = (req.depart - req.arrive) as usize;
        let stamp = sim.request_stamp_map_table
            .remove(&req.addr)
            .unwrap_or_else(|| sim.stamp);
        
        if req.type_id == 0 {{
            // Read response
            sim.{module_name}_response.valid = true;
            sim.{module_name}_response.addr = req.addr as usize;
            sim.{module_name}_response.data = vec![(req.addr as u8) & 0xFF, ((req.addr >> 8) as u8) & 0xFF, ((req.addr >> 16) as u8) & 0xFF, ((req.addr >> 24) as u8) & 0xFF];
            sim.{module_name}_response.read_succ = true;
            sim.{module_name}_response.is_write = false;
        }} else {{
            // Write response
            sim.{module_name}_response.valid = true;
            sim.{module_name}_response.addr = req.addr as usize;
            sim.{module_name}_response.write_succ = true;
            sim.{module_name}_response.is_write = true;
        }}
    }}
}}

""")

                # Generate module implementation
                module_code = em.visit_module(module)
                module_fd.write(module_code)

    return True
