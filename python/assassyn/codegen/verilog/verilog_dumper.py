"""Verilog dumper for Assassyn."""

import os

from collections import defaultdict, deque
from typing import Dict, List, Optional, Set, Tuple, Any

from ...builder import SysBuilder
from ...ir.block import Block, CondBlock
from ...ir.expr import Expr, Intrinsic, Operand, ArrayWrite, FIFOPush, ArrayRead
from ...ir.array import Array
from ...ir.module import Module, SRAM, Port, Downstream
from ...ir.visitor import Visitor
from ...ir.dtype import DType, Int
from ...utils import namify, identifierize, repo_path

from .utils import (
    bool_ty,
    declare_array,
    declare_in,
    declare_out,
    DisplayInstance,
    find_wait_until,
    declare_logic,
    reduce,
    select_1h,
    Edge,
    connect_top,
)
from .visit_expr import visit_expr_impl, dump_ref, dump_arith_ref
from .gather import Gather, ExternalUsage

def fifo_name(fifo: Port):
    """Get the name of a FIFO."""
    return namify(fifo.as_operand())

class VerilogDumper(Visitor):
    """Dumps Verilog code for Assassyn modules."""

    def __init__(self, sys: SysBuilder, config, external_usage, topo, array_memory_params_map, module_expr_map):
        """Initialize a VerilogDumper."""
        self.sys = sys
        self.config = config
        self.pred_stack = deque()
        self.fifo_pushes = {}
        self.array_stores = {}
        self.triggers = {}
        self.external_usage = external_usage
        self.current_module = ""
        self.before_wait_until = False
        self.topo = topo
        self.array_memory_params_map = array_memory_params_map
        self.module_expr_map = module_expr_map

    @classmethod
    def new(cls, sys, config, external_usage, topo, array_memory_params_map, module_expr_map):
        """Create a new VerilogDumper."""
        return cls(sys, config, external_usage, topo, array_memory_params_map, module_expr_map)

    @classmethod
    def collect_array_memory_params_map(cls, sys):
        """Collect array memory parameters."""
        result = {}
        for module in sys.modules:
            if isinstance(module, SRAM):
                result[module.payload] = module
        return result

    def get_pred(self) -> Optional[str]:
        """Get the current predicate."""
        if not self.pred_stack:
            return None
        return self.pred_stack[-1]

    def print_body(self, node) -> str:
        """Print the body of a node."""
        if isinstance(node, Expr):
            expr = node
            return self.visit_expr(expr) or ""
        elif isinstance(node, Block):
            block = node
            return self.visit_block(block) or ""
        else:
            raise ValueError(f"Unexpected reference type: {node}")

    def dump_memory_nodes(self, node, res: str) -> None:
        """Dump memory nodes."""
        # This is a placeholder for the actual implementation
        # TODO: Implement memory node dumping
        pass

    def dump_array(self, array: Array, mem_init_path=None):
        res = []
        display = DisplayInstance.from_array(array)
        w = display.field("w")
        widx = display.field("widx")
        d = display.field("d")
        q = display.field("q")

        res.append(f"  /* {array} */\n")
        map_ = self.array_memory_params_map

        if map_.get(array):
            res.append(declare_logic(array.scalar_ty(), q))
        else:
            res.append(declare_array("", array, q, ";"))

        seen = set()
        drivers = [
            Edge(display, x)
            for x in array.users if isinstance(x, ArrayWrite) and seen.add(x)
        ]

        scalar_bits = array.scalar_ty.bits
        array_size = array.size

        for edge in drivers:
            res.append(declare_logic(array.scalar_ty, edge.field("d")))
            res.append(declare_logic(Int(1), edge.field("w")))
            res.append(declare_logic(array.get_idx_type(), edge.field("widx")))

        if not map_.get(array):
            res.append(declare_logic(array.scalar_ty, d))
            res.append(declare_logic(array.index_type(), widx))
            res.append(declare_logic(Int(1), w))

            write_data = select_1h(
                [(edge.field("w"), edge.field("d")) for edge in drivers],
                scalar_bits
            )
            res.append(f"  assign {d} = {write_data};\n")

            write_idx = select_1h(
                [(edge.field("w"), edge.field("widx")) for edge in drivers],
                array.index_type().bits
            )
            res.append(f"  assign {widx} = {write_idx};\n")

            write_enable = reduce([edge.field("w") for edge in drivers], " | ")
            res.append(f"  assign {w} = {write_enable};\n")

            res.append("  always_ff @(posedge clk or negedge rst_n)\n")
            res.append("    if (!rst_n)\n")
            if mem_init_path:
                res.append(f"      $readmemh(\"{mem_init_path}\", {q});\n")
            elif array.initializer:
                res.append("    begin\n")
                for idx, value in enumerate(array.initializer):
                    elem_init = value.as_ref('IntImm', self.sys).get_value()
                    slice_ = f"{(idx + 1) * scalar_bits - 1}:{idx * scalar_bits}"
                    res.append(f"      {q}[{slice_}] <= {scalar_bits}'d{elem_init};\n")
                res.append("    end\n")
            else:
                init_bits = array.get_flattened_size()
                res.append(f"      {q} <= {init_bits}'d0;\n")

            res.append(f"    else if ({w}) begin\n\n")
            res.append(f"      case ({widx})\n")
            for i in range(array_size):
                slice_ = f"{(i + 1) * scalar_bits - 1}:{i * scalar_bits}"
                res.append(f"        {i} : {q}[{slice_}] <= {d};\n")
            res.append("        default: ;\n")
            res.append("      endcase\n")
            res.append("    end\n")

        return ''.join(res)

    def dump_exposed_array(self, array: Array, exposed_kind, mem_init_path=None):
        res = []
        display = DisplayInstance.from_array(array)
        w = display.field("w")
        widx = display.field("widx")
        d = display.field("d")
        q = display.field("q")

        temp = display.field("temp")
        i = display.field("exposed_i")
        i_valid = display.field("exposed_i_valid")

        res.append(f"  /* {array} */\n")
        map_ = self.array_memory_params_map

        if map_.get(array):
            res.append(declare_logic(array.scalar_ty(), q))
        else:
            res.append(declare_array("", array, q, ";"))

        seen = set()
        drivers = [
            Edge(display.clone(), x.as_ref('Module', array.sys))
            for x in array.users()
            if x.as_ref('Operand', array.sys).get_expr().get_opcode() == 'Store'
            and seen.add(x.get_key())
        ]

        scalar_bits = array.scalar_ty().get_bits()
        array_size = array.get_size()

        for edge in drivers:
            res.append(declare_logic(array.scalar_ty, edge.field("d")))
            res.append(declare_logic(Int(1), edge.field("w")))
            res.append(declare_logic(array.index_type(), edge.field("widx")))

        if exposed_kind in ['Output', 'Inout']:
            o = display.field("exposed_o")
            res.append(f"  assign {o} = {q};\n")
        if exposed_kind in ['Input', 'Inout']:
            res.append(declare_logic(array.scalar_ty(), temp))
            res.append(f"  assign {temp} = {i_valid}?{i}:{d};\n")

        if not map_.get(array):
            res.append(declare_logic(array.scalar_ty, d))
            res.append(declare_logic(array.index_type(), widx))
            res.append(declare_logic(Int(1), w))

            write_data = select_1h(
                [(edge.field("w"), edge.field("d")) for edge in drivers],
                scalar_bits
            )
            res.append(f"  assign {d} = {write_data};\n")

            write_idx = select_1h(
                [(edge.field("w"), edge.field("widx")) for edge in drivers],
                array.get_idx_type().get_bits()
            )
            res.append(f"  assign {widx} = {write_idx};\n")

            write_enable = reduce([edge.field("w") for edge in drivers], " | ")
            res.append(f"  assign {w} = {write_enable};\n")

            res.append("  always_ff @(posedge clk or negedge rst_n)\n")
            res.append("    if (!rst_n)\n")
            if mem_init_path:
                res.append(f"      $readmemh(\"{mem_init_path}\", {q});\n")
            elif array.get_initializer():
                res.append("    begin\n")
                for idx, value in enumerate(array.get_initializer()):
                    elem_init = value.as_ref('IntImm', self.sys).get_value()
                    slice_ = f"{(idx + 1) * scalar_bits - 1}:{idx * scalar_bits}"
                    res.append(f"      {q}[{slice_}] <= {scalar_bits}'d{elem_init};\n")
                res.append("    end\n")
            else:
                init_bits = array.get_flattened_size()
                res.append(f"      {q} <= {init_bits}'d0;\n")

            res.append(f"    else if ({w}) begin\n\n")
            res.append(f"      case ({widx})\n")
            if exposed_kind in ['Input', 'Inout']:
                for i in range(array_size):
                    slice_ = f"{(i + 1) * scalar_bits - 1}:{i * scalar_bits}"
                    res.append(f"        {i} : {q}[{slice_}] <= {temp};\n")
            else:
                for i in range(array_size):
                    slice_ = f"{(i + 1) * scalar_bits - 1}:{i * scalar_bits}"
                    res.append(f"        {i} : {q}[{slice_}] <= {d};\n")

            res.append("        default: ;\n")
            res.append("      endcase\n")
            res.append("    end\n")

        return ''.join(res)


    def dump_runtime(self, fd, sim_threshold):
        # Initialize the result string
        res = "module top(\n"

        # Iterate over exposed nodes
        for exposed_node, kind in self.sys.exposed_nodes:
            if isinstance(exposed_node, Array):
                exposed_nodes_ref = exposed_node
                display = DisplayInstance.from_array(exposed_nodes_ref)
                if kind in ['Output', 'Inout']:
                    o = display.field("exposed_o")
                    res += declare_array("output", exposed_nodes_ref, o, ",")
                if kind in ['Input', 'Inout']:
                    res += declare_in(exposed_nodes_ref.scalar_ty(), display.field("exposed_i"))
                    res += declare_in(bool_ty(), display.field("exposed_i_valid"))
            elif isinstance(exposed_node, Expr):
                expr = exposed_node
                id = namify(expr.as_operand())
                dtype = exposed_node.get_dtype(self.sys)
                bits = dtype.get_bits() - 1
                if kind in ['Output', 'Inout']:
                    res += f"  output logic [{bits}:0] {id}_exposed_o,\n"
                if kind in ['Input', 'Inout']:
                    res += f"  input logic [{bits}:0] {id}_exposed_i,\n"
                    res += f"  input logic {id}_exposed_i_valid,\n"

        res += "  input logic clk,\n  input logic rst_n\n);\n\n"

        # Memory initializations map
        mem_init_map = {}

        # Array -> init_file_path
        for m in self.sys.downstreams:
            if isinstance(m, SRAM):
                if m.init_file:
                    init_file_path = os.path.join(self.config.resource_base, m.init_file)
                    mem_init_map[m.array] = init_file_path

        for key, value in mem_init_map.items():
            res += f"//Array: {key.to_string(self.sys)}, Init File Path: {value}\n"

        exposed_map = dict(self.sys.exposed_nodes)

        # Array storage element definitions
        for array in self.sys.arrays:
            if array in exposed_map:
                res += self.dump_exposed_array(array, exposed_map[array], mem_init_map.get(array))
            else:
                res += self.dump_array(array, mem_init_map.get(array))

        # FIFO storage element definitions
        for module in self.sys.modules:
            for fifo in module.ports:
                res += self.dump_fifo(fifo)

        # Trigger FIFO definitions
        for module in self.sys.modules:
            res += self.dump_trigger(module)

        # Counter delta width
        if self.sys.has_module("Testbench") is not None:
            res += "  assign Testbench_counter_delta = 8'b1;\n\n"
        if self.sys.has_module("Driver") is not None:
            res += "  assign Driver_counter_delta = 8'b1;\n\n"

        # Module instances
        for module in self.sys.modules:
            res += self.dump_module_instance(module)

        # Downstream instances
        for module in self.sys.downstreams:
            res += self.dump_module_instance(module)

        res += "endmodule // top\n\n"

        fd.write(res)

        init = {
            'VCS': """
initial begin
  $fsdbDumpfile("wave.fsdb");
  $fsdbDumpvars();
  $fsdbDumpMDA();
end""",
            'verilator': "",
            'None': None
        }.get(self.config['verilog'])

        if init is None:
            raise ValueError("No simulator specified")

        with open(os.path.join(os.path.dirname(__file__), "runtime.sv"), "r") as src:
            fd.write(src.read())

        threshold = (sim_threshold + 1) * 100
        fd.write(f"""
module tb;

logic clk;
logic rst_n;
""")

        for exposed_node, kind in self.sys.exposed_nodes:
            if isinstance(exposed_node, Array):
                display = DisplayInstance.from_array(exposed_node)
                bits = exposed_node.scalar_ty().get_bits()
                bits_1 = bits - 1
                flatten_bits_1 = exposed_node.get_flattened_size() - 1
                if kind in ['Output', 'Inout']:
                    o = display.field("exposed_o")
                    fd.write(f"logic [{flatten_bits_1}:0]{o};\n")
                if kind in ['Input', 'Inout']:
                    i = display.field("exposed_i")
                    i_valid = display.field("exposed_i_valid")
                    fd.write(f"logic [{bits_1}:0]{i};\n")
                    fd.write(f"logic {i_valid};\n")
                    fd.write(f"\nassign {i_valid} = 1'd0;\n")
                    fd.write(f"assign {i} = {bits}'d0;\n")
            elif isinstance(exposed_node, Expr):
                id = namify(exposed_node.as_operand())
                dtype = exposed_node.get_dtype(self.sys)
                bits = dtype.get_bits()
                bits_1 = bits - 1
                if kind in ['Output', 'Inout']:
                    fd.write(f"logic [{bits_1}:0] {id}_exposed_o;\n")
                if kind in ['Input', 'Inout']:
                    fd.write(f"logic [{bits_1}:0] {id}_exposed_i;\n")
                    fd.write(f"logic {id}_exposed_i_valid;\n")
                    fd.write(f"\nassign {id}_exposed_i_valid = 1'd0;\n")
                    fd.write(f"assign {id}_exposed_i = {bits}'d0;\n")

        fd.write(f"""
initial begin
  clk = 1'b1;
  rst_n = 1'b0;
  #150;
  rst_n = 1'b1;
  #{threshold};
  `ifndef SYNTHESIS
  $finish();
  `endif
end

always #50 clk <= !clk;

{init}

top top_i (
  .clk(clk),
  .rst_n(rst_n)""")

        for exposed_node, kind in self.sys.exposed_nodes:
            if isinstance(exposed_node, Array):
                display = DisplayInstance.from_array(exposed_node)
                if kind in ['Output', 'Inout']:
                    o = display.field("exposed_o")
                    fd.write(f",\n  .{o}({o})")
                if kind in ['Input', 'Inout']:
                    i = display.field("exposed_i")
                    i_valid = display.field("exposed_i_valid")
                    fd.write(f",\n  .{i}({i}),\n  .{i_valid}({i_valid})")
            elif isinstance(exposed_node, Expr):
                id = namify(exposed_node.as_operand())
                if kind in ['Output', 'Inout']:
                    fd.write(f",\n  .{id}_exposed_o({id}_exposed_o)")
                if kind in ['Input', 'Inout']:
                    fd.write(f",\n  .{id}_exposed_i({id}_exposed_i)")
                    fd.write(f",\n  .{id}_exposed_i_valid({id}_exposed_i_valid)")

        fd.write("\n);\n\nendmodule\n")

    def visit_module(self, module: Module) -> Optional[str]:
        """Visit a module and generate Verilog code."""
        self.current_module = namify(module.name)

        result = []

        # Module header
        result.append(f"""
module {self.current_module} (
  input logic clk,
  input logic rst_n,
""")

        # FIFO ports
        for port in module.ports:
            name = fifo_name(port)
            ty = port.dtype
            display = DisplayInstance.from_fifo(port, False)

            result.append(f"  // Port FIFO {name}")
            result.append(declare_in(bool_ty(), display.field("pop_valid")))
            result.append(declare_in(ty, display.field("pop_data")))
            result.append(declare_out(bool_ty(), display.field("pop_ready")))

        # Memory parameters
        has_memory_params = False
        has_memory_init_path = False
        empty_pins = {
            'array': None,
            're': None,
            'we': None,
            'addr': None,
            'wdata': None,
        }

        memory_params = {
            'width': 0,
            'depth': 0,
            'lat': range(0, 1),
            'init_file': None,
            'pins': empty_pins,
        }

        init_file_path = self.config.get("resource_base", ".")

        # External interfaces
        for interf, ops in module.externals:
            if isinstance(interf, Port):
                fifo = interf
                parent_name = fifo.module.name
                display = DisplayInstance.from_fifo(fifo, True)

                result.append(f"  // External FIFO {parent_name}.{fifo.name}")
                result.append(declare_out(bool_ty(), display.field("push_valid")))
                result.append(declare_out(fifo.dtype, display.field("push_data")))
                result.append(declare_in(bool_ty(), display.field("push_ready")))

            elif isinstance(interf, Array):
                array = interf
                display = DisplayInstance.from_array(array)
                result.append(f"  /* {array} */")

                # Check for memory parameters
                for attr in module.attrs:
                    if isinstance(attr, MemoryParams):
                        has_memory_params = True
                        memory_params = attr

                        if attr.init_file:
                            init_file_path = os.path.join(init_file_path, attr.init_file)
                            result.append(f"  /* {init_file_path} */")
                            has_memory_init_path = True

                if has_memory_params:
                    pass
                else:
                    if self.sys.user_contains_opcode(ops, Opcode.LOAD):
                        result.append(declare_array("input", array, display.field("q"), ","))

                    if self.sys.user_contains_opcode(ops, Opcode.STORE):
                        result.append(declare_out(bool_ty(), display.field("w")))
                        result.append(declare_out(array.get_idx_type(), display.field("widx")))
                        result.append(declare_out(array.dtype, display.field("d")))

            elif isinstance(interf, Module):
                module_ref = interf
                display = utils.DisplayInstance.from_module(module_ref)
                result.append(f"  // Module {module_ref.name}")

                # FIXME: Don't hardcode counter delta width
                result.append(declare_out(Int(8), display.field("counter_delta")))
                result.append(declare_in(bool_ty(), display.field("counter_delta_ready")))

            elif isinstance(interf, Expr):
                # Handled below in module_expr_map
                pass

            else:
                raise ValueError(f"Unknown interf kind {type(interf)}")

            result.append("")

        # External usage out bounds
        out_bounds = self.external_usage.out_bounds(module)
        if out_bounds:
            for elem in out_bounds:
                id_ = identifierize(str(elem))
                dtype = elem.dtype
                result.append(declare_out(dtype, f"expose_{id_}"))
                result.append(declare_out(bool_ty(), f"expose_{id_}_valid"))

        # External usage in bounds
        in_bounds = self.external_usage.in_bounds(module)
        if in_bounds:
            for elem in in_bounds:
                id_ = identifierize(str(elem))
                dtype = elem.dtype
                result.append(declare_in(dtype, id_))
                result.append(declare_in(bool_ty(), f"{id_}_valid"))

        # Exposed expressions
        if module in self.module_expr_map:
            exposed_map = self.module_expr_map[module]
            for exposed_node, kind in exposed_map.items():
                if isinstance(exposed_node, Expr):
                    expr = exposed_node
                    id_ = identifierize(str(expr))
                    dtype = exposed_node.dtype
                    bits = dtype.bits - 1

                    if kind == "Output" or kind == "Inout":
                        result.append(f"  output logic [{bits}:0] {id_}_exposed_o,")

                    if kind == "Input" or kind == "Inout":
                        result.append(f"  input logic [{bits}:0] {id_}_exposed_i,")
                        result.append(f"  input logic {id_}_exposed_i_valid,")

        # Event queue for non-downstream modules
        if not isinstance(module, Downstream):
            result.append("  // self.event_q")
            result.append("  input logic counter_pop_valid,")
            result.append("  input logic counter_delta_ready,")
            result.append("  output logic counter_pop_ready,")

        # End of port declarations
        result.append("  output logic expose_executed);\n")

        # Wait until handling
        wait_until = ""
        skip = 0

        wu_intrin = find_wait_until(module)
        if wu_intrin is not None:
            self.before_wait_until = True

            body_iter = module.body.body
            for i, elem in enumerate(body_iter):
                if id(elem) == id(wu_intrin):
                    skip = i + 1
                    break
                result.append(self.print_body(elem))

            value = wu_intrin.args[0].value
            wait_until = f" && ({namify(value.as_operand())})"

        self.before_wait_until = False

        # Executed logic
        result.append("  logic executed;")

        # Testbench cycle counter
        if self.current_module == "testbench":
            result.append("""
  int cycle_cnt;
  always_ff @(posedge clk or negedge rst_n) if (!rst_n) cycle_cnt <= 0;
  else if (executed) cycle_cnt <= cycle_cnt + 1;
""")

        # Clear data structures for gathering
        self.fifo_pushes.clear()
        self.array_stores.clear()
        self.triggers.clear()

        # Handle memory parameters
        if has_memory_params:
            result.append(f"  logic [{memory_params.width - 1}:0] dataout;")
            self.dump_memory_nodes(module.body, result)
        else:
            for elem in list(module.body.body)[skip:]:
                result.append(self.print_body(elem))

        # Generate triggers
        for m, g in self.triggers.items():
            trigger_str = "1"
            if g.is_conditional():
                bits = g.bits
                trigger_str = " + ".join([f"{{ {bits-1}'b0, |{x} }}" for x in g.condition])

            result.append(f"  assign {m}_counter_delta = executed ? {trigger_str} : 0;\n")

        # Generate FIFO pushes
        result.append("  // Gather FIFO pushes")
        for fifo, g in self.fifo_pushes.items():
            result.append(f"""  assign fifo_{fifo}_push_valid = {g.and_("executed", " || ")};
  assign fifo_{fifo}_push_data = {g.select_1h()};
""")

        # Generate array writes
        result.append("  // Gather Array writes")
        if has_memory_params:
            result.append("  // this is Mem Array")

            for a, (idx, data) in self.array_stores.items():
                result.append(f"  logic array_{a}_w;")
                result.append(f"  logic [{memory_params.width - 1}:0] array_{a}_d;")
                addr_bits = (63 - (memory_params.depth - 1).bit_length())
                result.append(f"  logic [{addr_bits}:0] array_{a}_widx;")

                result.append(f"""  assign array_{a}_w = {idx.and_("executed", " || ")};
  assign array_{a}_d = {data.select_1h()};
  assign array_{a}_widx = {idx.value[0]};
""")

                result.append(f"""
  memory_blackbox_{a} #(
        .DATA_WIDTH({memory_params.width}),
        .ADDR_WIDTH({addr_bits})
    ) memory_blackbox_{a}(
    .clk     (clk),
    .address (array_{a}_widx),
    .wd      (array_{a}_d),
    .banksel (1'd1),
    .read    (1'd1),
    .write   (array_{a}_w),
    .dataout (dataout),
    .rst_n   (rst_n)
    );
          """)
        else:
            for a, (idx, data) in self.array_stores.items():
                result.append(f"""  assign array_{a}_w = {idx.and_("executed", " || ")};
    assign array_{a}_d = {data.select_1h()};
    assign array_{a}_widx = {idx.select_1h()};

  """)

        # Executed logic
        if not isinstance(module, Downstream):
            result.append(f"  assign executed = counter_pop_valid{wait_until};")
            result.append("  assign counter_pop_ready = executed;")
        else:
            upstream_execs = [f"{identifierize(x.as_ref(Module, module.sys).name)}_executed"
                            for x in upstreams(module, self.topo)]
            result.append(f"  assign executed = {' || '.join(upstream_execs)};")

        result.append("  assign expose_executed = executed;")
        result.append(f"endmodule // {self.current_module}\n")

        # Memory blackbox modules
        if has_memory_params:
            for a, (_, _) in self.array_stores.items():
                data_width = memory_params.width
                addr_bits = (63 - (memory_params.depth).bit_length())

                result.append(f"""
`ifdef SYNTHESIS
(* blackbox *)
`endif
module memory_blackbox_{a} #(
    parameter DATA_WIDTH = {data_width},
    parameter ADDR_WIDTH = {addr_bits}
)(
    input clk,
    input [ADDR_WIDTH-1:0] address,
    input [DATA_WIDTH-1:0] wd,
    input banksel,
    input read,
    input write,
    output reg [DATA_WIDTH-1:0] dataout,
    input rst_n
);

    localparam DEPTH = 1 << ADDR_WIDTH;
    reg [DATA_WIDTH-1:0] mem [DEPTH-1:0];

  """)

                if has_memory_init_path:
                    result.append(f"""  initial begin
          $readmemh("{init_file_path}", mem);
      end
        always @ (posedge clk) begin
            if (write & banksel) begin
                mem[address] <= wd;
            end
        end

        assign dataout = (read & banksel) ? mem[address] : {{DATA_WIDTH{{1'b0}}}};

    endmodule
              """)
                else:
                    result.append("""

        always @ (posedge clk) begin
            if (!rst_n) begin
                mem[address] <= {DATA_WIDTH{1'b0}};
            end
            else if (write & banksel) begin
                mem[address] <= wd;
            end
        end

        assign dataout = (read & banksel) ? mem[address] : {DATA_WIDTH{1'b0}};

    endmodule
              """)

        return "\n".join(result)

    def visit_block(self, block: Block) -> Optional[str]:
        """Visit a block and generate Verilog code."""
        result = []
        skip = 0

        # Handle block condition
        if isinstance(block, CondBlock):
            cond = block.cond
            dtype = cond.dtype

            if dtype.bits == 1:
                pred = dump_ref(self.sys, cond, True)
            else:
                pred = f"(|{dump_ref(self.sys, cond, False)})"

            self.pred_stack.append(pred)
            skip = 1

        # Handle cycled block
        elif block.cycle is not None:
            cycle = block.cycle
            self.pred_stack.append(f"(cycle_cnt == {cycle})")
            skip = 1

        # Process block body
        for elem in list(block.body)[skip:]:
            if isinstance(elem, Expr):
                expr = elem
                result.append(self.visit_expr(expr) or "")
            elif isinstance(elem, Block):
                sub_block = elem
                result.append(self.visit_block(sub_block) or "")
            else:
                raise ValueError(f"Unexpected reference type: {type(elem)}")

        if skip > 0:
            self.pred_stack.pop()

        return "".join(result)

    def visit_expr(self, expr: Expr) -> Optional[str]:
        """Visit an expression and generate Verilog code."""
        return visit_expr_impl(self, expr)

    def dump_fifo(self, fifo: Port) -> str:
        """Dump FIFO Verilog code."""
        res = []
        display = DisplayInstance.from_fifo(fifo, True)
        fifo_name = namify(f"{fifo.module.name}_{namify(fifo.name)}")
        fifo_width = fifo.dtype.bits
        fifo_depth =  [node.fifo_depth for node in fifo.users if isinstance(node, FIFOPush)]
        assert fifo_depth
        fifo_depth = fifo_depth[0]
        fifo_depth = self.config.get('fifo_depth', 8) if fifo_depth is None else fifo_depth

        res.append(f"  // fifo: {fifo}, depth: {fifo_depth}\n")

        push_valid = display.field("push_valid")
        push_data = display.field("push_data")
        pop_ready = display.field("pop_ready")
        push_ready = display.field("push_ready")
        pop_valid = display.field("pop_valid")
        pop_data = display.field("pop_data")

        edges = [
            Edge(display, x)
            for x in {node.parent.module for node in fifo.users if isinstance(node, FIFOPush)}
        ]

        res.append("  // Declare the pop.{data/valid/ready}\n")
        res.append(declare_logic(fifo.dtype, pop_data))
        res.append(declare_logic(bool_ty(), pop_valid))
        res.append(declare_logic(bool_ty(), pop_ready))

        for edge in edges:
            res.append(declare_logic(fifo.dtype, edge.field("push_data")))
            res.append(declare_logic(bool_ty(), edge.field("push_valid")))
            res.append(declare_logic(bool_ty(), edge.field("push_ready")))

        res.append("  // Broadcast the push_ready signal to all the pushers\n")
        res.append(f"  logic {push_ready};\n")
        for x in edges:
            res.append(f"  assign {x.field('push_ready')} = {push_ready};")

        res.append("  // Gather all the push signal\n")
        valid = reduce((x.field("push_valid") for x in edges), " | ")
        res.append(declare_logic(Int(1), push_valid))
        res.append(f"  assign {push_valid} = {valid};\n")

        res.append("  // 1-hot select the push data\n")
        data = select_1h(
            [(x.field("push_valid"), x.field("push_data")) for x in edges],
            fifo_width
        )
        res.append(declare_logic(fifo.dtype, push_data))
        res.append(f"  assign {push_data} = {data};\n")

        log2_depth = fifo_depth.bit_length() - 1
        res.append(f"""
  fifo #({fifo_width}, {log2_depth}) fifo_{fifo_name}_i (
    .clk(clk),
    .rst_n(rst_n),
    .push_valid({push_valid}),
    .push_data({push_data}),
    .push_ready({push_ready}),
    .pop_valid({pop_valid}),
    .pop_data({pop_data}),
    .pop_ready({pop_ready}));\n\n""")

        return ''.join(res)

    def dump_trigger(self, module: Module) -> str:
        """Dump the trigger event state machine's instantiation."""
        res = []
        module_name = namify(module.name)
        display = DisplayInstance.from_module(module)
        res.append(f"  // Trigger SM of Module: {module.name}\n")
        delta_value = display.field("counter_delta")
        pop_ready = display.field("counter_pop_ready")
        pop_valid = display.field("counter_pop_valid")
        delta_ready = display.field("counter_delta_ready")

        callers = set(x.parent.module for x in module.users)
        callers = [Edge(display, x) for x in callers]

        if module_name != "driver" and module_name != "testbench":
            for edge in callers:
                res.append(declare_logic(Int(8), edge.field("counter_delta")))
                res.append(declare_logic(bool_ty(), edge.field("counter_delta_ready")))
        res.append(declare_logic(bool_ty(), delta_ready))
        res.append(declare_logic(Int(8), delta_value))

        res.append("  // Gather all the push signal\n")
        if module_name != "driver" and module_name != "testbench":
            res.append(f"  assign {delta_value} = {reduce([x.field('counter_delta') for x in callers], ' + ')};\n")
        res.append("  // Broadcast the push_ready signal to all the pushers\n")
        res.append(declare_logic(bool_ty(), pop_ready))
        if module_name != "driver" and module_name != "testbench":
            for x in callers:
                res.append(f"  assign {x.field('counter_delta_ready')} = {pop_ready};\n")
        res.append(declare_logic(bool_ty(), pop_valid))
        res.append(f"  trigger_counter #(8) {module_name}_trigger_i (\n    .clk(clk),\n    .rst_n(rst_n),\n    .delta({delta_value}),\n    .delta_ready({delta_ready}),\n    .pop_valid({pop_valid}),\n    .pop_ready({pop_ready}));\n")
        return ''.join(res)

    def dump_module_instance(self, module: Module) -> str:
        """Dump the module instance."""
        res = []
        module_name = namify(module.name)

        if self.external_usage.out_bounds(module):
            for elem in self.external_usage.out_bounds(module):
                id_ = namify(str(elem))
                ty = elem.get_dtype(self.sys)
                res.append(declare_logic(ty, f"logic_{id_}"))
                res.append(declare_logic(bool_ty(), f"logic_{id_}_valid"))
        is_memory_instance = False
        res.append(declare_logic(bool_ty(), f"{module_name}_executed"))

        res.append(f"  // {module_name}\n  {module_name} {module_name}_i (\n    .clk(clk),\n    .rst_n(rst_n),\n")
        for port in module.ports:
            local = DisplayInstance.from_fifo(port, False)
            global_ = DisplayInstance.from_fifo(port, True)
            res.append(connect_top(local, global_, ["pop_ready", "pop_data", "pop_valid"]))
        for interf, ops in module.externals:
            if isinstance(interf, Port):
                fifo = interf
                fifo_display = DisplayInstance.from_fifo(fifo, True)
                edge = Edge(fifo_display, module)
                res.append(connect_top(fifo_display, edge, ["push_valid", "push_data", "push_ready"]))
            elif isinstance(interf, Array):
                array_ref = interf
                display = DisplayInstance.from_array(array_ref)
                edge = Edge(display, module)

                if isinstance(module, SRAM):
                    is_memory_instance = True
                    if any(isinstance(i, ArrayRead)  for i in ops):
                        res.append(f"    .{display.field('q')}({display.field('q')}),\n")
                    if any(isinstance(i, ArrayWrite) for i in ops):
                        res.append(connect_top(display, edge, ["w", "widx", "d"]))
            elif isinstance(interf, Module):
                interf_module = interf
                display = DisplayInstance.from_module(interf_module)
                edge = Edge(display, module)
                res.append(connect_top(display, edge, ["counter_delta_ready", "counter_delta"]))

        if isinstance(module, Downstream):
            res.append("    // Upstream executed signals\n")
            for x in upstreams(module, self.topo):
                name = namify(x.as_module(module.sys).name)
                res.append(f"    .{name}_executed({name}_executed),\n")

        if self.external_usage.out_bounds(module):
            for elem in self.external_usage.out_bounds(module):
                id_ = namify(str(elem))
                res.append(f"    .expose_{id_}(logic_{id_}),\n")
                res.append(f"    .expose_{id_}_valid(logic_{id_}_valid),\n")

        if self.external_usage.in_bounds(module):
            for elem in self.external_usage.in_bounds(module):
                id_ = namify(str(elem))
                res.append(f"    .{id_}(logic_{id_}),\n")
                res.append(f"    .{id_}_valid(logic_{id_}_valid),\n")

        if module in self.module_expr_map:
            exposed_map = self.module_expr_map[module]
            for exposed_node, kind in exposed_map.items():
                if exposed_node.get_kind() == 'Expr':
                    expr = exposed_node.as_expr(self.sys)
                    id_ = namify(str(expr))
                    if kind in ['Output', 'Inout']:
                        res.append(f"    .{id_}_exposed_o({id_}_exposed_o),\n")
                    if kind in ['Input', 'Inout']:
                        res.append(f"    .{id_}_exposed_i({id_}_exposed_i),\n")
                        res.append(f"    .{id_}_exposed_i_valid({id_}_exposed_i_valid),\n")

        if not isinstance(module, Downstream):
            display = DisplayInstance.from_module(module)
            res.append(f"    .counter_delta_ready({display.field('counter_delta_ready')}),\n")
            res.append(f"    .counter_pop_ready({display.field('counter_pop_ready')}),\n")
            res.append(f"    .counter_pop_valid({display.field('counter_pop_valid')}),\n")
        res.append(f"    .expose_executed({module_name}_executed));\n")
        return ''.join(res)

def generate_cpp_testbench(dir_path, sys, config):
    """Generate C++ testbench for Verilator simulation."""
    if config.get("verilog") == "verilator":
        # Copy main.cpp and Makefile
        tb_resource = repo_path() + "/testbench"

        main_fname = os.path.join(dir_path, "main.cpp")
        with open(os.path.join(tb_resource, "main.cpp"), "r") as src:
            with open(main_fname, "w") as dst:
                dst.write(src.read())

        make_fname = os.path.join(dir_path, "Makefile")
        with open(os.path.join(tb_resource, "Makefile"), "r") as src:
            content = src.read().format(sys.name)
            with open(make_fname, "w") as dst:
                dst.write(content)

    return True

class ExposeGather:
    """Gathers exposed expressions."""

    def __init__(self, sys):
        """Initialize an ExposeGather."""
        self.exposed_map = {}
        self.sys = sys

    def visit_expr(self, expr: Expr) -> None:
        """Visit an expression to check if it's exposed."""
        for node, kind in self.sys.exposed_nodes:
            if node == expr:
                self.exposed_map[expr] = kind
