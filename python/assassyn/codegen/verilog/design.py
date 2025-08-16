# pylint: disable=C0302
"""Verilog design generation and code dumping."""

from typing import List, Dict, Tuple
from string import Formatter

from .utils import HEADER,dump_type, dump_type_cast
from ...analysis import expr_externally_used
from ...ir.module import Module, Downstream, Port
from ...builder import SysBuilder
from ...ir.visitor import Visitor
from ...ir.block import Block, CondBlock,CycledBlock
from ...ir.const import Const
from ...ir.array import Array
from ...ir.dtype import Int, Bits, Record,RecordValue
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
    sys: SysBuilder
    async_callees: Dict[Module, List[Module]]
    downstream_dependencies: Dict[Module, List[Module]]
    is_top_generation: bool
    finish_body:str

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
        self.sys = None
        self.async_callees = {}
        self.exposed_ports_to_add = []
        self.downstream_dependencies = {}
        self.is_top_generation = False
        self.array_users = {}
        self.finish_body = None

    def get_pred(self) -> str:
        """Get the current predicate for conditional execution."""
        if not self.cond_stack:
            return "Bits(1)(1)"
        return " & ".join([s for s, _ in self.cond_stack])

    def get_external_port_name(self, node: Expr) -> str:
        """Get the mangled port name for an external value."""
        # This logic should mirror the port creation logic in visit_module.
        port_name = namify(node.as_operand())
        if port_name.startswith("_"):
            port_name = f"port{port_name}"
        return port_name


    def dump_rval(self,node, with_namespace: bool,module_name:str=None) -> str:  # pylint: disable=too-many-return-statements
        """Dump a reference to a node with options."""

        node = unwrap_operand(node)
        if isinstance(node, Expr) and node in self.current_module.externals \
            and not self.is_top_generation:
            return f"self.{self.get_external_port_name(node)}"
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
            if with_namespace:
                owner_module_name = namify(node.parent.module.name)
                if owner_module_name is None:
                    owner_module_name = module_name
                return f"{owner_module_name}_{raw}"
            return raw
        if isinstance(node, RecordValue):
            return self.dump_rval(node.value(), with_namespace, module_name)

        raise ValueError(f"Unknown node of kind {type(node).__name__}")

    def append_code(self, code: str):
        """Append code with proper indentation."""
        if code.strip() == '':
            self.code.append('')
        else:
            self.code.append(self.indent * ' ' + code)

    def expose(self, kind: str, expr: Expr):
        ''' Expose an expression out of the module.'''
        key = None
        if kind == 'expr':
            key = expr

        elif kind == 'array':
            assert isinstance(expr, (ArrayRead, ArrayWrite))
            key = expr.array
        elif kind == 'fifo':
            assert isinstance(expr, FIFOPush)
            key = expr.fifo
        elif kind == 'fifo_pop':
            assert isinstance(expr, FIFOPop)
            key = expr.fifo
        elif kind == 'trigger':
            assert isinstance(expr, AsyncCall)
            key = expr.bind.callee

        assert key is not None
        if key not in self._exposes:
            self._exposes[key] = []
        self._exposes[key].append((expr, self.get_pred()))

    def visit_block(self, node: Block):
        is_cond = isinstance(node, CondBlock)
        is_cycle = isinstance(node, CycledBlock)

        if is_cond:
            cond_str = self.dump_rval(node.cond, False)
            self.cond_stack.append((f"({cond_str})", node))
            def has_side_effect(block: Block) -> bool:
                for item in block.body:
                    if isinstance(item, Log):
                        return True
                    if isinstance(item, Block) and has_side_effect(item):
                        return True
                return False

            if has_side_effect(node):
                self.expose('expr', node.cond)

        elif is_cycle:
            self.cond_stack.append((f"(self.cycle_count == {node.cycle})", node))

        for i in node.body:
            if isinstance(i, Expr):
                self.visit_expr(i)
            elif isinstance(i, Block):
                self.visit_block(i)
            elif isinstance(i, RecordValue):
                pass
            else:
                print(i)
                raise ValueError(f'Unknown node type: {type(i)}')

        if is_cond or is_cycle:
            self.cond_stack.pop()


    def visit_expr(self, expr: Expr):  # pylint: disable=arguments-renamed,too-many-locals,too-many-branches,too-many-statements
        self.append_code(f'# {expr}')
        body = None
        rval = self.dump_rval(expr, False)

        if isinstance(expr, BinaryOp):
            binop = expr.opcode
            dtype = expr.dtype

            lhs_type = expr.lhs.dtype
            rhs_type = expr.rhs.dtype

            a = self.dump_rval(expr.lhs, False)
            b = self.dump_rval(expr.rhs, False)

            if binop in [BinaryOp.SHL, BinaryOp.SHR] or 'SHR' in str(binop):

                if lhs_type.bits != rhs_type.bits:
                    b = \
                    f"BitsSignal.concat([Bits({lhs_type.bits - rhs_type.bits})(0), {b}.as_bits()])"

                b = f"{b}.as_bits()"
                a = f"{a}.as_bits()"

                op_class_name = None
                if binop == BinaryOp.SHL:
                    op_class_name = "comb.ShlOp"
                elif binop == BinaryOp.SHR:
                    if expr.lhs.dtype.is_signed():
                        op_class_name = "comb.ShrSOp"
                    else:
                        op_class_name = "comb.ShrUOp"

                if op_class_name is None:
                    raise TypeError(f"Unhandled shift operation: {binop}")
                body = (
                    f"{rval} = {op_class_name}({a}.as_bits(), {b}.as_bits())"
                    f".as_bits({dtype.bits})[0:{dtype.bits}]"
                    f".{dump_type_cast(dtype)}"
                )
            elif binop == BinaryOp.MOD:
                if expr.dtype.is_signed():
                    op_class_name = "comb.ModSOp"
                else:
                    op_class_name = "comb.ModUOp"
                body = (
                    f"{rval} = {op_class_name}({a}.as_bits(), {b}.as_bits())"
                    f".as_bits({dtype.bits})[0:{dtype.bits}]"
                    f".{dump_type_cast(dtype)}"
                )
            else:
                op_str = BinaryOp.OPERATORS[expr.opcode]
                if expr.lhs.dtype != expr.rhs.dtype:
                    b=f"{b}.{dump_type_cast(expr.lhs.dtype )}"
                if op_str == "&":
                    if expr.rhs.dtype != Bits:
                        b=f"{b}.as_bits()"
                op_body = f"(({a} {op_str} {b}).{dump_type_cast(dtype)})"
                body = f'{rval} = {op_body}'

        elif isinstance(expr, UnaryOp):
            uop = expr.opcode
            target_cast_str = dump_type_cast(expr.dtype)
            op_str = "~" if uop == UnaryOp.FLIP else "-"
            x = self.dump_rval(expr.x, False)
            if uop == UnaryOp.FLIP:
                x = f"({x}.as_bits())"
            body = f"{op_str}{x}"
            body = f'{rval} = ({body}).{target_cast_str}'

        elif isinstance(expr, Log):
            formatter_str = expr.operands[0].value

            arg_print_snippets = []
            condition_snippets = []
            module_name = namify(self.current_module.name)

            for i in expr.operands[1:]:
                operand = unwrap_operand(i)
                if not isinstance(operand, Const):
                    self.expose('expr', operand)
                    exposed_name = self.dump_rval(operand, True)
                    valid_signal = f'dut.{module_name}.valid_{exposed_name}.value'
                    condition_snippets.append(valid_signal)

                    base_value = f"dut.{module_name}.expose_{exposed_name}.value"
                    if isinstance(operand.dtype, Int):
                        bits = operand.dtype.bits
                        expose_signal = (
                            f"({base_value} - (1 << {bits}) "
                            f"if ({base_value} >> ({bits} - 1)) & 1 else int({base_value}))"
                        )
                    else:
                        expose_signal = f"int({base_value})"
                    arg_print_snippets.append(expose_signal)
                else:
                    arg_print_snippets.append(str(operand.value))
            f_string_content_parts = []
            arg_iterator = iter(arg_print_snippets)

            for literal_text, field_name, format_spec, conversion \
                in Formatter().parse(formatter_str):

                if literal_text:
                    f_string_content_parts.append(literal_text)

                if field_name is not None:
                    if format_spec == '?':
                        conversion = 'r'
                        format_spec = None
                    arg_code = next(arg_iterator)
                    new_placeholder =  f"{{{arg_code}"
                    if conversion:  # for !s, !r, !a
                        new_placeholder += f"!{conversion}"
                    if format_spec:  # for :b, :08x,
                        new_placeholder += f":{format_spec}"
                    new_placeholder += "}"
                    f_string_content_parts.append(new_placeholder)

            f_string_content = "".join(f_string_content_parts)

            block_condition = self.get_pred()
            block_condition=block_condition.replace('cycle_count','dut.global_cycle_count')
            final_conditions = []

            for cond_str, cond_obj in self.cond_stack:
                if isinstance(cond_obj, CycledBlock):
                    tb_cond_path = \
                    cond_str.replace("self.cycle_count", "dut.global_cycle_count.value")
                    final_conditions.append(tb_cond_path)

                elif isinstance(cond_obj, CondBlock):
                    exposed_name = self.dump_rval(cond_obj.cond, True)

                    tb_expose_path = f"(dut.{module_name}.expose_{exposed_name}.value)"
                    tb_valid_path = f"(dut.{module_name}.valid_{exposed_name}.value)"

                    combined_cond = f"({tb_valid_path} & {tb_expose_path})"
                    final_conditions.append(combined_cond)

            if condition_snippets:
                final_conditions.append(" and ".join(condition_snippets))

            if_condition = " and ".join(final_conditions)

            self.logs.append(f'# {expr}')

            line_info = f"@line:{expr.loc.rsplit(':', 1)[-1]}"

            module_info = f"[{namify(self.current_module.name)}]"

            cycle_info = f"Cycle @{{float(dut.global_cycle_count.value):.2f}}:"# pylint: disable=W1309

            final_print_string = (
                 f'f"{line_info} {cycle_info} {module_info:<20} {f_string_content}"'
             )

            self.logs.append(f'#@ line {expr.loc}: {expr}')
            if if_condition:
                self.logs.append(f'if ( {if_condition} ):')
                self.logs.append(f'    print({final_print_string})')
            else:
                self.logs.append(f'print({final_print_string})')

        elif isinstance(expr, ArrayRead):
            array_ref = expr.array
            array_idx = unwrap_operand(expr.idx)
            array_idx = (self.dump_rval(array_idx, False)
                         if not isinstance(array_idx, Const) else array_idx.value)

            if dump_type(expr.idx.dtype)!=Bits and not isinstance(array_idx, int):
                array_idx = f"{array_idx}.as_bits()"

            array_name = self.dump_rval(array_ref, False)
            if isinstance(expr.dtype, Record):
                body = f'{rval} = self.{array_name}_q_in[{array_idx}]'
            else:
                body = f'{rval} = self.{array_name}_q_in[{array_idx}].{dump_type_cast(expr.dtype)}'
            self.expose('array', expr)
        elif isinstance(expr, ArrayWrite):
            self.expose('array', expr)
        elif isinstance(expr, FIFOPush):
            self.expose('fifo', expr)
        elif isinstance(expr,FIFOPop):
            rval = namify(expr.as_operand())
            fifo_name = self.dump_rval( expr.fifo, False)
            body = f'{rval} = self.{fifo_name}'
            self.expose('fifo_pop', expr)

        elif isinstance(expr, PureIntrinsic):
            intrinsic = expr.opcode
            if intrinsic in [PureIntrinsic.FIFO_VALID, PureIntrinsic.FIFO_PEEK]:
                fifo = expr.args[0]
                fifo_name = self.dump_rval(fifo, False)
                if intrinsic == PureIntrinsic.FIFO_PEEK:
                    body = f'{rval} = self.{fifo_name}'
                    self.expose('expr', expr)
                elif intrinsic == PureIntrinsic.FIFO_VALID:
                    body = f'{rval} = self.{fifo_name}_valid'
            elif intrinsic == PureIntrinsic.VALUE_VALID:
                value_expr = expr.operands[0].value
                if value_expr.parent.module != expr.parent.module:
                    port_name = self.get_external_port_name(value_expr)
                    body = f"{rval} = self.{port_name}_valid"
                else:
                    body = f"{rval} = self.executed"
            else:
                raise ValueError(f"Unknown intrinsic: {expr}")
        elif isinstance(expr, AsyncCall):
            self.expose('trigger', expr)
        elif isinstance(expr, Slice):
            a = self.dump_rval(expr.x, False)
            l = expr.l.value.value
            r = expr.r.value.value
            body = f"{rval} = {a}.as_bits()[{l}:{r+1}]"
        elif isinstance(expr, Concat):
            a = self.dump_rval(expr.msb, False)
            b = self.dump_rval(expr.lsb, False)
            body = f"{rval} = BitsSignal.concat([{a}.as_bits(), {b}.as_bits()])"


        elif isinstance(expr, Cast):
            dbits = expr.dtype.bits
            a = self.dump_rval(expr.x, False)
            src_dtype = expr.x.dtype
            pad = dbits - src_dtype.bits
            cast_body = ""
            cast_kind =  expr.opcode
            if cast_kind == Cast.BITCAST:
                # assert pad == 0
                cast_body = f"{a}.{dump_type_cast(expr.dtype,dbits)}"
            elif cast_kind == Cast.ZEXT:
                cast_body = (
                    f" BitsSignal.concat( [Bits({pad})(0) , {a}.as_bits()])"
                    f".{dump_type_cast(expr.dtype)} "
                )
            elif cast_kind == Cast.SEXT:
                cast_body =  (
                    f"BitsSignal.concat( [BitsSignal.concat([ {a}.as_bits()[{src_dtype.bits-1}] ]"
                    f" * {pad}) , {a}.as_bits()]).{dump_type_cast(expr.dtype)}"
                )
            body = f"{rval} = {cast_body}"
        elif isinstance(expr, Select):
            cond = self.dump_rval(expr.cond, False)
            true_value = self.dump_rval(expr.true_value, False)

            false_value = self.dump_rval(expr.false_value, False)
            if expr.true_value.dtype != expr.false_value.dtype:
                false_value =  f"{false_value}.{dump_type_cast(expr.true_value)}"
            body = f'{rval} = Mux({cond}, {false_value}, {true_value})'
        elif isinstance(expr, Bind):
            body = None
        elif isinstance(expr, Select1Hot):
            rval = self.dump_rval(expr, False)
            cond = self.dump_rval(expr.cond, False)
            values = [self.dump_rval(v, False) for v in expr.values]

            if len(values) == 1:
                body = f"{rval} = {values[0]}"
            else:
                selector_body = expr.parent._body # pylint: disable=W0212
                for expr_tmp in selector_body:
                    if self.dump_rval(expr_tmp, False)==cond:
                        b = self.dump_rval(expr_tmp.rhs, False)
                        break

                binary_selector_name = f"{rval}_selector"

                selector_bits = (len(values) - 1).bit_length()

                encoder_code = (
                    f"{binary_selector_name} = "
                    f"{b}.as_bits({selector_bits})"
                )
                self.append_code(encoder_code)
                values_str = ", ".join(values)
                mux_code = f"{rval} = Mux({binary_selector_name}, {values_str})"
                self.append_code(mux_code)

                body = None

        elif isinstance(expr, Intrinsic):
            intrinsic = expr.opcode
            if intrinsic == Intrinsic.FINISH:
                predicate_signal = self.get_pred()
                verilog_template = """
`ifndef SYNTHESIS
  always_ff @(posedge clk)
    // Finish if the execution path is active AND the specific finish condition is met.
    if ({{0}} & {{1}}) $finish;
`endif
"""
                self.finish_body = (f"sv.VerbatimOp({verilog_template!r}, "
                        f"substitutions=[{predicate_signal}.value, executed_wire.value])")
                body = None
            elif intrinsic == Intrinsic.ASSERT:
                self.expose('expr', expr.args[0])
            elif intrinsic == Intrinsic.WAIT_UNTIL:
                cond = self.dump_rval(expr.args[0], False)
                is_async_callee = self.current_module in self.async_callees

                final_cond = cond
                if is_async_callee:
                    final_cond = f"({cond}.as_bits() & self.trigger_counter_pop_valid)"

                self.wait_until = final_cond
            elif intrinsic == Intrinsic.BARRIER:
                body = None
            elif intrinsic == Intrinsic.MEM_WRITE:
            # Create a temporary ArrayWrite to reuse existing logic
                array = unwrap_operand(expr.args[0])
                idx = unwrap_operand(expr.args[1])
                val = unwrap_operand(expr.args[2])
                temp_write = ArrayWrite(array, idx, val)
                temp_write.parent = expr.parent
                self.expose('array', temp_write)
                body = None

            elif intrinsic == Intrinsic.MEM_READ:
                # Create a temporary ArrayRead to reuse existing logic
                array = unwrap_operand(expr.args[0])
                idx = unwrap_operand(expr.args[1])

                temp_read = ArrayRead(array, idx)
                temp_read.parent = expr.parent
                temp_read.scalar_ty = array.scalar_ty
                self.expose('array', temp_read)
                body = None

            else:
                raise ValueError(f"Unknown block intrinsic: {expr}")
        else:
            raise ValueError(f"Unhandled expression type: {type(expr).__name__}")

        if expr.is_valued() and expr_externally_used(expr, True):
            self.expose('expr', expr)

        if body is not None:
            self.append_code(body)

    def cleanup_post_generation(self):# pylint: disable=too-many-locals,too-many-branches,too-many-statements
        """genearting signals for connecting modules"""
        self.append_code('')

        exec_conditions = []
        if isinstance(self.current_module, Downstream):
            node = self.current_module
            if self.current_module in self.downstream_dependencies:
                dep_signals = [f'self.{namify(dep.name)}_executed' \
                    for dep in self.downstream_dependencies[node]]
                if dep_signals:
                    self.append_code(f"executed_wire = ({' | '.join(dep_signals)})")
                else:
                    self.append_code('executed_wire = Bits(1)(0)')
            else:
                self.append_code('executed_wire = Bits(1)(0)')
        else:
            exec_conditions.append("self.trigger_counter_pop_valid")
            if self.wait_until:
                exec_conditions.append(f"({self.wait_until})")

            if not exec_conditions:
                self.append_code('executed_wire = Bits(1)(1)')
            else:
                self.append_code(f"executed_wire = {' & '.join(exec_conditions)}")

        if self.finish_body:
            self.append_code(self.finish_body)

        for key, exposes in self._exposes.items():
            if isinstance(key, Array):
                writes = [(e, p) for e, p in exposes if isinstance(e, ArrayWrite)]
                if not writes:
                    continue
                array_name = self.dump_rval(key, False)
                array_dtype = key.scalar_ty

                ce_terms = [p for _, p in writes]
                self.append_code(f'self.{array_name}_w = executed_wire & ({" | ".join(ce_terms)})')

                write_0 = f'{self.dump_rval(writes[0][0].val, False)}'
                if writes[0][0].val.dtype != dump_type(array_dtype):
                    write_0 = f"{write_0}.{dump_type_cast(array_dtype)}"
                wdata_mux = f"Mux({writes[0][1]}, {dump_type(array_dtype)}(0),{write_0} )"
                for expr, pred in writes[1:]:
                    write_0 = f'{self.dump_rval(expr.val, False)}'
                    if expr.val.dtype != dump_type(array_dtype):
                        write_0 = f"{write_0}.{dump_type_cast(array_dtype)}"
                    wdata_mux = f"Mux({pred}, {wdata_mux},{write_0})"
                self.append_code(f'self.{array_name}_wdata = {wdata_mux}')

                widx_mux = (
                    f"Mux({writes[0][1]}, {dump_type(writes[0][0].idx.dtype)}(0),"
                    f" {self.dump_rval(writes[0][0].idx, False)})"
                )
                for expr, pred in writes[1:]:
                    widx_mux = f"Mux({pred},  {widx_mux},{self.dump_rval(expr.idx, False)})"
                self.append_code(f'self.{array_name}_widx = {widx_mux}')

            elif isinstance(key, Port):
                has_push = any(isinstance(e, FIFOPush) for e, p in exposes)
                has_pop = any(isinstance(e, FIFOPop) for e, p in exposes)

                if has_push:
                    fifo = self.dump_rval(key, False)
                    pushes = [(e, p) for e, p in exposes if isinstance(e, FIFOPush)]

                    final_push_predicate = " | ".join([f"({p})" for _, p in pushes]) \
                    if pushes else "Bits(1)(0)"

                    if len(pushes) == 1:
                        final_push_data = self.dump_rval(pushes[0][0].val, False)
                    else:
                        mux_data = f"{dump_type(key.dtype)}(0)"
                        for expr, pred in pushes:
                            rval = self.dump_rval(expr.val, False)
                            mux_data = f"Mux({pred}, {mux_data}, {rval})"
                        final_push_data = mux_data

                    self.append_code(f'# Push logic for port: {fifo}')
                    ready_signal = f"self.fifo_{namify(key.module.name)}_{fifo}_push_ready"

                    fifo_prefix = f"self.{namify(key.module.name)}_{fifo}"

                    self.append_code(
                        f"{fifo_prefix}_push_valid = executed_wire & "
                        f"({final_push_predicate}) & {ready_signal}"
                    )
                    self.append_code(f"{fifo_prefix}_push_data = {final_push_data}")

                if has_pop:
                    fifo = self.dump_rval(key, False)

                    pop_expr = [e for e, p in exposes if isinstance(e, FIFOPop)][0]
                    self.append_code(f'# {pop_expr}')
                    pop_predicates = [pred for expr, pred in exposes if isinstance(expr, FIFOPop)]

                    if pop_predicates:
                        final_pop_condition = " | ".join([f"({p})" for p in pop_predicates])
                    else:
                        final_pop_condition = "Bits(1)(0)"
                    self.append_code(
                        f"self.{fifo}_pop_ready = executed_wire & ({final_pop_condition})"
                    )

            elif isinstance(key, Module):
                rval = self.dump_rval(key, False)

                call_predicates = [pred for expr, pred in exposes if isinstance(expr, AsyncCall)]

                if not call_predicates:
                    self.append_code(f'self.{rval}_trigger = UInt(8)(0)')
                    continue

                self.append_code(f'# Summing triggers for {rval}')

                add_terms = [f"Mux({pred}, UInt(8)(0), UInt(8)(1))" for pred in call_predicates]

                if len(add_terms) == 1:
                    sum_expression = add_terms[0]
                else:
                    sum_expression = f"({' + '.join(add_terms)})"

                resized_sum = f"(({sum_expression}).as_bits()[0:8].as_uint())"
                final_trigger_value = f"Mux(executed_wire, UInt(8)(0), {resized_sum})"
                self.append_code(f'self.{rval}_trigger = {final_trigger_value}')

            else:
                expr, pred = exposes[0]
                rval = self.dump_rval(expr, False)
                exposed_name = self.dump_rval(expr, True)
                if not isinstance(key,ArrayWrite ):
                    dtype_str = dump_type(expr.dtype)
                else :
                    dtype_str = dump_type(expr.x.dtype)

                # Add port declaration strings to our list
                self.exposed_ports_to_add.append(f'expose_{exposed_name} = Output({dtype_str})')
                self.exposed_ports_to_add.append(f'valid_{exposed_name} = Output(Bits(1))')

                # Generate the logic assignment
                self.append_code(f'# Expose: {expr}')
                self.append_code(f'self.expose_{exposed_name} = {rval}')
                self.append_code(f'self.valid_{exposed_name} = executed_wire')
        self.append_code('self.executed = executed_wire')


    def visit_module(self, node: Module):# pylint: disable=too-many-locals,too-many-branches,too-many-statements
        # STAGE 1: ANALYSIS & BODY GENERATION
        original_code_buffer = self.code
        original_indent = self.indent
        self.code = []
        self.indent = original_indent + 8

        self.wait_until = None
        self._exposes = {}
        self.cond_stack = []
        self.current_module = node
        self.exposed_ports_to_add = []
        self.finish_body = None

        self.visit_block(node.body)
        self.cleanup_post_generation()

        construct_method_body = self.code

        self.code = original_code_buffer
        self.indent = original_indent

        self.current_module = node

        is_downstream = isinstance(node, Downstream)
        is_driver = node not in self.async_callees

        self.append_code(f'class {namify(node.name)}(Module):')
        self.indent += 4

        self.append_code('clk = Clock()')
        self.append_code('rst = Reset()')
        self.append_code('executed = Output(Bits(1))')
        self.append_code('cycle_count = Input(UInt(64))')

        if is_downstream:
            if node in self.downstream_dependencies:
                for dep_mod in self.downstream_dependencies[node]:
                    self.append_code(f'{namify(dep_mod.name)}_executed = Input(Bits(1))')
            for ext_val in node.externals:
                port_name = namify(ext_val.as_operand())
                if port_name.startswith("_"):
                    port_name = f"port{port_name}"
                port_type = dump_type(ext_val.dtype)
                self.append_code(f'{port_name} = Input({port_type})')
                self.append_code(f'{port_name}_valid = Input(Bits(1))')

        elif is_driver or node in self.async_callees:
            self.append_code('trigger_counter_pop_valid = Input(Bits(1))')

        if not is_downstream:
            for i in node.ports:
                name = namify(i.name)
                self.append_code(f'{name} = Input({dump_type(i.dtype)})')
                self.append_code(f'{name}_valid = Input(Bits(1))')
                has_pop = any(isinstance(e, FIFOPop) and e.fifo == i \
                              for e in self._walk_expressions(node.body))
                if has_pop:
                    self.append_code(f'{name}_pop_ready = Output(Bits(1))')

        pushes = [e for e in self._walk_expressions(node.body) if isinstance(e, FIFOPush)]
        calls = [e for e in self._walk_expressions(node.body) if isinstance(e, AsyncCall)]

        unique_push_handshake_targets = {(p.fifo.module, p.fifo.name) for p in pushes}
        unique_call_handshake_targets = {c.bind.callee for c in calls}
        unique_output_push_ports = {p.fifo for p in pushes}

        for module, fifo_name in unique_push_handshake_targets:
            port_name = f'fifo_{namify(module.name)}_{namify(fifo_name)}_push_ready'
            self.append_code(f'{port_name} = Input(Bits(1))')
        for callee in unique_call_handshake_targets:
            port_name = f'{namify(callee.name)}_trigger_counter_delta_ready'
            self.append_code(f'{port_name} = Input(Bits(1))')

        for fifo_port in unique_output_push_ports:
            port_prefix = f"{namify(fifo_port.module.name)}_{namify(fifo_port.name)}"
            self.append_code(f'{port_prefix}_push_valid = Output(Bits(1))')
            dtype = fifo_port.dtype
            self.append_code(f'{port_prefix}_push_data = Output({dump_type(dtype)})')
        for callee in unique_call_handshake_targets:
            self.append_code(f'{namify(callee.name)}_trigger = Output(UInt(8))')

        for arr_container in self.sys.arrays:
            for arr in arr_container.partition:
                if node in self.array_users.get(arr, []):
                    self.append_code(
                        f"{namify(arr.name)}_q_in = "
                        f"Input(dim({dump_type(arr.scalar_ty)}, {arr.size}))"
                    )
                    if any(isinstance(e, ArrayWrite) and e.array == arr \
                            for e in self._walk_expressions(node.body)):

                        self.append_code(f'{namify(arr.name)}_w = Output(Bits(1))')
                        self.append_code(
                            f'{namify(arr.name)}_wdata = Output({dump_type(arr.scalar_ty)})'
                            )

                        idx_type = next(e.idx.dtype for e in self._walk_expressions(node.body) \
                                        if isinstance(e, ArrayWrite) and e.array == arr)
                        self.append_code(
                            f'{namify(arr.name)}_widx = Output({dump_type(idx_type)})'
                        )

        for port_code in self.exposed_ports_to_add:
            self.append_code(port_code)

        self.append_code('')
        self.append_code('@generator')
        self.append_code('def construct(self):')

        self.code.extend(construct_method_body)

        self.indent -= 4
        self.append_code('')

    def _walk_expressions(self, block: Block):
        """Recursively walks a block and yields all expressions."""
        for item in block.body:
            if isinstance(item, Expr):
                yield item
            elif isinstance(item, Block):
                yield from self._walk_expressions(item)

    def visit_system(self, node: SysBuilder):# pylint: disable=too-many-locals,R0912
        sys = node
        self.sys = sys

        for arr_container in sys.arrays:
            for arr in arr_container.partition:
                self.visit_array(arr)

        expr_to_module = {}
        for module in sys.modules + sys.downstreams:
            for expr in self._walk_expressions(module.body):
                if expr.is_valued():
                    expr_to_module[expr] = module

        for ds_module in sys.downstreams:
            self.downstream_dependencies[ds_module] = []
            deps = set()
            for expr in self._walk_expressions(ds_module.body):
                # An operand is a dependency if it's an Expr defined in another module.
                for operand in expr.operands:
                    op = unwrap_operand(operand)
                    if isinstance(op, Expr) and op in expr_to_module:
                        producer_module = expr_to_module[op]
                        if producer_module != ds_module:
                            deps.add(producer_module)
            self.downstream_dependencies[ds_module] = list(deps)

        for module in sys.modules:
            for expr in self._walk_expressions(module.body):
                if isinstance(expr, AsyncCall):
                    callee = expr.bind.callee
                    if callee not in self.async_callees:
                        self.async_callees[callee] = []

                    if module not in self.async_callees[callee]:
                        self.async_callees[callee].append(module)

        self.array_users = {}
        for arr_container in self.sys.arrays:# pylint: disable=R1702
            for arr in arr_container.partition:
                self.array_users[arr] = []
                for mod in self.sys.modules + self.sys.downstreams:
                    for expr in self._walk_expressions(mod.body):
                        if isinstance(expr, (ArrayRead, ArrayWrite)) and expr.array == arr:
                            if mod not in self.array_users[arr]:
                                self.array_users[arr].append(mod)

        for elem in sys.modules:
            self.current_module = elem
            self.visit_module(elem)
        self.current_module = None
        for elem in sys.downstreams:
            self.visit_module(elem)
        self.is_top_generation = True
        self._generate_top_harness()
        self.is_top_generation = False


    def visit_array(self, node: Array):
        """Generates a PyCDE Module to encapsulate an array and its write logic."""
        array = node
        size = array.size
        dtype = array.scalar_ty
        index_bits = array.index_bits if array.index_bits > 0 else 1

        dim_type = f"dim({dump_type(dtype)} , {size})"

        class_name = namify(array.name)
        self.append_code(f'class {class_name}(Module):')
        self.indent += 4
        self.append_code('clk = Clock()')
        self.append_code('rst = Reset()')
        self.append_code('')
        self.append_code('w_ins = Input(Bits(1))')
        self.append_code(f'widx_ins = Input(Bits({index_bits}))')
        self.append_code(f'wdata_ins = Input({dump_type(dtype)})')
        self.append_code('')
        self.append_code(f'q_out = Output({dim_type})')
        self.append_code('')
        self.append_code('@generator')
        self.append_code('def construct(self):')
        self.indent += 4

        initializer = array.initializer
        if initializer is not None:
            rst_value_str = str(initializer)
        else:
            rst_value_str = f"[0] * {size}"

        self.append_code(
            f'data_reg = Reg({dim_type}, '
            f'clk=self.clk, rst=self.rst, rst_value={rst_value_str})'
            )
        self.append_code('')

        self.append_code(
            f"next_data_values = "
            f"[ Mux(self.widx_ins == Bits({index_bits})(i), data_reg[i],self.wdata_ins)"
            f" for i in range({size}) ]"
        )
        self.append_code(f'next_data_values =  {dim_type}(next_data_values)')
        self.append_code('next_data = Mux(self.w_ins,data_reg,next_data_values)')
        self.append_code('data_reg.assign(next_data)')
        self.append_code('self.q_out = data_reg')

        self.indent -= 8
        self.append_code('')

    def _generate_top_harness(self):# pylint: disable=too-many-locals,too-many-branches,too-many-statements
        """
        Generates a generic Top-level harness that connects all modules based on
        the analyzed dependencies (async calls, array usage).
        """

        self.append_code('class Top(Module):')
        self.indent += 4
        self.append_code('clk = Clock()')
        self.append_code('rst = Reset()')
        self.append_code('global_cycle_count = Output(UInt(64))')
        self.append_code('')
        self.append_code('@generator')
        self.append_code('def construct(self):')
        self.indent += 4

        self.append_code('\n# --- Global Cycle Counter ---')
        self.append_code('# A free-running counter for testbench control')

        self.append_code('cycle_count = Reg(UInt(64), clk=self.clk, rst=self.rst, rst_value=0)')
        self.append_code(
            'cycle_count.assign( (cycle_count + UInt(64)(1)).as_bits()[0:64].as_uint() )'
            )
        self.append_code('self.global_cycle_count = cycle_count')
        # --- 1. Wire Declarations (Generic) ---
        self.append_code('# --- Wires for FIFOs, Triggers, and Arrays ---')
        for module in self.sys.modules:
            for port in module.ports:
                fifo_base_name = f'fifo_{namify(module.name)}_{namify(port.name)}'
                self.append_code(f'# Wires for FIFO connected to {module.name}.{port.name}')
                self.append_code(f'{fifo_base_name}_push_valid = Wire(Bits(1))')
                self.append_code(f'{fifo_base_name}_push_data = Wire(Bits({port.dtype.bits}))')
                self.append_code(f'{fifo_base_name}_push_ready = Wire(Bits(1))')
                self.append_code(f'{fifo_base_name}_pop_valid = Wire(Bits(1))')
                self.append_code(f'{fifo_base_name}_pop_data = Wire(Bits({port.dtype.bits}))')
                self.append_code(f'{fifo_base_name}_pop_ready = Wire(Bits(1))')

        # Wires for TriggerCounters (one per module)
        for module in self.sys.modules:
            tc_base_name = f'{namify(module.name)}_trigger_counter'
            self.append_code(f'# Wires for {module.name}\'s TriggerCounter')
            self.append_code(f'{tc_base_name}_delta = Wire(Bits(8))')
            self.append_code(f'{tc_base_name}_delta_ready = Wire(Bits(1))')
            self.append_code(f'{tc_base_name}_pop_valid = Wire(Bits(1))')
            self.append_code(f'{tc_base_name}_pop_ready = Wire(Bits(1))')

        for arr_container in self.sys.arrays:
            for arr in arr_container.partition:
                arr_name = namify(arr.name)
                index_bits = arr.index_bits if arr.index_bits > 0 else 1
                self.append_code(f'# Wires for {arr_name}')
                self.append_code(f'aw_{arr_name}_w_ins = Wire(Bits(1))')
                self.append_code(f'aw_{arr_name}_wdata_ins = Wire({dump_type(arr.scalar_ty)})')
                self.append_code(f'aw_{arr_name}_widx_ins = Wire(Bits({index_bits}))')

                self.append_code(f'array_writer_{arr_name} = {arr_name}(')
                self.append_code('    clk=self.clk, rst=self.rst,')
                self.append_code(
                    f'    w_ins=aw_{arr_name}_w_ins,'
                    f' wdata_ins=aw_{arr_name}_wdata_ins, widx_ins=aw_{arr_name}_widx_ins)'
                )
                self.append_code('')

        # --- 2. Hardware Instantiations (Generic) ---
        self.append_code('\n# --- Hardware Instantiations ---')

        # Instantiate FIFOs
        for module in self.sys.modules:
            for port in module.ports:
                fifo_base_name = f'fifo_{namify(module.name)}_{namify(port.name)}'
                self.append_code(
                    f'{fifo_base_name}_inst = FIFO(WIDTH={port.dtype.bits}, DEPTH_LOG2=2)'
                    f'(clk=self.clk, rst_n=~self.rst, push_valid={fifo_base_name}_push_valid, '
                    f'push_data={fifo_base_name}_push_data, pop_ready={fifo_base_name}_pop_ready)'
                )

                self.append_code(
                    f'{fifo_base_name}_push_ready.assign({fifo_base_name}_inst.push_ready)'
                )
                self.append_code(
                    f'{fifo_base_name}_pop_valid.assign({fifo_base_name}_inst.pop_valid)'
                )
                self.append_code(
                    f'{fifo_base_name}_pop_data.assign({fifo_base_name}_inst.pop_data)'
                )

        # Instantiate TriggerCounters
        for module in self.sys.modules:
            tc_base_name = f'{namify(module.name)}_trigger_counter'
            self.append_code(
                f'{tc_base_name}_inst = TriggerCounter(WIDTH=8)'
                f'(clk=self.clk, rst_n=~self.rst, '
                f'delta={tc_base_name}_delta, pop_ready={tc_base_name}_pop_ready)'
            )
            self.append_code(
                f'{tc_base_name}_delta_ready.assign({tc_base_name}_inst.delta_ready)'
            )
            self.append_code(f'{tc_base_name}_pop_valid.assign({tc_base_name}_inst.pop_valid)')

        all_driven_fifo_ports = set()
        self.append_code('\n# --- Module Instantiations and Connections ---')
        for module in self.sys.modules + self.sys.downstreams:
            mod_name = namify(module.name)
            self.append_code(f'# Instantiation for {module.name}')

            port_map = ['clk=self.clk', 'rst=self.rst','cycle_count=cycle_count']

            is_downstream = isinstance(module, Downstream)

            if not is_downstream:
                port_map.append(f"trigger_counter_pop_valid={mod_name}_trigger_counter_pop_valid")
            else:
                if module in self.downstream_dependencies:
                    for dep_mod in self.downstream_dependencies[module]:
                        dep_name = namify(dep_mod.name)
                        port_map.append(f"{dep_name}_executed=inst_{dep_name}.executed")
                for ext_val in module.externals:
                    producer_module = ext_val.parent.module
                    port_name = namify(ext_val.as_operand())
                    if port_name.startswith("_"):
                        port_name = f"port{port_name}"
                    exposed_name = self.dump_rval(ext_val, True)

                    data_conn = \
                        f"{port_name}=inst_{namify(producer_module.name)}.expose_{exposed_name}"
                    valid_conn = (
                        f"{port_name}_valid="
                        f"inst_{namify(producer_module.name)}.valid_{exposed_name}"
                    )

                    port_map.append(data_conn)
                    port_map.append(valid_conn)


            for arr, users in self.array_users.items():
                if module in users:
                    port_map.append(
                        f"{namify(arr.name)}_q_in = array_writer_{namify(arr.name)}.q_out"
                    )

            pushes = [e for e in self._walk_expressions(module.body) if isinstance(e, FIFOPush)]
            calls = [e for e in self._walk_expressions(module.body) if isinstance(e, AsyncCall)]

            for p in pushes:
                # Store the actual Port object that is the target of a push
                all_driven_fifo_ports.add(p.fifo)

            unique_push_targets = {(p.fifo.module, p.fifo) for p in pushes}
            unique_call_targets = {c.bind.callee for c in calls}

            for (callee_mod, callee_port) in unique_push_targets:
                port_map.append(
                    f"fifo_{namify(callee_mod.name)}_{namify(callee_port.name)}_push_ready="
                    f"fifo_{namify(callee_mod.name)}_{namify(callee_port.name)}_push_ready"
                )
            for callee_mod in unique_call_targets:
                port_map.append(
                    f"{namify(callee_mod.name)}_trigger_counter_delta_ready="
                    f"{namify(callee_mod.name)}_trigger_counter_delta_ready"
                )

            if not is_downstream:
                for port in module.ports:
                    fifo_base_name = f'fifo_{mod_name}_{namify(port.name)}'
                    if isinstance(port.dtype, Record):
                        port_map.append(f"{namify(port.name)}={fifo_base_name}_pop_data")
                    else:
                        port_map.append(
                            f"{namify(port.name)}="
                            f"{fifo_base_name}_pop_data.{dump_type_cast(port.dtype)}"
                        )
                    port_map.append(f"{namify(port.name)}_valid={fifo_base_name}_pop_valid")

            self.append_code(f"inst_{mod_name} = {mod_name}({', '.join(port_map)})")

            if not is_downstream:
                self.append_code(
                    f"{mod_name}_trigger_counter_pop_ready.assign(inst_{mod_name}.executed)"
                    )
                for port in module.ports:
                    if any(isinstance(e, FIFOPop) and e.fifo == port \
                           for e in self._walk_expressions(module.body)):
                        self.append_code(
                            f"fifo_{mod_name}_{namify(port.name)}_pop_ready"
                            f".assign(inst_{mod_name}.{namify(port.name)}_pop_ready)"
                            )
            for (callee_mod, callee_port) in unique_push_targets:
                callee_mod_name = namify(callee_mod.name)
                callee_port_name = namify(callee_port.name)
                self.append_code(
                    f"fifo_{callee_mod_name}_{callee_port_name}_push_valid"
                    f".assign(inst_{mod_name}.{callee_mod_name}_{callee_port_name}_push_valid)"
                    )
                self.append_code(
                    f"fifo_{callee_mod_name}_{callee_port_name}_push_data"
                    f".assign(inst_{mod_name}.{callee_mod_name}_{callee_port_name}_push_data"
                    f".as_bits())"
                    )

        # self.append_code('\n# --- Tie off unused FIFO push ports ---')
        for module in self.sys.modules:
            for port in module.ports:
                if port not in all_driven_fifo_ports:
                    fifo_base_name = f'fifo_{namify(module.name)}_{namify(port.name)}'
                    self.append_code(f'{fifo_base_name}_push_valid.assign(Bits(1)(0))')
                    self.append_code(
                        f"{fifo_base_name}_push_data"
                        f".assign(Bits({port.dtype.bits})(0))"
                        )
        self.append_code('\n# --- Array Write-Back Connections ---')
        for arr_container in self.sys.arrays:
            for arr in arr_container.partition:
                arr_name = namify(arr.name)
                users = self.array_users.get(arr, [])
                writers = [m for m in users \
                           if any(isinstance(e, ArrayWrite) and e.array == arr \
                                            for e in self._walk_expressions(m.body))]

                if len(writers) == 1:
                    # Single writer: direct connection
                    writer_mod_name = namify(writers[0].name)
                    self.append_code(
                        f"aw_{arr_name}_w_ins.assign(inst_{writer_mod_name}.{arr_name}_w)"
                        )
                    self.append_code(
                        f"aw_{arr_name}_wdata_ins.assign(inst_{writer_mod_name}.{arr_name}_wdata)"
                        )
                    if arr.index_bits > 0:
                        self.append_code(
                            f"aw_{arr_name}_widx_ins"
                            f".assign(inst_{writer_mod_name}.{arr_name}_widx"
                            f".as_bits({arr.index_bits}))"
                            )
                    else:
                        self.append_code(f"aw_{arr_name}_widx_ins.assign(Bits(1)(0))")

                elif len(writers) > 1:
                    # Multiple writers: arbitration logic
                    self.append_code(f'# Arbitrating multiple writers for array {arr_name}')
                    w_terms = [f"inst_{namify(w.name)}.{arr_name}_w" for w in writers]
                    self.append_code(f"aw_{arr_name}_w_ins.assign({' | '.join(w_terms)})")

                    # WData Mux
                    wdata_mux = f"{dump_type(arr.scalar_ty)}(0)"
                    for writer in reversed(writers):
                        w_mod_name = namify(writer.name)
                        cond = f"inst_{w_mod_name}.{arr_name}_w"
                        true_val = f"inst_{w_mod_name}.{arr_name}_wdata"
                        wdata_mux = f"Mux({cond}, {wdata_mux}, {true_val})"
                    self.append_code(f"aw_{arr_name}_wdata_ins.assign({wdata_mux})")
                    # WIdx Mux
                    if arr.index_bits > 0:
                        widx_mux = f"Bits({arr.index_bits})(0)"
                        for writer in reversed(writers):
                            w_mod_name = namify(writer.name)
                            cond = f"inst_{w_mod_name}.{arr_name}_w"
                            true_val = f"inst_{w_mod_name}.{arr_name}_widx"
                            widx_mux = f"Mux({cond}, {widx_mux}, {true_val})"
                        self.append_code(f"aw_{arr_name}_widx_ins.assign({widx_mux})")
                    else:
                        self.append_code(f"aw_{arr_name}_widx_ins.assign(Bits(1)(0))")
                else:
                    self.append_code(f"aw_{arr_name}_w_ins.assign(Bits(1)(0))")
                    self.append_code(
                        f"aw_{arr_name}_wdata_ins.assign({dump_type(arr.scalar_ty)}(0))"
                        )
                    if arr.index_bits > 0:
                        self.append_code(
                            f"aw_{arr_name}_widx_ins.assign(Bits({arr.index_bits})(0))"
                            )
                    else:
                        self.append_code(f"aw_{arr_name}_widx_ins.assign(Bits(1)(0))")

        self.append_code('\n# --- Trigger Counter Delta Connections ---')
        for module in self.sys.modules:
            mod_name = namify(module.name)
            if module in self.async_callees:
                callers_of_this_module = self.async_callees[module]
                trigger_terms = [
                    f"inst_{namify(c.name)}.{mod_name}_trigger"
                    for c in callers_of_this_module
                ]
                if len(trigger_terms) > 1:
                    summed_triggers = f"({' + '.join(trigger_terms)})"
                else:
                    summed_triggers = trigger_terms[0]

                self.append_code(
                    f"{mod_name}_trigger_counter_delta.assign({summed_triggers}.as_bits(8))"
                    )
            else:
                self.append_code(f"{mod_name}_trigger_counter_delta.assign(Bits(8)(1))")

        self.indent -= 8
        self.append_code('')
        self.append_code('system = System([Top], name="Top", output_directory="sv")')
        self.append_code('system.compile()')

def generate_design(fname: str, sys: SysBuilder):
    """Generate a complete Verilog design file for the system."""
    with open(fname, 'w', encoding='utf-8') as fd:
        fd.write(HEADER)
        dumper = CIRCTDumper()
        dumper.visit_system(sys)
        code = '\n'.join(dumper.code)
        fd.write(code)
    logs = dumper.logs
    return logs
