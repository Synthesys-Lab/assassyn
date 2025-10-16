"""Module elaboration for simulator code generation."""

from __future__ import annotations

import typing

from ...ir.visitor import Visitor
from ...ir.block import Block, CondBlock, CycledBlock
from ...ir.dtype import RecordValue
from ...ir.expr import Expr, WireAssign, WireRead
from ...utils import namify
from .node_dumper import dump_rval_ref
from ...analysis import expr_externally_used
from .utils import dtype_to_rust_type
from ...ir.module.external import ExternalSV
from .external import (
    codegen_external_wire_assign,
    codegen_external_wire_read,
    collect_external_value_assignments,
    external_handle_field,
    has_module_body,
    lookup_external_port,
)

if typing.TYPE_CHECKING:
    from ...ir.module import Module
    from ...builder import SysBuilder


class ElaborateModule(Visitor):  # pylint: disable=too-many-instance-attributes
    """Visitor for elaborating modules with ExternalSV support."""

    def __init__(self, sys, external_specs: dict[str, typing.Any] | None = None):
        super().__init__()
        self.indent = 0
        self.module_name = ""
        self.module_ctx = None
        self.external_specs = external_specs or getattr(sys, "_external_ffi_specs", {})
        self.external_value_assignments = collect_external_value_assignments(sys)
        self.emitted_external_assignments = set()

    def visit_module(self, node: Module):
        """Visit a module and generate its implementation."""
        self.module_name = node.name
        self.module_ctx = node

        if isinstance(node, ExternalSV) and not has_module_body(node):
            return self.visit_external_module(node)

        result = [f"\n// Elaborating module {self.module_name}"]
        result.append(f"pub fn {namify(self.module_name)}(sim: &mut Simulator) -> bool {{")

        self.indent += 2
        body = self.visit_block(node.body)
        result.append(body)

        self.indent -= 2
        result.append(" true }")

        return "\n".join(result)

    def visit_expr(self, node: Expr):  # pylint: disable=too-many-locals
        """Visit an expression and generate its implementation."""
        from ._expr import codegen_expr  # pylint: disable=import-outside-toplevel

        if isinstance(node, WireAssign):
            value_code = dump_rval_ref(self.module_ctx, node.value)
            code = codegen_external_wire_assign(
                node,
                external_specs=self.external_specs,
                external_value_assignments=self.external_value_assignments,
                value_code=value_code,
            )
            if code:
                indent_str = " " * self.indent
                return f"{indent_str}{code}\n"
            return ""

        custom_code = None
        if isinstance(node, WireRead):
            custom_code = codegen_external_wire_read(
                node,
                external_specs=self.external_specs,
            )

        id_and_exposure = None
        if node.is_valued():
            need_exposure = expr_externally_used(node, True)
            id_expr = namify(node.as_operand())
            id_and_exposure = (id_expr, need_exposure)

        code = (
            custom_code
            if custom_code is not None
            else codegen_expr(
                node,
                self.module_ctx,
            )
        )

        indent_str = " " * self.indent
        result = ""

        # Add location comment if available
        if hasattr(node, 'loc') and node.loc:
            result += f"{indent_str}// @{node.loc}\n"

        if id_and_exposure:
            id_expr, need_exposure = id_and_exposure
            if code:
                lines = [f"{indent_str}let {id_expr} = {{ {code} }};"]
                if need_exposure:
                    lines.append(f"{indent_str}sim.{id_expr}_value = Some({id_expr}.clone());")
                key = (self.module_ctx, id_expr)
                if (
                    key in self.external_value_assignments
                    and key not in self.emitted_external_assignments
                ):
                    assignments = self.external_value_assignments[key]
                    for ext_module, wire in assignments:
                        handle_field = external_handle_field(ext_module.name)
                        setter_suffix = namify(wire.name)
                        port_spec = lookup_external_port(
                            self.external_specs, ext_module.name, wire.name, "input"
                        )
                        rust_ty = (
                            port_spec.rust_type
                            if port_spec is not None
                            else dtype_to_rust_type(wire.dtype)
                        )
                        lines.append(
                            f"{indent_str}sim.{handle_field}.set_{setter_suffix}("
                            f"ValueCastTo::<{rust_ty}>::cast(&{id_expr}));"
                        )
                    self.emitted_external_assignments.add(key)
                result = "\n".join(lines) + "\n"
        else:
            if code:
                result += f"{indent_str}{code};\n"

        return result

    def visit_int_imm(self, int_imm):
        """Render integer immediates as Rust ``ValueCastTo`` expressions."""
        ty = dump_rval_ref(self.module_ctx, int_imm.dtype)
        value = int_imm.value
        return f"ValueCastTo::<{ty}>::cast(&{value})"

    def visit_block(self, node: Block):
        result = []
        visited = set()

        restore_indent = self.indent

        if isinstance(node, CondBlock):
            if isinstance(node.cond, Expr):
                cond_code = self.visit_expr(node.cond)
                if cond_code:
                    result.append(cond_code)
            cond = dump_rval_ref(self.module_ctx, node.cond)
            result.append(f"if {cond} {{\n")
            self.indent += 2
        elif isinstance(node, CycledBlock):
            result.append(f"if sim.stamp / 100 == {node.cycle} {{\n")
            self.indent += 2

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

        if restore_indent != self.indent:
            self.indent -= 2
            result.append(f"{' ' * self.indent}}}\n")

        return "".join(result)

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

    external_specs = getattr(sys, "_external_ffi_specs", {})
    em = ElaborateModule(sys, external_specs)

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

                if module_name.startswith('DRAM'):
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
