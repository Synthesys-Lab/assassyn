'''The module to generate the assassyn IR builder for the given system'''

from . import visitor
from . import dtype
from . import expr
from . import module
from . import block
from . import const
from .builder import SysBuilder
from .array import Array
from .module import Module, Port
from .block import Block
from .expr import Expr

CG_OPCODE = {
    expr.BinaryOp.ADD: 'add',
    expr.BinaryOp.SUB: 'sub',
    expr.BinaryOp.MUL: 'mul',
    expr.BinaryOp.DIV: 'div',
    expr.BinaryOp.MOD: 'mod',

    # TODO(@were): Support non-integer comparisons
    expr.BinaryOp.ILT: 'ilt',
    expr.BinaryOp.IGT: 'igt',
    expr.BinaryOp.ILE: 'ile',
    expr.BinaryOp.IGE: 'ige',
    expr.BinaryOp.EQ:  'eq',

    expr.BinaryOp.BITWISE_OR:  'bitwise_or',
    expr.BinaryOp.BITWISE_AND: 'bitwise_and',
    expr.BinaryOp.BITWISE_XOR: 'bitwise_xor',

    expr.UnaryOp.FLIP: 'flip',
    expr.UnaryOp.NEG: 'neg',

    expr.Slice.SLICE: 'slice',
    expr.Concat.CONCAT: 'concat',

    expr.FIFOField.FIFO_PEEK: 'peek',
    expr.FIFOField.FIFO_VALID: 'valid',

    expr.FIFOPop.FIFO_POP: 'pop',
    expr.FIFOPush.FIFO_PUSH: 'push',

    expr.ArrayRead.ARRAY_READ: 'array_read',
    expr.ArrayWrite.ARRAY_WRITE: 'array_write',

    expr.AsyncCall.ASYNC_CALL: 'async_call',

    expr.Cast.BITCAST: 'bitcast',
    expr.Cast.ZEXT: 'zext',
    expr.Cast.SEXT: 'sext',

    expr.Select.SELECT: 'select',

    expr.Log.LOG: 'log',

    expr.intrinsic.Intrinsic.WAIT_UNTIL: 'wait_until',
}

CG_ARRAY_ATTR = {
    Array.FULLY_PARTITIONED: 'FullyPartitioned',
}

def opcode_to_ib(node: Expr):
    '''Convert the opcode to the corresponding IR builder method'''
    opcode = node.opcode
    if node.opcode == expr.Bind.BIND:
        return ''
    if node.is_fifo_related():
        return f'create_fifo_{CG_OPCODE[opcode]}'
    return f'create_{CG_OPCODE[opcode]}'

def generate_dtype(ty: dtype.DType):
    '''Generate AST data type representation into assassyn data type representation'''
    prefix = 'eir::ir::DataType'
    if isinstance(ty, dtype.Int):
        return f'{prefix}::int_ty({ty.bits})'
    if isinstance(ty, dtype.UInt):
        return f'{prefix}::uint_ty({ty.bits})'
    assert isinstance(ty, dtype.Bits), f'{ty} is given'
    return f'{prefix}::bits_ty({ty.bits})'

def generate_init_value(init_value, ty: dtype.DType):
    '''Generate the initial value for the given array'''
    if init_value is None:
        return ("\n", "None")

    str1 = f'let init_val = sys.get_const_int({ty}, {init_value});'
    str2 = 'Some(vec![init_val])'

    return (str1, str2)

def generate_port(port: Port):
    '''Generate the port information for the given port for module construction'''
    ty = f'{generate_dtype(port.dtype)}'
    return f'eir::builder::PortInfo::new("{port.name}", {ty})'

class EmitBinds(visitor.Visitor):
    '''Gather all the binds and emit them in advance'''

    def __init__(self, cg):
        self.cg = cg

    def visit_expr(self, node):
        if isinstance(node, expr.Bind):
            bind_var = self.cg.generate_rval(node)
            module_var = self.cg.generate_rval(node.callee)
            self.cg.code.append(f'  let {bind_var} = sys.get_init_bind({module_var});')


