# pylint: disable=C0302
# pylint: disable=no-member
"""Verilog design generation and code dumping."""

from typing import List, Dict, Tuple, Union, Optional
from collections import defaultdict
from pathlib import Path

from .utils import (
    HEADER,
    dump_type,
    extract_sram_params,
    ensure_bits,
)

from ...ir.module import Module
from ...ir.memory.sram import SRAM
from ...builder import SysBuilder
from ...ir.visitor import Visitor
from ...ir.const import Const
from ...ir.array import Array
from ...ir.dtype import RecordValue
from ...utils import namify, unwrap_operand
from ...utils.enforce_type import enforce_type
from ...ir.expr import Expr
from ._expr import codegen_expr
from .cleanup import cleanup_post_generation
from .rval import dump_rval as dump_rval_impl
from .module import generate_module_ports
from .system import generate_system
from .metadata import ModuleMetadata, FIFORegistry
from .analysis import collect_fifo_metadata
from .array import ArrayMetadataRegistry


class CIRCTDumper(Visitor):  # pylint: disable=too-many-instance-attributes,too-many-statements
    """Dumps IR to CIRCT-compatible Verilog code."""

    wait_until: bool
    indent: int
    code: List[str]
    logs: List[str]
    connections: List[Tuple[Module, str, str]]
    current_module: Module
    sys: SysBuilder
    async_callees: Dict[Module, List[Module]]
    downstream_dependencies: Dict[Module, List[Module]]
    is_top_generation: bool
    finish_body: list[str]
    memory_defs: set

    def __init__(
        self,
        *,
        module_metadata: Dict[Module, ModuleMetadata] | None = None,
        fifo_registry: FIFORegistry | None = None,
    ):
        super().__init__()
        self.wait_until = None
        self.indent = 0
        self.code = []
        self.logs = []
        self.connections = []
        self.current_module = None
        self.sys = None
        self.async_callees = {}
        self.downstream_dependencies = {}
        self.is_top_generation = False
        self.finish_body = []
        self.array_metadata = ArrayMetadataRegistry()
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
        self.module_metadata: Dict[Module, ModuleMetadata] = (
            module_metadata if module_metadata is not None else {}
        )
        self.fifo_registry = fifo_registry if fifo_registry is not None else FIFORegistry()

    def get_pred(self, expr: Expr) -> str:
        """Format the predicate guarding *expr* (or return the default literal)."""
        return self.format_predicate(expr.meta_cond)

    def format_predicate(self, predicate: Optional[Expr]) -> str:
        """Format a predicate value as a Bits expression."""
        if predicate is None:
            return "Bits(1)(1)"
        predicate_code = self.dump_rval(predicate, False)
        return ensure_bits(predicate_code)

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


    def dump_rval(self, node, with_namespace: bool, module_name: str = None) -> str:
        """Dump a reference to a node with options."""
        return dump_rval_impl(self, node, with_namespace, module_name)

    def append_code(self, code: str):
        """Append code with proper indentation."""
        if code.strip() == '':
            self.code.append('')
        else:
            self.code.append(self.indent * ' ' + code)

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

        if body is not None:
            self.append_code(body)



    # pylint: disable=too-many-locals,too-many-branches,too-many-statements
    def visit_module(self, node: Module):
        # STAGE 1: ANALYSIS & BODY GENERATION
        original_code_buffer = self.code
        original_indent = self.indent
        self.code = []
        self.indent = original_indent + 8

        metadata = self.module_metadata.get(node)
        if metadata is None:
            raise RuntimeError(
                f"FIFO metadata missing for module {node.name}; run collect_fifo_metadata "
                "and pass the results to CIRCTDumper."
            )
        if metadata.registry is not self.fifo_registry:
            raise RuntimeError(
                f"FIFO metadata for module {node.name} is associated with a different registry."
            )

        self.wait_until = None
        self.current_module = node
        previous_module_ctx = self.module_ctx
        self.module_ctx = node
        self.finish_body = []

        # For downstream modules, we still need to process the body
        if node.body is not None:
            self._visit_body(node.body)
        cleanup_post_generation(self)

        construct_method_body = self.code

        self.code = original_code_buffer
        self.indent = original_indent

        self.current_module = node

        self.append_code(f'class {namify(node.name)}(Module):')
        self.indent += 4

        generate_module_ports(self, node)

        self.append_code('')
        self.append_code('@generator')
        self.append_code('def construct(self):')

        if isinstance(node, SRAM):
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

        metadata = self.array_metadata.metadata_for(array)
        if metadata is None:
            num_write_ports = len(array.get_write_ports())
            num_read_ports = 0
        else:
            num_write_ports = len(metadata.write_ports)
            num_read_ports = len(metadata.read_order)

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
        metadata = self.array_metadata.metadata_for(arr)
        if metadata is None:
            return
        write_mapping = metadata.write_ports
        read_mapping = metadata.read_ports_by_module
        if not write_mapping and not any(read_mapping.values()):
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

        module_metadata, fifo_registry = collect_fifo_metadata(sys)
        dumper = CIRCTDumper(
            module_metadata=module_metadata,
            fifo_registry=fifo_registry,
        )

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


# Late-bind CIRCTDumper so forward-referenced helpers resolve the runtime type.
from ._expr import array as _array_codegen  # pylint: disable=wrong-import-position

_array_codegen.CIRCTDumper = CIRCTDumper
