from . import visitor
from .builder import SysBuilder
from .data import Array
from . import dtype


def cg_dtype(ty: dtype.DType):
    prefix = 'eir::ir::DataType'
    if isinstance(ty, dtype.Int):
        return f'{prefix}::int_ty({ty.bits})'
    elif isinstance(ty, dtype.UInt):
        return f'{prefix}::uint_ty({ty.bits})'
    elif isinstance(ty, dtype.Bits):
        return f'{prefix}::bits_ty({ty.bits})'


class CodeGen(visitor.Visitor):

    def visit_system(self, node: SysBuilder):
        self.code.append('use eir::builder::SysBuilder;')
        self.code.append('fn main() {')
        self.code.append('  let mut sys = SysBuilder::new(\"%s\");' % node.name)
        for elem in node.arrays:
            self.visit_array(elem)
        for elem in node.modules:
            self.visit_module(elem)
        self.code.append('}\n')

    def visit_module(self, node):
        self.code.append('  // %s' % node.name)
        self.code.append('  let module = sys.create_module("%s", vec![]);' % node.name.lower())

    def visit_array(self, node: Array):
        name = node.name
        ty = cg_dtype(node.scalar_ty)
        self.code.append('  // %s' % repr(node))
        self.code.append('  // TODO: Support initial values')
        self.code.append('  // TODO: Support array attributes')
        self.code.append('  let %s = sys.create_array(%s, \"%s\", %d, None, vec![]);' % (name, ty, name, node.size))

    def __init__(self):
        self.code = []

def codegen(sys: SysBuilder):
    cg = CodeGen()
    cg.visit_system(sys)
    return '\n'.join(cg.code)

