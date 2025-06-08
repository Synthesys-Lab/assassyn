"""Verilog design generation and code dumping."""

from typing import List, Dict, Tuple

from ...analysis import expr_externally_used
from ...ir.module import Module, Downstream, Port
from ...builder import SysBuilder
from ...ir.visitor import Visitor
from ...ir.block import Block, CondBlock
from ...ir.const import Const
from ...ir.array import Array
from ...ir.dtype import Int, UInt, Bits, DType
from ...utils import namify, unwrap_operand
from ...ir.expr import (
    Expr,
    BinaryOp,
    UnaryOp,
    FIFOPop,
    Log,
    ArrayRead,
    ArrayWrite,
    FIFOPush,
    PureIntrinsic,
    AsyncCall,
    Slice,
    Concat,
    Cast,
    Select,
    Bind,
    Select1Hot,
    Intrinsic
)

def dump_rval(node, with_namespace: bool) -> str:  # pylint: disable=too-many-return-statements
    """Dump a reference to a node with options."""

    node = unwrap_operand(node)

    if isinstance(node, Module):
        return namify(node.name)
    if isinstance(node, Array):
        array = node
        return namify(array.name)
    if isinstance(node, Port):
        return namify(node.name)
    if isinstance(node, FIFOPop):
        if not with_namespace:
            return f'self.{namify(node.fifo.name)}'
        return namify(node.fifo.module.name) + "_" + namify(node.fifo.name)
    if isinstance(node, Const):
        int_imm = node
        value = int_imm.value
        ty = dump_type(int_imm.dtype)
        return f"{ty}({value})"
    if isinstance(node, str):
        value = node
        return f'"{value}"'
    if isinstance(node, Expr):
        raw = namify(node.as_operand())
        return raw
    raise ValueError(f"Unknown node of kind {type(node).__name__}")

def dump_type(ty: DType) -> str:
    """Dump a type to a string."""
    if isinstance(ty, Int):
        return f"SInt({ty.bits})"
    if isinstance(ty, UInt):
        return f"UInt({ty.bits})"
    if isinstance(ty, Bits):
        return f"Bits({ty.bits})"
    raise ValueError(f"Unknown type: {type(ty)}")

def dump_type_cast(ty: DType) -> str:
    """Dump a type to a string."""
    if isinstance(ty, Int):
        return "as_sint()"
    if isinstance(ty, UInt):
        return "as_uint()"
    if isinstance(ty, Bits):
        return "as_bits()"
    raise ValueError(f"Unknown type: {type(ty)}")

