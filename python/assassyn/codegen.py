from . import visitor
from . import dtype
from . import expr
from .builder import SysBuilder
from .data import Array
from .module import Module
from .block import Block
from .expr import Expr


def generate_dtype(ty: dtype.DType):
    prefix = 'eir::ir::DataType'
    if isinstance(ty, dtype.Int):
        return f'{prefix}::int_ty({ty.bits})'
    elif isinstance(ty, dtype.UInt):
        return f'{prefix}::uint_ty({ty.bits})'
    elif isinstance(ty, dtype.Bits):
        return f'{prefix}::bits_ty({ty.bits})'

class CodeGen(visitor.Visitor):

    def visit_system(self, node: SysBuilder):
        self.header.append('use eir::{builder::SysBuilder, created_here};')
        self.code.append('fn main() {')
        self.code.append('  let mut sys = SysBuilder::new(\"%s\");\n' % node.name)
        self.code.append('  // TODO: Support initial values')
        self.code.append('  // TODO: Support array attributes')
        for elem in node.arrays:
            self.visit_array(elem)
        for elem in node.modules:
            self.visit_module(elem)
        self.code.append('''
  let config = eir::backend::common::Config{
     base_dir: (env!("CARGO_MANIFEST_DIR").to_string() + "/simulator").into(),
    ..Default::default()
  };''')
        self.code.append('  eir::backend::simulator::elaborate(&sys, &config).unwrap();')
        self.code.append('}\n')

    def visit_module(self, node: Module):
        self.code.append('  // %s' % node.name)
        self.code.append('  let module = sys.create_module("%s", vec![]);' % node.name.lower())
        self.code.append('  sys.set_current_module(module);')
        self.visit_block(node.body)

    def visit_block(self, node: Block):
        if node.kind == Block.MODULE_ROOT:
            self.code.append('  // module root block')
        for elem in node.body:
            self.visit_expr(elem)


    def generate_rval(self, node):
        if isinstance(node, dtype.Const):
            self.code.append('  // %s' % repr(node))
            ty = generate_dtype(node.dtype)
            self.code.append(f'  let imm_{id(node)} = sys.get_const_int({ty}, {node.value});')
            return f'imm_{id(node)}'
        else:
            return node.as_operand()

    def visit_expr(self, node: Expr):
        self.code.append('  // %s' % repr(node))
        ib_method = expr.opcode_to_ib(node.opcode)
        if expr.is_binary(node.opcode):
            lhs = self.generate_rval(node.lhs)
            rhs = self.generate_rval(node.rhs)
            res = f'  sys.{ib_method}(created_here!(), {lhs}, {rhs});'
        elif expr.is_unary(node.opcode):
            x = self.generate_rval(node.x)
            res = f'  sys.{ib_method}(created_here!(), {x});'
        elif expr.is_fifo_related(node.opcode) and isinstance(node, expr.FIFOField):
            fifo = self.generate_rval(node.fifo)
            res = f'  sys.{ib_method}({fifo});'
        elif node.opcode == expr.SideEffect.FIFO_PUSH:
            fifo = node.generate_rval(node.args[0])
            val = node.generate_rval(node.args[1])
            res = f'  sys.{ib_method}({fifo}, {val});'
        elif node.opcode == expr.SideEffect.FIFO_POP:
            fifo = node.generate_rval(node.args[0])
            res = f'  sys.{ib_method}({fifo});'
        elif node.opcode == expr.SideEffect.LOG:
            fmt = '"' + node.args[0] + '"'
            self.code.append('  let fmt = sys.get_str_literal(%s.into());' % fmt)
            args = ', '.join(self.generate_rval(i) for i in node.args[1:])
            res = f'  sys.{ib_method}(fmt, vec![{args}]);'
        elif node.opcode == expr.BinaryOp.ARRAY_READ:
            array = node.lhs.name
            idx = self.generate_rval(node.rhs)
            res = f'  sys.{ib_method}(created_here!(), {array}, {idx});'
        elif node.opcode == expr.SideEffect.ARRAY_WRITE:
            array = node.args[0].name
            idx = self.generate_rval(node.args[1])
            val = self.generate_rval(node.args[2])
            res = f'  sys.{ib_method}(created_here!(), {array}, {idx}, {val});'
        else:
            res = f'  // TODO ^'

        if expr.is_valued(node.opcode):
            res = f'  let {node.as_operand()} = {res}'

        self.code.append(res)


    def visit_array(self, node: Array):
        name = node.name
        ty = generate_dtype(node.scalar_ty)
        self.code.append('  // %s' % repr(node))
        self.code.append('  let %s = sys.create_array(%s, \"%s\", %d, None, vec![]);' % (name, ty, name, node.size))

    def __init__(self):
        self.code = []
        self.header = []

    def get_source(self):
        return '\n'.join(self.header) + '\n' + '\n'.join(self.code)

def codegen(sys: SysBuilder):
    cg = CodeGen()
    cg.visit_system(sys)
    return cg.get_source()

