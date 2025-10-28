"""Module elaboration for simulator code generation."""

from __future__ import annotations

import typing

from ...ir.visitor import Visitor
from ...ir.dtype import RecordValue
from ...ir.expr import Expr
from ...ir.expr.intrinsic import Intrinsic as IRIntrinsic
from ...ir.memory.dram import DRAM
from ...utils import namify
from .node_dumper import dump_rval_ref
from ...analysis import expr_externally_used
from ...ir.module.external import ExternalSV
from .external import has_module_body

if typing.TYPE_CHECKING:
    from ...ir.module import Module
    from ...builder import SysBuilder


class ElaborateModule(Visitor):  # pylint: disable=too-many-instance-attributes
    """Visitor for elaborating modules with ExternalSV support."""

    def __init__(self, sys):
        super().__init__()
        self.sys = sys
        self.indent = 0
        self.module_name = ""
        self.module_ctx = None

    def visit_module(self, node: Module):
        """Visit a module and generate its implementation."""
        self.module_name = node.name
        self.module_ctx = node

        if isinstance(node, ExternalSV) and not has_module_body(node):
            return self.visit_external_module(node)

        result = [f"\n// Elaborating module {self.module_name}"]
        result.append(f"pub fn {namify(self.module_name)}(sim: &mut Simulator) -> bool {{")

        self.indent += 2
        body = self._emit_body(node.body or [])
        result.append(body)

        self.indent -= 2
        result.append(" true }")

        return "\n".join(result)

    def visit_expr(self, node: Expr):  # pylint: disable=too-many-locals
        """Visit an expression and generate its implementation."""
        from ._expr import codegen_expr  # pylint: disable=import-outside-toplevel


        id_and_exposure = None
        if node.is_valued():
            need_exposure = expr_externally_used(node, True)
            id_expr = namify(node.as_operand())
            id_and_exposure = (id_expr, need_exposure)

        # Handle PUSH/POP_CONDITION at the visitor level to form if blocks
        code = None
        if isinstance(node, IRIntrinsic) and node.opcode in (
            IRIntrinsic.PUSH_CONDITION,
            IRIntrinsic.POP_CONDITION,
        ):
            indent_str = " " * self.indent
            if node.opcode == IRIntrinsic.PUSH_CONDITION:
                cond_val = dump_rval_ref(self.module_ctx, node.args[0])
                result = f"{indent_str}if {cond_val} {{\n"
                # Increase indentation for the body inside this condition
                self.indent += 2
                return result
            # POP_CONDITION closes the current scope
            self.indent = max(0, self.indent - 2)
            return f"{' ' * self.indent}}}\n"

        code = codegen_expr(node, self.module_ctx)

        indent_str = " " * self.indent
        result = ""

        # Add location comment if available
        if hasattr(node, 'loc') and node.loc:
            result += f"{indent_str}// @{node.loc}\n"

        if id_and_exposure:
            id_expr, need_exposure = id_and_exposure
            if code:
                lines = [f"{indent_str}let {id_expr} = {{ {code} }};"]
                # Skip validity tracking for ExternalIntrinsic
                # pylint: disable=import-outside-toplevel
                from ...ir.expr.intrinsic import ExternalIntrinsic
                if need_exposure and not isinstance(node, ExternalIntrinsic):
                    lines.append(f"{indent_str}sim.{id_expr}_value = Some({id_expr}.clone());")
                result = "\n".join(lines) + "\n"
        else:
            if code:
                result += f"{indent_str}{code};\n"

        return result

    def _emit_body(self, body_nodes):
        result = []
        visited = set()

        for elem in body_nodes:
            elem_id = id(elem)
            if elem_id in visited:
                continue
            visited.add(elem_id)
            if isinstance(elem, Expr):
                result.append(self.visit_expr(elem))
            elif isinstance(elem, RecordValue):
                result.append(self.visit_expr(elem.value()))
            else:
                raise ValueError(f"Unexpected reference type: {type(elem).__name__}")

        return "".join(result)

    def visit_int_imm(self, int_imm):
        """Render integer immediates as Rust ``ValueCastTo`` expressions."""
        ty = dump_rval_ref(self.module_ctx, int_imm.dtype)
        value = int_imm.value
        return f"ValueCastTo::<{ty}>::cast(&{value})"

    def visit_external_module(self, node: ExternalSV):
        """Emit a stub implementation for an external module."""
        module_id = namify(node.name)
        return (
            f"\n// External module {node.name} is driven via FFI handles\n"
            f"pub fn {module_id}(sim: &mut Simulator) -> bool {{\n"
            "    let _ = sim;\n"
            "    true\n"
            " }\n"
        )


def dump_modules(sys: SysBuilder, modules_dir):
    """Generate individual module files in the modules/ directory."""
    modules_dir.mkdir(exist_ok=True)

    em = ElaborateModule(sys)

    mod_rs_path = modules_dir / "mod.rs"
    with open(mod_rs_path, 'w', encoding="utf-8") as mod_fd:
        mod_fd.write("""use sim_runtime::*;
use super::simulator::Simulator;
use std::collections::VecDeque;
use sim_runtime::num_bigint::{BigInt, BigUint};
use sim_runtime::libloading::{Library, Symbol};
use std::ffi::{CString, c_char, c_float, c_longlong, c_void};
use std::sync::Arc;

""")

        for module in sys.modules[:] + sys.downstreams[:]:
            module_name = namify(module.name)
            mod_fd.write(f"pub mod {module_name};\n")

            module_file_path = modules_dir / f"{module_name}.rs"
            with open(module_file_path, 'w', encoding="utf-8") as module_fd:
                module_fd.write("""use sim_runtime::*;
use sim_runtime::num_bigint::{BigInt, BigUint};
use crate::simulator::Simulator;
use std::ffi::c_void;

""")

                if isinstance(module, DRAM):
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
            sim.{module_name}_response.data = vec![
                (req.addr as u8) & 0xFF,
                ((req.addr >> 8) as u8) & 0xFF,
                ((req.addr >> 16) as u8) & 0xFF,
                ((req.addr >> 24) as u8) & 0xFF
            ];
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

                module_code = em.visit_module(module)
                module_fd.write(module_code)

    return True
