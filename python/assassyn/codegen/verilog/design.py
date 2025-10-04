# pylint: disable=C0302
# pylint: disable=no-member
"""Verilog design generation and code dumping."""

from typing import List, Dict, Tuple
from collections import defaultdict

from .utils import (
    HEADER,
    dump_type,
    get_sram_info,
    extract_sram_params,
    ensure_bits,
)

from ...analysis import expr_externally_used
from ...ir.module import Module, Downstream, Port,SRAM, Wire
from ...ir.module.external import ExternalSV
from ...builder import SysBuilder
from ...ir.visitor import Visitor
from ...ir.block import Block, CondBlock,CycledBlock
from ...ir.const import Const
from ...ir.array import Array
from ...ir.dtype import RecordValue
from ...utils import namify, unwrap_operand
from ...analysis import get_upstreams
from ...ir.expr import (
    Expr,
    FIFOPop,
    Log,
    ArrayRead,
    ArrayWrite,
    FIFOPush,
    AsyncCall,
    Bind,
    WireRead
)
from .expr import codegen_expr
from .top import generate_top_harness
from .cleanup import cleanup_post_generation


class CIRCTDumper(Visitor):  # pylint: disable=too-many-instance-attributes,too-many-statements
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
    finish_body:list[str]
    sram_payload_arrays:set
    memory_defs:set

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
        self.finish_body = []
        self.finish_conditions = []
        self.array_write_port_mapping = {}
        self.sram_payload_arrays = set()
        self.memory_defs = set()
        self.expr_to_name = {}
        self.name_counters = defaultdict(int)
        # Track external module usage for downstream modules
        self.external_wire_assignments = []
        self.pending_external_inputs = defaultdict(list)
        self.instantiated_external_modules = set()
        self.external_modules = []

    def get_pred(self) -> str:
        """Get the current predicate for conditional execution."""
        if not self.cond_stack:
            return "Bits(1)(1)"
        pred_parts = []
        for s, _ in self.cond_stack:
            s_bits = ensure_bits(s)
            pred_parts.append(s_bits)
        return " & ".join(pred_parts)

    def get_external_port_name(self, node: Expr) -> str:
        """Get the mangled port name for an external value."""
        producer_module = node.parent.module
        producer_name = namify(producer_module.name)
        base_port_name = namify(node.as_operand())
        if base_port_name.startswith("_"):
            base_port_name = f"port{base_port_name}"
        port_name = f"{producer_name}_{base_port_name}"
        return port_name

    # pylint: disable=protected-access
    @staticmethod
    def _is_external_module(module: Module) -> bool:
        """Return True if the module represents an external implementation."""

        if isinstance(module, ExternalSV):
            return True

        attrs = getattr(module, '_attrs', None)
        return attrs is not None and Module.ATTR_EXTERNAL in attrs


    # pylint: disable=too-many-return-statements,too-many-branches
    def dump_rval(self,node, with_namespace: bool,module_name:str=None) -> str:
        """Dump a reference to a node with options."""

        node = unwrap_operand(node)
        if (
            isinstance(node, Expr)
            and self.current_module is not None
            and hasattr(self.current_module, 'externals')
            and node in self.current_module.externals
            and not self.is_top_generation
        ):
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
            if node not in self.expr_to_name:
                base_name = namify(node.as_operand())
                # Handle anonymous expressions which namify to '_' or an empty string.
                if not base_name or base_name == '_':
                    base_name = 'tmp'

                count = self.name_counters[base_name]
                unique_name = f"{base_name}_{count}" if count > 0 else base_name
                self.name_counters[base_name] += 1
                self.expr_to_name[node] = unique_name

            unique_name = self.expr_to_name[node]

            if with_namespace:
                owner_module_name = namify(node.parent.module.name)
                if owner_module_name is None:
                    owner_module_name = module_name
                return f"{owner_module_name}_{unique_name}"
            return unique_name

        if isinstance(node, RecordValue):
            return self.dump_rval(node.value(), with_namespace, module_name)
        if isinstance(node, Wire):
            # For wires, we use their name directly
            return namify(node.name)

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
                if block.body is None:
                    return False
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

        if node is not None and node.body is not None:
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


    # pylint: disable=arguments-renamed
    def visit_expr(self, expr: Expr):
        self.append_code(f'# {expr}')

        # Delegate to the expression code generator
        body = codegen_expr(self, expr)

        # Handle exposure logic for valued expressions that are externally used
        if expr.is_valued() and not isinstance(expr, WireRead) \
                and expr_externally_used(expr, True):
            if not isinstance(unwrap_operand(expr), Const):
                self.expose('expr', expr)

        if body is not None:
            self.append_code(body)



    # pylint: disable=too-many-locals,too-many-branches,too-many-statements
    def visit_module(self, node: Module):
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
        self.finish_body = []
        self.finish_conditions = []
        self.external_wire_assignments = []
        self.pending_external_inputs.clear()
        self.instantiated_external_modules.clear()

        # For downstream modules, we still need to process the body
        if node.body is not None:
            self.visit_block(node.body)
        cleanup_post_generation(self)

        construct_method_body = self.code

        self.code = original_code_buffer
        self.indent = original_indent

        self.current_module = node

        is_downstream = isinstance(node, Downstream)
        is_sram = isinstance(node, SRAM)
        is_driver = node not in self.async_callees

        self.append_code(f'class {namify(node.name)}(Module):')
        self.indent += 4

        self.append_code('clk = Clock()')
        self.append_code('rst = Reset()')
        self.append_code('executed = Output(Bits(1))')
        self.append_code('cycle_count = Input(UInt(64))')
        self.append_code('finish = Output(Bits(1))')

        if is_downstream:
            if node in self.downstream_dependencies:
                for dep_mod in self.downstream_dependencies[node]:
                    self.append_code(f'{namify(dep_mod.name)}_executed = Input(Bits(1))')
            for ext_val in node.externals:
                if isinstance(ext_val,Bind) or isinstance(unwrap_operand(ext_val), Const):
                    continue
                port_name = self.get_external_port_name(ext_val)
                port_type = dump_type(ext_val.dtype)
                self.append_code(f'{port_name} = Input({port_type})')
                self.append_code(f'{port_name}_valid = Input(Bits(1))')
            if is_sram:
                sram_info = get_sram_info(node)
                if sram_info:
                    sram_array = sram_info['array']
                    self.append_code(f'mem_dataout = Input({dump_type(sram_array.scalar_ty)})')
                    index_bits = sram_array.index_bits if sram_array.index_bits > 0 else 1
                    self.append_code(f'mem_address = Output(Bits({index_bits}))')
                    self.append_code(f'mem_write_data = Output({dump_type(sram_array.scalar_ty)})')
                    self.append_code('mem_write_enable = Output(Bits(1))')
                    self.append_code('mem_read_enable = Output(Bits(1))')

        elif is_driver or node in self.async_callees:
            self.append_code('trigger_counter_pop_valid = Input(Bits(1))')

        if not is_downstream and not self._is_external_module(node):
            for i in node.ports:
                name = namify(i.name)
                self.append_code(f'{name} = Input({dump_type(i.dtype)})')
                self.append_code(f'{name}_valid = Input(Bits(1))')
                has_pop = any(
                    isinstance(e, FIFOPop) and e.fifo == i
                    for e in self._walk_expressions(node.body)
                )
                if has_pop:
                    self.append_code(f'{name}_pop_ready = Output(Bits(1))')

        pushes = [e for e in self._walk_expressions(node.body) if isinstance(e, FIFOPush)]
        calls = [e for e in self._walk_expressions(node.body) if isinstance(e, AsyncCall)]

        unique_push_handshake_targets = {(p.fifo.module, p.fifo.name) for p in pushes}
        unique_call_handshake_targets = {c.bind.callee for c in calls}
        unique_output_push_ports = {p.fifo for p in pushes}

        # Skip external modules for handshake targets
        filtered_push_targets = set()
        for module, fifo_name in unique_push_handshake_targets:
            if not self._is_external_module(module):
                filtered_push_targets.add((module, fifo_name))

        filtered_call_targets = set()
        for callee in unique_call_handshake_targets:
            if not self._is_external_module(callee):
                filtered_call_targets.add(callee)

        for module, fifo_name in filtered_push_targets:
            port_name = f'fifo_{namify(module.name)}_{namify(fifo_name)}_push_ready'
            self.append_code(f'{port_name} = Input(Bits(1))')
        for callee in filtered_call_targets:
            port_name = f'{namify(callee.name)}_trigger_counter_delta_ready'
            self.append_code(f'{port_name} = Input(Bits(1))')

        # Skip external modules for output push ports
        filtered_output_push_ports = set()
        for fifo_port in unique_output_push_ports:
            if not self._is_external_module(fifo_port.module):
                filtered_output_push_ports.add(fifo_port)

        for fifo_port in filtered_output_push_ports:
            port_prefix = f"{namify(fifo_port.module.name)}_{namify(fifo_port.name)}"
            self.append_code(f'{port_prefix}_push_valid = Output(Bits(1))')
            dtype = fifo_port.dtype
            self.append_code(f'{port_prefix}_push_data = Output({dump_type(dtype)})')
        for callee in filtered_call_targets:
            self.append_code(f'{namify(callee.name)}_trigger = Output(UInt(8))')
        # pylint: disable=too-many-nested-blocks
        for arr_container in self.sys.arrays:
            arr = arr_container
            if is_sram:
                sram_info = get_sram_info(node)
                if sram_info and arr == sram_info['array']:
                    continue
            if node in self.array_users.get(arr, []):
                self.append_code(
                    f"{namify(arr.name)}_q_in = "
                    f"Input(dim({dump_type(arr.scalar_ty)}, {arr.size}))"
                )
                port_mapping = self.array_write_port_mapping.get(arr, {})
                for module_key, port_idx in port_mapping.items():
                    if module_key == node:
                        port_suffix = f"_port{port_idx}"
                        self.append_code( \
                            f'{namify(arr.name)}_w{port_suffix} = Output(Bits(1))')
                        self.append_code(
                            f'{namify(arr.name)}_wdata{port_suffix} ='
                            f' Output({dump_type(arr.scalar_ty)})'
                        )
                        idx_type = arr.index_bits if arr.index_bits > 0 else 1
                        self.append_code(
                            f'{namify(arr.name)}_widx{port_suffix} ='
                            f' Output(Bits({idx_type}))'
                        )


        for port_code in self.exposed_ports_to_add:
            self.append_code(port_code)

        self.append_code('')
        self.append_code('@generator')
        self.append_code('def construct(self):')

        if is_sram:
            self.indent += 4
            self.append_code('# SRAM dataout from memory')
            self.append_code('dataout = self.mem_dataout')
            self.code.extend(construct_method_body)
            self.indent -= 4
        else:
            self.code.extend(construct_method_body)
        self.indent -= 4
        self.append_code('')

    def _walk_expressions(self, block: Block):
        """Recursively walks a block and yields all expressions."""
        if block is None:
            return
        if block.body is None:
            return
        for item in block.body:
            if isinstance(item, Expr):
                yield item
            elif isinstance(item, Block):
                yield from self._walk_expressions(item)


    # pylint: disable=too-many-locals,R0912
    def visit_system(self, node: SysBuilder):
        sys = node
        self.sys = sys
        for module in sys.downstreams:
            if isinstance(module, SRAM) and hasattr(module, 'payload'):
                self.sram_payload_arrays.add(module.payload)

        # Collect external modules
        self.external_modules = []
        for module in sys.modules + sys.downstreams:
            if self._is_external_module(module):
                if module not in self.external_modules:
                    self.external_modules.append(module)
            # Also check for external modules used within downstream modules
            for expr in self._walk_expressions(module.body):
                if isinstance(expr, AsyncCall):
                    callee = expr.bind.callee
                    if self._is_external_module(callee):
                        if callee not in self.external_modules:
                            self.external_modules.append(callee)

        # Generate PyCDE wrapper classes for external modules first
        for ext_module in self.external_modules:
            self._generate_external_module_wrapper(ext_module)

        for arr_container in sys.arrays:
            if arr_container in self.sram_payload_arrays:
                continue
            sub_array = arr_container
            if sub_array not in self.array_write_port_mapping:
                self.array_write_port_mapping[sub_array] = {}
            sub_array_writers = sub_array.get_write_ports()
            for module, _ in sub_array_writers.items():
                if module not in self.array_write_port_mapping[sub_array]:
                    port_idx = len(self.array_write_port_mapping[sub_array])
                    self.array_write_port_mapping[sub_array][module] = port_idx

        for arr_container in sys.arrays:
            if arr_container not in self.sram_payload_arrays:
                self.visit_array(arr_container)

        expr_to_module = {}
        for module in sys.modules + sys.downstreams:
            for expr in self._walk_expressions(module.body):
                if expr.is_valued():
                    expr_to_module[expr] = module

        for ds_module in sys.downstreams:
            self.downstream_dependencies[ds_module] = get_upstreams(ds_module)

        all_modules = self.sys.modules + self.sys.downstreams
        for module in all_modules:
            for expr in self._walk_expressions(module.body):
                if isinstance(expr, AsyncCall):
                    callee = expr.bind.callee
                    if callee not in self.async_callees:
                        self.async_callees[callee] = []

                    if module not in self.async_callees[callee]:
                        self.async_callees[callee].append(module)

        self.array_users = {}
        # pylint: disable=R1702
        for arr_container in self.sys.arrays:
            if arr_container in self.sram_payload_arrays:
                continue
            arr = arr_container
            self.array_users[arr] = []
            for mod in self.sys.modules + self.sys.downstreams:
                if isinstance(mod, SRAM) and hasattr(mod, 'payload') and arr == mod.payload:
                    continue
                for expr in self._walk_expressions(mod.body):
                    if isinstance(expr, (ArrayRead, ArrayWrite)) and expr.array == arr:
                        if mod not in self.array_users[arr]:
                            self.array_users[arr].append(mod)

        # Process only non-external modules from sys.modules
        for elem in sys.modules:
            if self._is_external_module(elem):
                continue

            self.current_module = elem
            self.visit_module(elem)
        self.current_module = None
        for elem in sys.downstreams:
            self.current_module = elem
            self.visit_module(elem)
        self.current_module = None
        self.is_top_generation = True
        generate_top_harness(self)
        self.is_top_generation = False

    # pylint: disable=too-many-statements
    def visit_array(self, node: Array):
        """Generates a PyCDE Module to encapsulate an array and its write logic."""
        array = node
        size = array.size
        dtype = array.scalar_ty
        index_bits = array.index_bits if array.index_bits > 0 else 1

        writers = list(array.get_write_ports().keys())
        num_write_ports = len(writers)

        dim_type = f"dim({dump_type(dtype)}, {size})"
        class_name = namify(array.name)

        self.append_code(f'class {class_name}(Module):')
        self.indent += 4
        self.append_code('clk = Clock()')
        self.append_code('rst = Reset()')
        self.append_code('')

        for i in range(num_write_ports):
            port_suffix = f"_port{i}"
            self.append_code(f'w{port_suffix} = Input(Bits(1))')
            self.append_code(f'widx{port_suffix} = Input(Bits({index_bits}))')
            self.append_code(f'wdata{port_suffix} = Input({dump_type(dtype)})')
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
        if num_write_ports != 0:
            self.append_code('# Multi-port write logic')
            self.append_code('next_data_values = []')
            self.append_code(f'for i in range({size}):')
            self.indent += 4
            self.append_code('# Check each write port for this address')
            self.append_code('element_value = data_reg[i]')
            for port_idx in reversed(range(num_write_ports)):
                port_suffix = f"_port{port_idx}"
                self.append_code(
                    f'# Port {port_idx} write check'
                )
                self.append_code(
                    f'if_write_port{port_idx} = '
                    f'(self.w{port_suffix} & '
                    f'(self.widx{port_suffix} == Bits({index_bits})(i)))'
                )
                self.append_code(
                    f'element_value = Mux(if_write_port{port_idx}, '
                    f'element_value, self.wdata{port_suffix})'
                )
            self.append_code('next_data_values.append(element_value)')
            self.indent -= 4
            self.append_code(f'next_data = {dim_type}(next_data_values)')
        else:
            self.append_code('next_data = data_reg')
        self.append_code('data_reg.assign(next_data)')
        self.append_code('self.q_out = data_reg')

        self.indent -= 8
        self.append_code('')


    def _generate_external_module_wrapper(self, ext_module: ExternalSV):
        """Generate a PyCDE wrapper class for an external module."""
        class_name = namify(ext_module.name)
        module_name = getattr(ext_module, 'external_module_name', class_name)

        self.append_code(f'class {class_name}(Module):')
        self.indent += 4

        # Set the module name for PyCDE
        self.append_code(f'module_name = f"{module_name}"')
        if getattr(ext_module, 'has_clock', False):
            self.append_code('clk = Clock()')
        if getattr(ext_module, 'has_reset', False):
            self.append_code('rst = Reset()')

        # Check if the external module carries declared wires
        if hasattr(ext_module, '_wires') and ext_module._wires:
            # Handle wires with explicit directions
            for wire_name, wire in ext_module._wires.items():
                wire_type = dump_type(wire.dtype)
                if wire.direction == 'input':
                    self.append_code(f'{wire_name} = Input({wire_type})')
                elif wire.direction == 'output':
                    self.append_code(f'{wire_name} = Output({wire_type})')
                else:
                    # For undirected wires, default to Input (backward compatibility)
                    self.append_code(f'{wire_name} = Input({wire_type})')
        else:
            # Fallback to handling ports for backward compatibility
            for port in ext_module.ports:
                port_name = namify(port.name)
                port_type = dump_type(port.dtype)
                # For external modules, default all ports to Input for backward compatibility
                # Actual connections will be handled in the instantiation
                self.append_code(f'{port_name} = Input({port_type})')

        self.indent -= 4
        self.append_code('')

    def _connect_array(self, arr):
        """Connect each array to its writers"""
        arr_name = namify(arr.name)
        port_mapping = self.array_write_port_mapping.get(arr, {})
        if not port_mapping:
            return

        self.append_code(f'# Multi-port connections for {arr_name}')

        # Connect each module to its dedicated port
        for module, port_idx in port_mapping.items():
            module_name = namify(module.name)
            port_suffix = f"_port{port_idx}"

            self.append_code(
                f'aw_{arr_name}_w{port_suffix}.assign('
                f'inst_{module_name}.{arr_name}_w{port_suffix})'
            )
            self.append_code(
                f'aw_{arr_name}_wdata{port_suffix}.assign('
                f'inst_{module_name}.{arr_name}_wdata{port_suffix})'
            )
            if arr.index_bits > 0:
                self.append_code(
                    f'aw_{arr_name}_widx{port_suffix}.assign('
                    f'inst_{module_name}.{arr_name}_widx{port_suffix}'
                    f".as_bits({arr.index_bits}))"
                )
            else:
                self.append_code(
                    f'aw_{arr_name}_widx{port_suffix}.assign(Bits(1)(0))'
                )


