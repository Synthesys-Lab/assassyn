from . import visitor
from . import dtype
from . import expr
from . import module
from .builder import SysBuilder
from .data import Array
from .module import Module, Port
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

def generate_port(port: Port):
    ty = f'{generate_dtype(port.dtype)}'
    return f'eir::builder::PortInfo::new("{port.name}", {ty})'

class CodeGen(visitor.Visitor):

    def visit_system(self, node: SysBuilder):
        self.header.append('use eir::{builder::SysBuilder, created_here};')
        self.header.append('use eir::ir::node::IsElement;')
        self.code.append('fn main() {')
        self.code.append('  let mut sys = SysBuilder::new(\"%s\");\n' % node.name)
        self.code.append('  // TODO: Support initial values')
        self.code.append('  // TODO: Support array attributes')
        for elem in node.arrays:
            self.visit_array(elem)
        for elem in node.modules:
            name = elem.name.lower()
            ports = ', '.join(generate_port(p) for p in elem.ports)
            self.code.append(f'  let {name} = sys.create_module("{name}", vec![{ports}]);')
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
        self.code.append('  // Fill in the body of %s' % node.name)
        self.code.append('  sys.set_current_module(%s);' % node.name.lower())
        self.visit_block(node.body)

    def visit_block(self, node: Block):
        if node.kind == Block.MODULE_ROOT:
            self.code.append('  // module root block')
        for elem in node.body:
            self.visit_expr(elem)

    def generate_rval(self, node):
        if isinstance(node, dtype.Const):
            ty = generate_dtype(node.dtype)
            self.code.append(f'  let imm_{id(node)} = sys.get_const_int({ty}, {node.value}); // {node}')
            return f'imm_{id(node)}'
        if isinstance(node, module.Port):
            module_name = node.module.as_operand().lower()
            port_name = f'{module_name}_{node.name}'
            self.code.append(f'''  // Get port {node.name}
  let {port_name} = {{
    let module = {module_name}.as_ref::<eir::ir::Module>(&sys).unwrap();
    module.get_port_by_name("{node.name}").unwrap().upcast()
  }};''')
            return port_name 
        else:
            return node.as_operand()

    def visit_expr(self, node: Expr):
        self.code.append('  // %s' % repr(node))
        ib_method = expr.opcode_to_ib(node)
        if node.is_binary():
            lhs = self.generate_rval(node.lhs)
            rhs = self.generate_rval(node.rhs)
            res = f'sys.{ib_method}(created_here!(), {lhs}, {rhs});'
        elif node.is_unary():
            x = self.generate_rval(node.x)
            res = f'sys.{ib_method}(created_here!(), {x});'
        elif isinstance(node, expr.FIFOField):
            fifo = self.generate_rval(node.fifo)
            res = f'sys.{ib_method}({fifo});'
        elif isinstance(node, expr.FIFOPush):
            fifo = self.generate_rval(node.fifo)
            val = self.generate_rval(node.val)
            res = f'sys.{ib_method}({fifo}, {val});'
        elif isinstance(node, expr.FIFOPop):
            fifo = self.generate_rval(node.fifo)
            res = f'sys.{ib_method}({fifo});'
        elif isinstance(node, expr.Log):
            fmt = '"' + node.args[0] + '"'
            self.code.append('  let fmt = sys.get_str_literal(%s.into());' % fmt)
            args = ', '.join(self.generate_rval(i) for i in node.args[1:])
            res = f'sys.{ib_method}(fmt, vec![{args}]);'
        elif isinstance(node, expr.ArrayRead):
            arr = node.arr.name
            idx = self.generate_rval(node.idx)
            res = f'sys.{ib_method}(created_here!(), {arr}, {idx});'
        elif isinstance(node, expr.ArrayWrite):
            arr = node.arr.name
            idx = self.generate_rval(node.idx)
            val = self.generate_rval(node.val)
            res = f'sys.{ib_method}(created_here!(), {arr}, {idx}, {val});'
        else:
            length = len(repr(node))
            res = f'  // ^{"~" * length}: Support the instruction above'

        if 'Support the instruction above' in res:
            pass
        elif node.is_valued():
            res = f'  let {node.as_operand()} = {res}'
        else:
            res = f'  {res}'

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