class CodeGen(visitor.Visitor):
    '''Generate the assassyn IR builder for the given system'''

    def emit_module_attrs(self, m: Module, var_id: str):
        '''Generate module attributes.'''
        module_mut = f'{var_id}.as_mut::<eir::ir::Module>(&mut sys).unwrap()'
        path = 'eir::ir::module::Attribute'
        if m.is_systolic:
            self.code.append(f'{module_mut}.add_attr({path}::Systolic);')
        if m.disable_arbiter_rewrite:
            self.code.append(f'{module_mut}.add_attr({path}::NoArbiter);')

    def emit_config(self):
        '''Emit the configuration fed to the generated simulator'''
        if self.kwargs:
            config = self.kwargs
            idle_threshold = config.get('idle_threshold', 100)
            sim_threshold = config.get('sim_threshold', 100)
            return f'idle_threshold: {idle_threshold}, sim_threshold: {sim_threshold},'
        return ''

    def generate_init_value(self, init_value, ty: str):
        '''Generate the initializer vector. NOTE: ty is already generated in an str!'''
        if init_value is None:
            return 'None'

        vec = []
        for i, j in enumerate(init_value):
            self.code.append(f'let init_{i} = sys.get_const_int({ty}, {j});')
            vec.append(f'init_{i}')

        self.code.append(f'let init = vec![{", ".join(vec)}];')

        return 'Some(init)'


    def visit_system(self, node: SysBuilder):
        self.header.append('use eir::{builder::SysBuilder, created_here};')
        self.header.append('use eir::ir::node::IsElement;')
        self.code.append('fn main() {')
        self.code.append(f'  let mut sys = SysBuilder::new(\"{node.name}\");')
        self.code.append('  let mut block_stack : Vec<eir::ir::node::BaseNode> = Vec::new();\n')
        self.code.append('  // TODO: Support initial values')
        self.code.append('  // TODO: Support array attributes')
        for elem in node.arrays:
            self.visit_array(elem)
        for elem in node.modules:
            lval = elem.as_operand()
            name = elem.synthesis_name().lower()
            ports = ', '.join(generate_port(p) for p in elem.ports)
            self.code.append(f'  let {lval} = sys.create_module("{name}", vec![{ports}]);')
        self.code.append('  // Gathered binds')
        for elem in node.modules:
            bind_emitter = EmitBinds(self)
            name = self.generate_rval(elem)
            self.code.append('  // Set the current module redundantly to emit related binds')
            self.code.append(f'  sys.set_current_module({name});')
            bind_emitter.visit_module(elem)
        for elem in node.modules:
            self.visit_module(elem)
        config = self.emit_config()
        self.code.append(f'''
            let config = eir::backend::common::Config{{
               base_dir: (env!("CARGO_MANIFEST_DIR").to_string() + "/simulator").into(),
               {config}
               ..Default::default()
            }};
        ''')
        self.code.append('  println!("{}", sys);')
        config = 'eir::xform::Config{ rewrite_wait_until: true }'
        self.code.append(f'  eir::xform::basic(&mut sys, &{config});')
        self.code.append('  eir::backend::simulator::elaborate(&sys, &config).unwrap();')
        self.code.append('}\n')

    def visit_module(self, node: Module):
        self.code.append(f'  // Fill in the body of {node.as_operand()}')
        self.code.append(f'  sys.set_current_module({self.generate_rval(node)});')
        self.visit_block(node.body)

    def visit_block(self, node: Block):
        if node.kind == Block.MODULE_ROOT:
            self.code.append('  // module root block')
        else:
            self.code.append('  // restore current block')
            self.code.append('  block_stack.push(sys.get_current_block().unwrap().upcast());')

            block_var = self.generate_rval(node)
            if isinstance(node, block.CondBlock):
                self.code.append('  // conditional block')
                cond = self.generate_rval(node.cond)
                self.code.append(f'  let {block_var} = sys.create_conditional_block({cond});')
                self.code.append(f'  sys.set_current_block({block_var});')
            elif isinstance(node, block.CycledBlock):
                self.code.append('  // cycled block')
                self.code.append(f'  let {block_var} = sys.create_cycled_block({node.cycle});')
                self.code.append(f'  sys.set_current_block({block_var});')

        for elem in node.iter():
            self.dispatch(elem)

        if isinstance(node, (block.CondBlock, block.CycledBlock)):
            self.code.append('  let restore = block_stack.pop().unwrap();')
            self.code.append('  sys.set_current_block(restore);')

    def generate_rval(self, node):
        '''Generate the value reference on as the right-hand side of an assignment'''
        if isinstance(node, const.Const):
            ty = generate_dtype(node.dtype)
            imm_var = f'imm_{hex(id(node))[-5:-1]}'
            imm_decl = f'  let {imm_var} = sys.get_const_int({ty}, {node.value}); // {node}'
            self.code.append(imm_decl)
            return imm_var
        if isinstance(node, module.Port):
            module_name = self.generate_rval(node.module)
            port_name = f'{module_name}_{node.name}'
            self.code.append(f'''  // Get port {node.name}
                let {port_name} = {{
                  let module = {module_name}.as_ref::<eir::ir::Module>(&sys).unwrap();
                  module.get_port_by_name("{node.name}").unwrap().upcast()
                }};
            ''')
            return port_name
        return node.as_operand()

    #pylint: disable=too-many-branches, too-many-locals, too-many-statements
    def visit_expr(self, node):
        self.code.append(f'  // {node}')
        ib_method = opcode_to_ib(node)
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
        elif isinstance(node, expr.FIFOPop):
            fifo = self.generate_rval(node.fifo)
            res = f'sys.{ib_method}({fifo});'
        elif isinstance(node, expr.Log):
            fmt = '"' + node.args[0] + '"'
            self.code.append(f'  let fmt = sys.get_str_literal({fmt}.into());')
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
        elif isinstance(node, expr.FIFOPush):
            bind_var = self.generate_rval(node.bind)
            fifo_name = node.fifo.name
            val = self.generate_rval(node.val)
            res = f'sys.add_bind({bind_var}, "{fifo_name}".into(), {val}, Some(false));'
        elif isinstance(node, expr.Bind):
            res = '// Already handled by `EmitBinds`'
        elif isinstance(node, expr.AsyncCall):
            bind_var = self.generate_rval(node.bind)
            res = f'sys.create_async_call({bind_var});'
        elif isinstance(node, expr.Concat):
            msb = self.generate_rval(node.msb)
            lsb = self.generate_rval(node.lsb)
            res = f'sys.{ib_method}(created_here!(), {msb}, {lsb});'
        elif isinstance(node, expr.Slice):
            x = self.generate_rval(node.x)
            l = self.generate_rval(node.l)
            r = self.generate_rval(node.r)
            res = f'sys.{ib_method}({x}, {l}, {r});'
        elif isinstance(node, expr.Cast):
            x = self.generate_rval(node.x)
            ty = generate_dtype(node.dtype)
            res = f'sys.{ib_method}(created_here!(), {x}, {ty});'
        elif isinstance(node, expr.Intrinsic):
            if node.opcode == expr.Intrinsic.WAIT_UNTIL:
                cond = self.generate_rval(node.args[0])
                res = f'sys.{ib_method}({cond});'
            else:
                length = len(repr(node)) - 1
                res = f'  // ^{"~" * length}: Support the instruction above'
        elif isinstance(node, expr.Select):
            cond = self.generate_rval(node.cond)
            true_value = self.generate_rval(node.true_value)
            false_value = self.generate_rval(node.false_value)
            res = f'sys.{ib_method}(created_here!(), {cond}, {true_value}, {false_value});'
        else:
            length = len(repr(node)) - 1
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
        size = node.size
        ty = generate_dtype(node.scalar_ty)
        init = self.generate_init_value(node.initializer, ty)
        self.code.append(f'  // {node}')
        attrs = ', '.join(f'eir::ir::data::ArrayAttr::{CG_ARRAY_ATTR[i]}' for i in node.attr)
        attrs = f'vec![{attrs}]'
        array_decl = f'  let {name} = sys.create_array({ty}, \"{name}\", {size}, {init}, {attrs});'
        self.code.append(array_decl)

    def __init__(self, **kwargs):
        self.code = []
        self.header = []
        self.emitted_bind = set()
        self.kwargs = kwargs

    def get_source(self):
        '''Concatenate the generated source code for the given system'''
        return '\n'.join(self.header) + '\n' + '\n'.join(self.code)

def codegen(sys: SysBuilder, **kwargs):
    '''
    The help function to generate the assassyn IR builder for the given system

    Args:
        sys (SysBuilder): The system to generate the builder for
        kwargs: Additional arguments to pass to the code
    '''
    cg = CodeGen(**kwargs)
    cg.visit_system(sys)
    return cg.get_source()
