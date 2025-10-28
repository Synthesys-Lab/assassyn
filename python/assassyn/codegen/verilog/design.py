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
from ...builder import SysBuilder
from ...ir.visitor import Visitor
from ...ir.const import Const
from ...ir.array import Array
from ...ir.dtype import RecordValue
from ...utils import namify, unwrap_operand
from ...utils.enforce_type import enforce_type
from ...ir.expr import (
    Expr,
    FIFOPop,
    ArrayRead,
    ArrayWrite,
    FIFOPush,
    AsyncCall,
)
from ...ir.expr.intrinsic import PureIntrinsic, ExternalIntrinsic
from ._expr import codegen_expr
from .cleanup import cleanup_post_generation
from .rval import dump_rval as dump_rval_impl
from .module import generate_module_ports
from .system import generate_system
from .metadata import PostDesignGeneration


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
        self.cross_module_external_reads = []
        self.external_outputs_by_instance = defaultdict(list)
        self.external_output_exposures = defaultdict(dict)
        self.module_ctx = None
        self.external_wrapper_names = {}
        self.external_instance_names = {}
        self.external_instance_owners = {}
        self.external_intrinsics = []
        self.external_classes = []
        self.module_metadata: Dict[Module, PostDesignGeneration] = {}

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
        producer_module = node.parent
        producer_name = namify(producer_module.name)
        base_port_name = namify(node.as_operand())
        if base_port_name.startswith("_"):
            base_port_name = f"port{base_port_name}"
        port_name = f"{producer_name}_{base_port_name}"
        return port_name

    def get_external_wire_key(self, instance, port_name, index_operand):
        """Create a stable key for cross-module external output wiring."""
        idx_key = None
        if index_operand is not None:
            idx_value = unwrap_operand(index_operand)
            if isinstance(idx_value, Const):
                idx_key = ('const', idx_value.value)
            else:
                idx_key = ('expr', idx_value)
        return (instance, port_name, idx_key)


    # pylint: disable=protected-access
    @staticmethod
    def _is_external_module(module: Module) -> bool:
        """Return True if the module represents an external implementation."""

        attrs = getattr(module, '_attrs', None)
        return attrs is not None and Module.ATTR_EXTERNAL in attrs

    @staticmethod
    def is_stub_external(module: Module) -> bool:
        """Return True if the module has no generated body and acts as a pure external stub."""
        if not CIRCTDumper._is_external_module(module):
            return False
        body = getattr(module, "body", None)
        body_insts = body if isinstance(body, list) else []
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

    def _visit_body(self, body_nodes):
        for node in body_nodes:
            if isinstance(node, Expr):
                self.visit_expr(node)
            elif isinstance(node, RecordValue):
                pass
            else:
                raise ValueError(f'Unknown node type: {type(node)}')


    # pylint: disable=arguments-renamed
    def visit_expr(self, expr: Expr):
        self.append_code(f'# {expr}')

        # Add location comment if available
        if hasattr(expr, 'loc') and expr.loc:
            self.append_code(f'#{expr.loc}')

        # Delegate to the expression code generator
        body = codegen_expr(self, expr)

        # Handle exposure logic for valued expressions that are externally used
        if expr.is_valued() and expr_externally_used(expr, True):
            # Skip exposure for ExternalIntrinsic - they should never be exposed as ports
            skip_exposure = isinstance(expr, ExternalIntrinsic)

            # For EXTERNAL_OUTPUT_READ, skip exposure only if the instance is in the same module
            if (
                isinstance(expr, PureIntrinsic)
                and expr.opcode == PureIntrinsic.EXTERNAL_OUTPUT_READ
            ):
                instance = expr.args[0]  # The ExternalIntrinsic
                instance_owner = self.external_instance_owners.get(instance)
                # Skip exposure if the instance is in the current module
                if instance_owner == self.current_module:
                    skip_exposure = True

            if not skip_exposure and not isinstance(unwrap_operand(expr), Const):
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

        # Initialize metadata for this module
        self.module_metadata[node] = PostDesignGeneration()

        self.wait_until = None
        self._exposes = {}
        self.cond_stack = []
        self.current_module = node
        previous_module_ctx = self.module_ctx
        self.module_ctx = node
        self.exposed_ports_to_add = []
        self.finish_body = []
        self.finish_conditions = []

        # For downstream modules, we still need to process the body
        if node.body is not None:
            self._visit_body(node.body)
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

        # Use metadata instead of walking expressions
        metadata = self.module_metadata.get(node)
        pushes = metadata.pushes if metadata else []
        calls = metadata.calls if metadata else []
        pops = metadata.pops if metadata else []

        generate_module_ports(self, node, is_downstream, is_sram, is_driver, pushes, calls, pops)

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

    def _walk_expressions(self, block):
        """Iterate through module bodies and yield all expressions."""
        if block is None:
            return
        for item in block:
            if isinstance(item, Expr):
                yield item


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


    def _generate_external_module_wrapper(self, ext_class):
        """Generate a PyCDE wrapper class for an external ExternalSV descriptor."""
        class_name = f"{ext_class.__name__}_ffi"
        metadata = ext_class.metadata()
        module_name = metadata.get('module_name', ext_class.__name__)

        self.external_wrapper_names[ext_class] = class_name

        self.append_code(f'class {class_name}(Module):')
        self.indent += 4

        # Set the module name for PyCDE
        self.append_code(f'module_name = f"{module_name}"')
        if metadata.get('has_clock'):
            self.append_code('clk = Clock()')
        if metadata.get('has_reset'):
            self.append_code('rst = Reset()')

        wires = ext_class.port_specs()
        for wire_name, wire_spec in wires.items():
            wire_type = dump_type(wire_spec.dtype)
            if wire_spec.direction == 'in':
                self.append_code(f'{wire_name} = Input({wire_type})')
            else:
                self.append_code(f'{wire_name} = Output({wire_type})')

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