class CIRCTDumper(Visitor):  # pylint: disable=too-many-instance-attributes
    """Dumps IR to CIRCT-compatible Verilog code."""

    wait_until: bool
    indent: int
    code: List[str]
    cond_stack: List[str]
    _exposes: Dict[Expr, List[Tuple[Expr, str]]]
    logs: List[str]
    connections: List[Tuple[Module, str, str]]
    current_module: Module

    def __init__(self):
        super().__init__()
        self.wait_until = None
        self.indent = 0
        self.code = []
        self._exposes = {}
        self.cond_stack = []
        self.logs = []
        self.connections = []
        self.current_module = None

    def get_pred(self) -> str:
        """Get the current predicate for conditional execution."""
        if not self.cond_stack:
            return "Bits(1)(1)" if self.wait_until is None else self.wait_until
        return (" & ".join(self.cond_stack) +
                (" & {self.wait_until}" if self.wait_until is not None else ""))

    def append_code(self, code: str):
        """Append code with proper indentation."""
        if code.strip() == '':
            self.code.append('')
        else:
            self.code.append(self.indent * ' ' + code)

    def append_port(self, name: str, kind: str, dtype: str, connect: str):
        """Append a port definition with connection."""
        self.append_code(f'{name} = {kind}({dtype})')
        if kind == 'Input':
            self.connections.append((self.current_module, name, connect))

    def expose(self, kind: str, expr: Expr):
        ''' Expose an expression out of the module.'''
        ret = False  # pylint: disable=unused-variable
        key = None
        if kind == 'expr':
            key = expr
        elif kind == 'array':  # pylint: disable=possibly-used-before-assignment
            assert isinstance(expr, (ArrayRead, ArrayWrite))
            key = expr.array
        if kind == 'fifo':
            assert isinstance(expr, FIFOPush)
            key = expr.fifo
        if kind == 'trigger':
            assert isinstance(expr, AsyncCall)
            key = expr.bind.callee
        assert key is not None
        if key not in self._exposes:
            self._exposes[key] = []
        self._exposes[key].append((expr, self.get_pred()))

    def visit_block(self, node: Block):
        if isinstance(node, CondBlock):
            cond = dump_rval(node.cond, False)
            self.cond_stack.append("(" + cond + ")")
        for i in node.body:
            if isinstance(i, Expr):
                self.visit_expr(i)
            elif isinstance(i, Block):
                self.visit_block(i)
            else:
                raise ValueError(f'Unknown node type: {type(node)}')
        if isinstance(node, CondBlock):
            self.cond_stack.pop()

    def visit_expr(self, expr: Expr):  # pylint: disable=arguments-renamed,too-many-locals,too-many-branches,too-many-statements
        # Handle different expression types
        self.append_code(f'# {expr}')
        body = None
        rval = dump_rval(expr, False)
        if isinstance(expr, BinaryOp):
            binop = expr.opcode
            dtype = expr.dtype
            a = dump_rval(expr.lhs, False)
            op_str = BinaryOp.OPERATORS[expr.opcode]
            if binop == BinaryOp.SHR:
                op_str = ">>>" if dtype.is_signed() else ">>"
            b = dump_rval(expr.rhs, False)
            body = f"(({a} {op_str} {b}).as_bits()[0:{dtype.bits}]).{dump_type_cast(dtype)}"
            body = f'{dump_rval(expr, False)} = {body}'
        elif isinstance(expr, UnaryOp):
            uop = expr.opcode
            op_str = "~" if uop == UnaryOp.FLIP else "-"
            x = dump_rval(expr.x, False)
            body = f"{op_str}{x}"
            body = f'self.{dump_rval(expr, with_namespace=False)} = {body}'
        elif isinstance(expr, FIFOPop):
            fifo = dump_rval(expr.fifo, False)
            dtype = dump_type(expr.fifo.dtype)
            body = f'self.{fifo}_pop_ready = {self.get_pred()}'
        elif isinstance(expr, Log):
            formatter = expr.operands[0]
            args = []
            for i in expr.operands[1:]:
                self.expose('expr', unwrap_operand(i))
                rval = dump_rval(unwrap_operand(i), True)
                args.append(f'dut.{rval}.value')
            args = ", ".join(args)
            self.logs.append(f'# {expr}')
            self.logs.append(f'print("{formatter}".format({args}))')
        elif isinstance(expr, ArrayRead):
            array_ref = expr.array
            array_idx = unwrap_operand(expr.idx)
            array_idx = (dump_rval(array_idx, False)
                         if not isinstance(array_idx, Const) else array_idx.value)
            array_name = dump_rval(array_ref, False)
            rval = dump_rval(expr, False)
            body = f'{rval} = self.{array_name}_payload[{array_idx}]'
            self.expose('array', expr)
        elif isinstance(expr, ArrayWrite):
            self.expose('array', expr)
        elif isinstance(expr, FIFOPush):
            self.expose('fifo', expr)
        elif isinstance(expr, PureIntrinsic):
            rval = dump_rval(expr, False)
            intrinsic = expr.opcode
            if intrinsic in [PureIntrinsic.FIFO_VALID, PureIntrinsic.FIFO_PEEK]:
                fifo = expr.args[0]
                fifo_name = dump_rval(fifo, False)
                if intrinsic == PureIntrinsic.FIFO_PEEK:
                    body = f'{rval} = self.{fifo_name}_data'
                elif intrinsic == PureIntrinsic.FIFO_VALID:
                    body = f'{rval} = self.{fifo_name}_valid'
            elif intrinsic == PureIntrinsic.VALUE_VALID:
                value = expr.operands[0].value
                value_expr = value
                if value_expr.parent.module != expr.parent.module:
                    body = f"{rval} = self.{namify(str(value_expr).as_operand())}_valid"  # pylint: disable=no-member
                else:
                    pred = self.get_pred()
                    body = f"{rval} = (executed & {pred})"
            else:
                # TODO(@were): Handle other intrinsics
                raise ValueError(f"Unknown intrinsic: {expr}")
        elif isinstance(expr, AsyncCall):
            self.expose('trigger', expr)
        elif isinstance(expr, Slice):
            a = dump_rval(expr.x, False)
            l = expr.l.value.value
            r = expr.r.value.value
            body = f"{a}[{r}:{l}]"
        elif isinstance(expr, Concat):
            a = dump_rval(expr.msb, False)
            b = dump_rval(expr.lsb, False)
            body = f"{{{a}, {b}}}"
        elif isinstance(expr, Cast):
            dbits = expr.dtype.bits
            a = dump_rval(expr.x, False)
            src_dtype = expr.src_type
            pad = dbits - src_dtype.bits
            if expr.cast_kind == Cast.BITCAST:
                assert pad == 0
                if isinstance(expr.dtype, Int):
                    body = f"{a}.as_sint()"
                elif isinstance(expr.dtype, UInt):
                    body = f"{a}.as_uint()"
                elif isinstance(expr.dtype, Bits):
                    body = f"{a}.as_bits()"
                else:
                    raise ValueError(f"Unknown cast type: {expr.dtype}")
            elif expr.cast_kind == Cast.ZEXT:
                body = f"{{{pad}'b0, {a}}}"
            elif expr.cast_kind == Cast.SEXT:
                dest_dtype = expr.dtype
                if (src_dtype.is_int() and src_dtype.is_signed and
                    dest_dtype.is_int() and dest_dtype.is_signed and
                    dest_dtype.bits > src_dtype.bits):
                    # perform sext
                    body = f"{{{pad}'{{{a}[{src_dtype.bits - 1}]}}, {a}}}"
                else:
                    body = f"{{{pad}'b0, {a}}}"
        elif isinstance(expr, Select):
            cond = dump_rval(expr.cond, True)
            true_value = dump_rval(expr.true_value, True)
            false_value = dump_rval(expr.false_value, True)
            body = f'Mux({cond}, {false_value}, {true_value})'
        elif isinstance(expr, Bind):
            # handled in AsyncCall
            body = None
        elif isinstance(expr, Select1Hot):
            dbits = expr.dtype.bits
            cond = dump_rval(expr.cond, False)
            terms = []
            mask = "1" * expr.dtype.bits  # pylint: disable=unused-variable
            mask = "Bits({expr.dtype.bits})({mask})"
            dtype = f"Bits({expr.dtype.bits})"
            for i, elem in enumerate(expr.values):
                value = dump_rval(elem, False)
                terms.append("(Mux({cond}[i:i], {dtype}(0), {mask}) & {value})")
            body = " | ".join(terms)
        elif isinstance(expr, Intrinsic):
            intrinsic = expr.opcode
            if intrinsic == Intrinsic.FINISH:
                pred = self.get_pred() or "1"
                body = (f"\n`ifndef SYNTHESIS\n  always_ff @(posedge clk) "
                        f"if (executed && {pred}) $finish();\n`endif\n")
            elif intrinsic == Intrinsic.ASSERT:
                self.expose('expr', expr.args[0])
            elif intrinsic == Intrinsic.WAIT_UNTIL:
                cond = dump_rval(expr.args[0], False)
                self.append_code(f'self.executed = {cond}')
                self.wait_until = "(" + cond + ")"
            else:
                raise ValueError(f"Unknown block intrinsic: {expr}")
        else:
            raise ValueError(f"Unhandled expression type: {type(expr).__name__}")

        # Handle expressions that are externally used
        if expr.is_valued() and expr_externally_used(expr, True):
            self.expose('expr', expr)

        if body is not None:
            self.append_code(body)

    def cleanup_post_generation(self):  # pylint: disable=too-many-locals,too-many-branches,too-many-statements
        """Clean up and finalize code generation."""
        self.append_code('')
        for key, exposes in self._exposes.items():
            if isinstance(key, Array):
                array = dump_rval(key, False)
                has_write = False
                ce = "Bits(1)(0)"
                widx = f"{dump_type(key.index_type())}(0)"
                wdata = f"Bits({key.scalar_ty.bits})(0)"
                for expr, pred in exposes:
                    if isinstance(expr, ArrayWrite):
                        has_write = True
                        self.append_code(f'# Expose: {expr}')
                        ce = ce + " | " + pred
                        idx = dump_rval(expr.idx, False)
                        data = dump_rval(expr.val, False)
                        widx = f"Mux({pred}, {widx}, {idx})"
                        wdata = f"Mux({pred}, {wdata}, {data}.as_bits())"
                if has_write:
                    self.append_code(f'self.{array}_ce = {ce}')
                    self.append_code(f'self.{array}_wdata = {wdata}')
                    if key.index_bits > 0:
                        self.append_code(f'self.{array}_widx = {widx}')
            elif isinstance(key, Port):
                push_valid = "Bits(1)(0)"
                push_data = f"{dump_type(key.dtype)}(0)"
                fifo = dump_rval(key, False)
                for expr, pred in exposes:
                    self.append_code(f'# {expr}')
                    assert isinstance(expr, FIFOPush)
                    rval = dump_rval(expr.val, False)
                    push_valid = push_valid + " | " + pred
                    push_data = f"Mux({pred}, {push_data}, {rval})"
                self.append_code(f'self.{fifo}_push_valid = {push_valid}')
                self.append_code(f'self.{fifo}_push_data = {push_data}')
            elif isinstance(key, Expr):
                for expr, _ in exposes:
                    self.append_code(f'# Expose: {expr}')
                expr, pred = exposes[0]
                rval = dump_rval(expr, False)
                self.append_code(f'self.expose_{dump_rval(expr, True)} = {rval}')
                self.append_code(f'self.valid_{dump_rval(expr, True)} = {pred}')
            elif isinstance(key, Module):
                rval = dump_rval(key, False)
                ce = "Bits(1)(0)"
                for expr, pred in exposes:
                    self.append_code(f'# {expr}')
                    ce = ce + " | " + pred
                self.append_code(f'self.{rval}_trigger = {ce}')
        if self.wait_until is None:
            self.append_code('self.executed = Bits(1)(1)')  # pylint: disable=f-string-without-interpolation

        self.indent -= 4
        self.append_code('')
        for key, exposes in self._exposes.items():
            if isinstance(key, Array):
                array = dump_rval(key, False)
                read_dumped = write_dumped = False
                for expr, pred in exposes:
                    if isinstance(expr, ArrayRead) and not read_dumped:
                        scalar_ty = dump_type(expr.dtype)
                        self.append_code(f'{array}_payload = Input(Array({scalar_ty}, {key.size}))')
                        read_dumped = True
                    elif isinstance(expr, ArrayWrite) and not write_dumped:
                        self.append_code(f'{array}_ce = Output(Bits(1))')
                        self.append_code(f'{array}_wdata = Output(Bits({expr.val.dtype.bits}))')
                        if key.index_bits > 0:
                            self.append_code(
                                f'{array}_widx = Output({dump_type(key.index_type())})')
                        write_dumped = True
                    if read_dumped and write_dumped:
                        break
            elif isinstance(key, FIFOPush):
                self.append_code(f'{key}_push_valid = Output(Bits(1))')
                self.append_code(f'{key}_push_data = Output({dump_type(key.dtype)})')
            elif isinstance(key, Expr):
                rval = namify(dump_rval(key, True))
                self.append_code(f'expose_{rval} = Output({dump_type(expr.dtype)})')
                self.append_code(f'valid_{rval} = Output(Bits(1))')
            elif isinstance(key, Module):
                rval = dump_rval(key, False)
                self.append_code(f'{rval}_trigger = Output(Bits(1))')

    def visit_module(self, node: Module):
        self.wait_until = None
        self._exposes = {}
        self.cond_stack = []
        self.append_code(f'class {node.name}(Module):')
        if not isinstance(node, Downstream):
            self.indent += 4
            self.append_code('clk = Clock()')
            self.append_code('rst = Reset()')
            self.append_code('executed = Output(Bits(1))')
            for i in node.ports:
                dtype = dump_type(i.dtype)
                name = namify(i.name)
                self.append_code(f'{name} = Input({dtype})')
                self.append_code(f'{name}_valid = Input(Bits(1))')
                self.append_code(f'{name}_pop_ready = Output(Bits(1))')
            self.append_code('')
            self.append_code('@generator')
            self.append_code('def construct(self):')
            self.indent += 4
            self.visit_block(node.body)
            self.cleanup_post_generation()
        else:
            assert False, "TODO"
        self.indent -= 4
        self.append_code('')
        self._exposes  # pylint: disable=pointless-statement
        self.append_code(
            f'system = System([{node.name}], name="{node.name}", output_directory="sv")')
        self.append_code('system.compile()')