def generate_design(fname: str, sys: SysBuilder):
    """Generate a complete Verilog design file for the system."""
    with open(fname, 'w', encoding='utf-8') as fd:
        fd.write(HEADER)

        dumper = CIRCTDumper()

        # Generate sramBlackbox module definitions for each SRAM
        sram_modules = [m for m in sys.downstreams if isinstance(m, SRAM)]
        if sram_modules:
            for sram in sram_modules:
                params = extract_sram_params(sram)
                array_name = params['array_name']
                data_width = params['data_width']
                addr_width = params['addr_width']
                dumper.memory_defs.add((data_width, addr_width, array_name))

            # Write sramBlackbox module definitions
            for data_width, addr_width, array_name in dumper.memory_defs:
                fd.write(f'''
@modparams
def sramBlackbox_{array_name}():
    class sramBlackboxImpl(Module):
        module_name = "sram_blackbox_{array_name}"
        clk = Clock()
        rst_n = Input(Bits(1))
        address = Input(Bits({addr_width}))
        wd = Input(Bits({data_width}))
        banksel = Input(Bits(1))
        read = Input(Bits(1))
        write = Input(Bits(1))
        dataout = Output(Bits({data_width}))
    return sramBlackboxImpl

''')
        dumper.visit_system(sys)
        code = '\n'.join(dumper.code)
        fd.write(code)
    logs = dumper.logs
    return logs
