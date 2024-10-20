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
from .utils import identifierize

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
    expr.BinaryOp.NEQ: 'neq',

    expr.BinaryOp.BITWISE_OR:  'bitwise_or',
    expr.BinaryOp.BITWISE_AND: 'bitwise_and',
    expr.BinaryOp.BITWISE_XOR: 'bitwise_xor',

    expr.BinaryOp.SHL: 'shl',
    expr.BinaryOp.SHR: 'shr',

    expr.UnaryOp.FLIP: 'flip',
    expr.UnaryOp.NEG: 'neg',

    expr.Slice.SLICE: 'slice',
    expr.Concat.CONCAT: 'concat',

    expr.PureInstrinsic.FIFO_PEEK: 'peek',
    expr.PureInstrinsic.FIFO_VALID: 'valid',
    expr.PureInstrinsic.MODULE_TRIGGERED: 'module_triggered',
    expr.PureInstrinsic.VALUE_VALID: 'value_valid',

    expr.FIFOPop.FIFO_POP: 'pop',
    expr.FIFOPush.FIFO_PUSH: 'push',

    expr.ArrayRead.ARRAY_READ: 'array_read',
    expr.ArrayWrite.ARRAY_WRITE: 'array_write',

    expr.AsyncCall.ASYNC_CALL: 'async_call',

    expr.Cast.BITCAST: 'bitcast',
    expr.Cast.ZEXT: 'zext',
    expr.Cast.SEXT: 'sext',

    expr.Select.SELECT: 'select',
    expr.Select1Hot.SELECT_1HOT: 'select_1hot',

    expr.Log.LOG: 'log',

    expr.intrinsic.Intrinsic.WAIT_UNTIL: 'wait_until',
    expr.intrinsic.Intrinsic.FINISH: 'finish',
    expr.intrinsic.Intrinsic.ASSERT: 'assert',
}

CG_MIDFIX = {
    expr.FIFOPop.FIFO_POP: 'fifo',
    expr.FIFOPush.FIFO_PUSH: 'fifo',
    expr.PureInstrinsic.FIFO_PEEK: 'fifo',
    expr.PureInstrinsic.FIFO_VALID: 'fifo',
}

CG_SIMULATOR = {
    'verilator': 'Verilator',
    'vcs': 'VCS',
}

def opcode_to_ib(node: Expr):
    '''Convert the opcode to the corresponding IR builder method'''
    opcode = node.opcode
    if node.opcode == expr.Bind.BIND:
        return ''
    midfix = f'_{CG_MIDFIX.get(opcode)}' if opcode in CG_MIDFIX else ''
    return f'create{midfix}_{CG_OPCODE[opcode]}'

def generate_dtype(ty: dtype.DType):
    '''Generate AST data type representation into assassyn data type representation'''
    prefix = 'assassyn::ir::DataType'
    if isinstance(ty, dtype.Int):
        return f'{prefix}::int_ty({ty.bits})'
    if isinstance(ty, dtype.UInt):
        return f'{prefix}::uint_ty({ty.bits})'
    if isinstance(ty, dtype.Bits):
        return f'{prefix}::bits_ty({ty.bits})'
    assert isinstance(ty, dtype.Record), 'Expecting a record type, but got {ty}'
    return f'{prefix}::bits_ty({ty.bits})'

def const_int_wrapper(value: int, ty: str):
    '''Generate the constant integer wrapper for the given value'''
    value = hex(value)
    if value[0] == '-':
        value = value[1:]
    if value.endswith('L'):
        value = value[:-1]
        assert value[2:].isnumeric() and len(value) <= 18, f'Int too large: {value}'
    return f'sys.get_const_int({ty}, {value} as u64)'

def generate_init_value(init_value, ty: str):
    '''Generate the initial value for the given array'''
    if init_value is None:
        return ("\n", "None")
    str1 = f'let init_val = {const_int_wrapper(init_value, ty)};'
    str2 = 'Some(vec![init_val])'

    return (str1, str2)

def generate_port(port: Port):
    '''Generate the port information for the given port for module construction'''
    ty = f'{generate_dtype(port.dtype)}'
    return f'assassyn::builder::PortInfo::new("{port.name}", {ty})'

class EmitBinds(visitor.Visitor):
    '''Gather all the binds and emit them in advance'''

    def __init__(self, cg):
        self.cg = cg

    def visit_expr(self, node):
        if isinstance(node, expr.Bind):
            bind_var = self.cg.generate_rval(node)
            module_var = self.cg.generate_rval(node.callee)
            self.cg.code.append(f'  let {bind_var} = sys.get_init_bind({module_var});')