HEADER = '''from pycde import Input, Output, Module, System, Clock, Reset
from pycde import generator, modparams
from pycde.constructs import Reg, Array, Mux
from pycde.types import Bits, SInt, UInt\n

@modparams
def FIFO(WIDTH: int, DEPTH_LOG2: int):
    class FIFOImpl(Module):
        module_name = f"fifo"
        # Define inputs
        clk = Clock()
        rst_n = Input(Bits(1))
        push_valid = Input(Bits(1))
        push_data = Input(Bits(WIDTH))
        pop_ready = Input(Bits(1))
        # Define outputs
        push_ready = Output(Bits(1))
        pop_valid = Output(Bits(1))
        pop_data = Output(Bits(WIDTH))
    return FIFOImpl

@modparams
def TriggerCounter(WIDTH: int):
    class TriggerCounterImpl(Module):
        module_name = f"trigger_counter"
        clk = Clock()
        rst_n = Input(Bits(1))
        trigger = Input(Bits(1))
        count = Output(Bits(WIDTH))
    return TriggerCounterImpl

'''

def generate_design(fname: str, sys: SysBuilder):
    """Generate a complete Verilog design file for the system."""
    with open(fname, 'w', encoding='utf-8') as fd:
        # Generate the header
        fd.write(HEADER)
        # Generate the module implementations
        dumper = CIRCTDumper()
        dumper.visit_system(sys)
        code = '\n'.join(dumper.code)
        fd.write(code)
        # Generate the top function

    return dumper.logs
