# pylint: disable=C0302
# pylint: disable=no-member
"""Verilog design generation and code dumping."""

from typing import List, Dict, Tuple, Union
from collections import defaultdict
from pathlib import Path

from .utils import (
    HEADER,
    dump_type,
    extract_sram_params,
    ensure_bits,
)

from ...analysis import expr_externally_used
from ...ir.module import Module, Downstream
from ...ir.memory.sram import SRAM
from ...ir.module.external import ExternalSV
from ...builder import SysBuilder
from ...ir.visitor import Visitor
from ...ir.block import Block, CondBlock,CycledBlock
from ...ir.const import Const
from ...ir.array import Array
from ...ir.dtype import RecordValue
from ...utils import namify, unwrap_operand
from ...utils.enforce_type import enforce_type
from ...ir.expr import (
    Expr,
    FIFOPop,
    Log,
    ArrayRead,
    ArrayWrite,
    FIFOPush,
    AsyncCall,
    WireRead
)
from ._expr import codegen_expr
from .cleanup import cleanup_post_generation
from .rval import dump_rval as dump_rval_impl
from .module import generate_module_ports
from .system import generate_system


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
        self.array_read_port_mapping = {}
        self.array_read_ports = {}
        self.array_read_expr_port = {}
        self.sram_payload_arrays = set()
        self.memory_defs = set()
        self.expr_to_name = {}
        self.name_counters = defaultdict(int)
        # Track external module usage for downstream modules
        self.external_wire_assignments = []
        self.external_wire_assignment_keys = set()
        self.external_wire_outputs = {}
        self.pending_external_inputs = defaultdict(list)
        self.instantiated_external_modules = set()
        self.external_modules = []
        self.module_ctx = None

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

    @staticmethod
    def is_stub_external(module: Module) -> bool:
        """Return True if the module has no generated body and acts as a pure external stub."""
        if not CIRCTDumper._is_external_module(module):
            return False
        body = getattr(module, "body", None)
        body_insts = getattr(body, "body", []) if body is not None else []
        return not body_insts


    def dump_rval(self, node, with_namespace: bool, module_name: str = None) -> str:
        """Dump a reference to a node with options."""
        return dump_rval_impl(self, node, with_namespace, module_name)

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

        # Add location comment if available
        if hasattr(expr, 'loc') and expr.loc:
            self.append_code(f'#{expr.loc}')

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
        previous_module_ctx = self.module_ctx
        self.module_ctx = node
        self.exposed_ports_to_add = []
        self.finish_body = []
        self.finish_conditions = []
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

        pushes = [e for e in self._walk_expressions(node.body) if isinstance(e, FIFOPush)]
        calls = [e for e in self._walk_expressions(node.body) if isinstance(e, AsyncCall)]

        generate_module_ports(self, node, is_downstream, is_sram, is_driver, pushes, calls)

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
        self.module_ctx = previous_module_ctx

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
        """Visit a system and generate code for all modules."""
        generate_system(self, node)

    # pylint: disable=too-many-statements
    def visit_array(self, node: Array):
        """Generates a PyCDE Module to encapsulate an array and its write logic."""
        array = node
        size = array.size
        dtype = array.scalar_ty
        index_bits = array.index_bits
        index_bits_type = index_bits if index_bits > 0 else 1

        writers = list(array.get_write_ports().keys())
        num_write_ports = len(writers)
        read_ports = self.array_read_ports.get(array, [])
        num_read_ports = len(read_ports)

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
            self.append_code(f'widx{port_suffix} = Input(Bits({index_bits_type}))')
            self.append_code(f'wdata{port_suffix} = Input({dump_type(dtype)})')
            self.append_code('')

        for i in range(num_read_ports):
            port_suffix = f"_port{i}"
            if index_bits > 0:
                self.append_code(f'ridx{port_suffix} = Input(Bits({index_bits_type}))')
            self.append_code(f'rdata{port_suffix} = Output({dump_type(dtype)})')
            self.append_code('')

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
                    f'(self.widx{port_suffix} == Bits({index_bits_type})(i)))'
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

        for port_idx in range(num_read_ports):
            port_suffix = f"_port{port_idx}"
            if index_bits > 0:
                self.append_code(
                    f'self.rdata{port_suffix} = data_reg[self.ridx{port_suffix}]'
                )
            else:
                self.append_code(
                    f'self.rdata{port_suffix} = data_reg[0]'
                )

        self.indent -= 8
        self.append_code('')


    def _generate_external_module_wrapper(self, ext_module: ExternalSV):
        """Generate a PyCDE wrapper class for an external module."""
        class_name = f"{namify(ext_module.name)}_ffi"
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
        """Connect each array to its writers and readers."""
        arr_name = namify(arr.name)
        write_mapping = self.array_write_port_mapping.get(arr, {})
        read_mapping = self.array_read_port_mapping.get(arr, {})
        if not write_mapping and not read_mapping:
            return

        self.append_code(f'# Connections for array {arr_name}')

        # Connect writer modules to their dedicated ports.
        for module, port_idx in write_mapping.items():
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

        # Connect read address signals from modules into the array writer.
        if arr.index_bits > 0:
            for module, port_indices in read_mapping.items():
                module_name = namify(module.name)
                for port_idx in port_indices:
                    port_suffix = f"_port{port_idx}"
                    self.append_code(
                        f'aw_{arr_name}_ridx{port_suffix}.assign('
                        f'inst_{module_name}.{arr_name}_ridx{port_suffix})'
                    )


@enforce_type
def generate_design(fname: Union[str, Path], sys: SysBuilder) -> None:
    """Generate a complete Verilog design file for the system."""
    with open(str(fname), 'w', encoding='utf-8') as fd:
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
        code = code.replace('system.compile()")', 'system.compile()')
        fd.write(code)
    logs = dumper.logs
    return logs