# pylint: disable=too-many-instance-attributes
class CodeGen(visitor.Visitor):
    '''Generate the assassyn IR builder for the given system'''

    def emit_module_attrs(self, m: Module, var_id: str):
        '''Generate module attributes.'''
        module_mut = f'{var_id}.as_mut::<assassyn::ir::Module>(&mut sys).unwrap()'
        path = 'assassyn::ir::module::Attribute'
        if m.is_systolic:
            self.code.append(f'{module_mut}.add_attr({path}::Systolic);')
        if m.no_arbiter:
            self.code.append(f'{module_mut}.add_attr({path}::NoArbiter);')


    def emit_memory_attrs(self, m: module.SRAM, var_id):
        '''Emit the memory attributes only for downstream modules'''
        if isinstance(m, module.SRAM):
            module_mut = f'{var_id}.as_mut::<assassyn::ir::Module>(&mut sys).unwrap()'
            path = 'assassyn::ir::module'
            # (width, depth, init_file, we, re, addr, wdata)
            params = [f'{path}::attrs::MemoryParams::new(']
            params.append(f'{m.width}, // width')
            params.append(f'{m.depth}, // depth')
            params.append('1..=1, // lat')
            if m.init_file is not None:
                params.append(f'Some("{m.init_file}".into()), // init-file')
            else:
                params.append('None, // init-file')
            params.append(f'{path}::attrs::MemoryPins::new(')
            params.append(f'{self.generate_rval(m.payload)}, // array')
            params.append(f'{self.generate_rval(m.re)}, // re')
            params.append(f'{self.generate_rval(m.we)}, // we')
            params.append(f'{self.generate_rval(m.addr)}, // addr')
            params.append(f'{self.generate_rval(m.wdata)}, // wdata')
            params.append('))')
            params = '\n'.join(params)
            self.code.append(f'{module_mut}.add_attr({path}::Attribute::MemoryParams({params}));')

    def emit_config(self):
        '''Emit the configuration fed to the generated simulator'''
        idle_threshold = f'idle_threshold: {self.idle_threshold}'
        sim_threshold = f'sim_threshold: {self.sim_threshold}'
        random_option = f'random: {self.random}'
        config = [idle_threshold, sim_threshold, random_option]
        if self.resource_base is not None:
            resource_base = f'resource_base: PathBuf::from("{self.resource_base}")'
            config.append(resource_base)
        if 'verilog' in self.targets:
            verilog_target = self.targets['verilog']
            simulator = f'assassyn::backend::verilog::Simulator::{CG_SIMULATOR[verilog_target]}'
            config.append(f'verilog: {simulator}')
        return ', '.join(config)

    def generate_init_value(self, init_value, ty: str):
        '''Generate the initializer vector. NOTE: ty is already generated in an str!'''
        if init_value is None:
            return 'None'

        vec = []
        for i, j in enumerate(init_value):
            self.code.append(f'let init_{i} = {const_int_wrapper(j, ty)};')
            vec.append(f'init_{i}')

        self.code.append(f'let init = vec![{", ".join(vec)}];')

        return 'Some(init)'


    # pylint: disable=too-many-locals, too-many-statements
    def visit_system(self, node: SysBuilder):
        self.header.append('use std::path::PathBuf;')
        self.header.append('use std::collections::HashMap;')
        self.header.append('use assassyn::builder::SysBuilder;')
        self.header.append('use assassyn::ir::node::IsElement;')
        self.code.append('fn main() {')
        self.code.append(f'  let mut sys = SysBuilder::new(\"{node.name}\");')
        self.code.append(
                '  let mut block_stack : Vec<assassyn::ir::node::BaseNode> = Vec::new();\n')
        self.code.append('  // Declare modules')
        for elem in node.modules:
            lval = elem.as_operand()
            name = elem.name.lower()
            ports = ', '.join(generate_port(p) for p in elem.ports)
            self.code.append(f'  let {lval} = sys.create_module("{name}", vec![{ports}]);')
            self.emit_module_attrs(elem, lval)
        self.code.append('  // Declare downstream modules')
        for elem in node.downstreams:
            var = self.generate_rval(elem)
            self.code.append(f'  let {var} = sys.create_downstream("{elem.name}");')
        self.code.append('  // declare arrays')
        for elem in node.arrays:
            self.visit_array(elem)
        self.code.append('  // Gathered binds')
        for elem in node.modules + node.downstreams:
            bind_emitter = EmitBinds(self)
            name = self.generate_rval(elem)
            self.code.append('  // Set the current module redundantly to emit related binds')
            self.code.append(f'  sys.set_current_module({name});')
            bind_emitter.visit_module(elem)
        for elem in node.modules:
            self.visit_module(elem)

        self.finalize_bind()

        self.code.append('  // Emit downstream modules')
        for elem in node.downstreams:
            self.code.append(f'  // Module {elem.name}')
            var = self.generate_rval(elem)
            self.code.append(f'  sys.set_current_module({var});')
            # FIXME(@were): This is a hack to emit memory parameters, it should be generalized
            self.emit_memory_attrs(elem, var)
            self.visit_module(elem)

        for elem , kind in node.exposed_nodes.items():
            name = elem.name if f'{id(elem)}' in elem.name else self.generate_rval(elem)
            size = elem.size
            ty = generate_dtype(elem.scalar_ty)
            if kind is None:
                kind = 'Inout'
            self.code.append(f'  // try0 {elem} {kind}')
            self.code.append(f'  // try1 {name} {size} {ty}')
            self.visit_exposed_nodes(elem)
            self.code.append(
                f'  sys.expose_to_top({elem.name}, '
                f'assassyn::builder::system::ExposeKind::{kind});'
            )

        config = self.emit_config()
        self.code.append(f'''
            let mut config = assassyn::backend::common::Config{{
               {config},
               ..Default::default()
            }};
        ''')
        self.code.append('  println!("{}", sys);')
        config = 'assassyn::xform::Config{ rewrite_wait_until: true }'
        self.code.append(f'  assassyn::xform::basic(&mut sys, &{config});')
        backend = 'assassyn::backend'
        if self.targets['simulator']:
            base_dir = '(env!("CARGO_MANIFEST_DIR").to_string()).into()'
            self.code.append(f'  config.base_dir = {base_dir};')
            self.code.append(f'  {backend}::simulator::elaborate(&sys, &config).unwrap();')
        if 'verilog' in self.targets:
            base_dir = '(env!("CARGO_MANIFEST_DIR").to_string()).into()'
            self.code.append(f'  config.base_dir = {base_dir};')
            self.code.append(f'  {backend}::verilog::elaborate(&sys, &config).unwrap();')
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
            imm_var = f'imm_{identifierize(node)}'
            imm_decl = f'  let {imm_var} = {const_int_wrapper(node.value, ty)}; // {node}'
            self.code.append(imm_decl)
            return imm_var
        if isinstance(node, module.Port):
            module_name = self.generate_rval(node.module)
            port_name = f'{module_name}_{node.name}'
            self.code.append(f'''  // Get port {node.name}
                let {port_name} = {{
                  let module = {module_name}.as_ref::<assassyn::ir::Module>(&sys).unwrap();
                  module.get_fifo("{node.name}").unwrap().upcast()
                }};''')
            return port_name
        return node.as_operand()

    #pylint: disable=too-many-branches, too-many-locals, too-many-statements
    def visit_expr(self, node):
        self.code.append(f'  // {node}, {node.loc}')
        ib_method = opcode_to_ib(node)
        if node.is_binary():
            lhs = self.generate_rval(node.lhs)
            rhs = self.generate_rval(node.rhs)
            res = f'sys.{ib_method}({lhs}, {rhs});'
        elif node.is_unary():
            x = self.generate_rval(node.x)
            res = f'sys.{ib_method}({x});'
        elif isinstance(node, expr.PureInstrinsic):
            if len(node.args) == 1:
                master = self.generate_rval(node.args[0])
                res = f'sys.{ib_method}({master});'
            fifo = self.generate_rval(node.args[0])
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
            arr = self.generate_rval(node.arr)
            idx = self.generate_rval(node.idx)
            res = f'sys.{ib_method}({arr}, {idx});'
        elif isinstance(node, expr.ArrayWrite):
            arr = self.generate_rval(node.arr)
            idx = self.generate_rval(node.idx)
            val = self.generate_rval(node.val)
            res = f'sys.{ib_method}({arr}, {idx}, {val});'
        elif isinstance(node, expr.FIFOPush):
            bind_var = self.generate_rval(node.bind)
            fifo_name = node.fifo.name
            val = self.generate_rval(node.val)
            res = f'let push{bind_var} = sys.bind_arg({bind_var}, "{fifo_name}".into(), {val});'
        elif isinstance(node, expr.Bind):
            fifo_depths = node.fifo_depths
            self.fifo_depths.update(fifo_depths)
            res = f'// Already handled by `EmitBinds` {fifo_depths}'
        elif isinstance(node, expr.AsyncCall):
            bind_var = self.generate_rval(node.bind)
            res = f'sys.create_async_call({bind_var});'
        elif isinstance(node, expr.Concat):
            msb = self.generate_rval(node.msb)
            lsb = self.generate_rval(node.lsb)
            res = f'sys.{ib_method}({msb}, {lsb});'
        elif isinstance(node, expr.Slice):
            x = self.generate_rval(node.x)
            l = self.generate_rval(node.l)
            r = self.generate_rval(node.r)
            res = f'sys.{ib_method}({x}, {l}, {r});'
        elif isinstance(node, expr.Cast):
            x = self.generate_rval(node.x)
            ty = generate_dtype(node.dtype)
            res = f'sys.{ib_method}({x}, {ty});'
        elif isinstance(node, expr.Intrinsic):
            if node.opcode in [expr.Intrinsic.WAIT_UNTIL, expr.Intrinsic.ASSERT]:
                cond = self.generate_rval(node.args[0])
                res = f'sys.{ib_method}({cond});'
            elif node.opcode == expr.Intrinsic.FINISH:
                res = f'sys.{ib_method}();'
            else:
                length = len(repr(node)) - 1
                res = f'  // ^{"~" * length}: Support the instruction above'
        elif isinstance(node, expr.Select):
            cond = self.generate_rval(node.cond)
            true_value = self.generate_rval(node.true_value)
            false_value = self.generate_rval(node.false_value)
            res = f'sys.{ib_method}({cond}, {true_value}, {false_value});'
        elif isinstance(node, expr.Select1Hot):
            cond = self.generate_rval(node.cond)
            values = ', '.join(self.generate_rval(i) for i in node.values)
            res = f'sys.{ib_method}({cond}, vec![{values}]);'
        # TODO(@were): For now, optional is a ad-hoc solution for downstream's inputs.
        # Later, it will be replaced by a more general IR node, predicated select.
        # The predicated condition is not fully supported in the current assassyn IR.
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


    def generate_array_attr(self, node: Array):
        '''Generate the array attributes for the given array'''
        attrs = []
        path = 'assassyn::ir::array::ArrayAttr'
        for attr in node.attr:
            if attr == Array.FULLY_PARTITIONED:
                attrs.append(f'{path}::FullyPartitioned')
            elif isinstance(attr, module.SRAM):
                # Skip this, this is handled in the module attributes
                pass
            else:
                assert False, f'Unsupported memory attribute {attr}'
        return ', '.join(attrs)


    def visit_array(self, node: Array):
        name = node.name if f'{id(node)}' in node.name else self.generate_rval(node)
        size = node.size
        ty = generate_dtype(node.scalar_ty)
        init = self.generate_init_value(node.initializer, ty)
        self.code.append(f'  // {node}')
        attrs = self.generate_array_attr(node)
        attrs = f'vec![{attrs}]'
        array_decl = f'  let {name} = sys.create_array({ty}, \"{name}\", {size}, {init}, {attrs});'
        self.code.append(array_decl)

    def visit_exposed_nodes(self, node: Array):
        """Generate the exposed nodes for the given array"""
        name = node.name if f'{id(node)}' in node.name else self.generate_rval(node)
        size = node.size
        ty = generate_dtype(node.scalar_ty)
        self.code.append(f'  // {node}')
        attrs = self.generate_array_attr(node)
        attrs = f'vec![{attrs}]'
        array_decl = (
            f'  let Exposed_{name} = sys.create_custom_basenode('
            f'{ty}, "Exposed_{name}", {size}, {attrs});'
        )
        self.code.append(array_decl)

    def __init__(self, simulator, verilog, idle_threshold, sim_threshold, random, resource_base): #pylint: disable=too-many-arguments
        self.code = []
        self.header = []
        self.emitted_bind = set()
        self.targets = {}
        self.resource_base = resource_base
        if simulator:
            self.targets['simulator'] = True
        if verilog:
            self.targets['verilog'] = verilog
        self.idle_threshold = idle_threshold
        self.sim_threshold = sim_threshold
        self.random = random
        self.fifo_depths = {}

    def get_source(self):
        '''Concatenate the generated source code for the given system'''
        return '\n'.join(self.header) + '\n' + '\n'.join(self.code)

    def finalize_bind(self):
        '''Finalize the bind by setting the FIFO depths'''
        self.code.append('  // Set FIFO depths')
        for push, depth in self.fifo_depths.items():
            depth = depth if depth & (depth - 1) == 0 \
                else 1 << (depth - 1).bit_length()
            res = f'''  push{push}.as_mut::<assassyn::ir::Expr>(&mut sys).unwrap()
                            .add_metadata(assassyn::ir::expr::Metadata::FIFODepth({depth}));
            '''
            self.code.append(res)

def codegen( #pylint: disable=too-many-arguments
        sys: SysBuilder,
        simulator,
        verilog,
        idle_threshold,
        sim_threshold,
        random,
        resource_base):
    '''
    The help function to generate the assassyn IR builder for the given system

    Args:
        sys (SysBuilder): The system to generate the builder for
        kwargs: Additional arguments to pass to the code
    '''
    cg = CodeGen(simulator, verilog, idle_threshold, sim_threshold, random, resource_base)
    cg.visit_system(sys)
    return cg.get_source()
