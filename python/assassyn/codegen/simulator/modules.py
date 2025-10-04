"""Module elaboration for simulator code generation."""

# pylint: disable=duplicate-code

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

class ElaborateModule(Visitor):
    """Visitor for elaborating modules with multi-port write support."""

    def __init__(self, sys):
        """Initialize the module elaborator."""
        super().__init__()
        self.sys = sys
        self.indent = 0
        self.module_name = ""
        self.module_ctx = None
        self.modules_for_callback = {}

    def visit_module_for_callback(self, node: Module):
        """Visit a module to collect module names for callback."""
        self.module_name = node.name
        self.module_ctx = node
        self.visit_block(node.body)
        return self.modules_for_callback

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
        code = codegen_expr(node, self.module_ctx, self.sys, self.module_name,
                           self.modules_for_callback)

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
