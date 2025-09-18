"""Elaborate Assassyn IR to Verilog."""

import os
from pathlib import Path
import shutil
from .testbench import generate_testbench
from .design import generate_design
from ...ir.module import SRAM
from .utils import extract_sram_params

from ...builder import SysBuilder

from ...utils import create_and_clean_dir

def generate_sram_blackbox_files(sys, path,resource_base):
    """Generate separate Verilog files for SRAM memory blackboxes."""
    sram_modules = [m for m in sys.downstreams if isinstance(m, SRAM)]
    for sram in sram_modules:
        params = extract_sram_params(sram)
        sram_info = params['sram_info']
        array_name = params['array_name']
        data_width = params['data_width']
        addr_width = params['addr_width']
        verilog_code = f'''`ifdef SYNTHESIS
(* blackbox *)
`endif
module sram_blackbox_{array_name} #(
    parameter DATA_WIDTH = {data_width},
    parameter ADDR_WIDTH = {addr_width}
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
'''

        if sram_info['init_file']:
            init_file =  sram_info['init_file']
            src_file = os.path.join(resource_base, init_file) if resource_base else init_file
            verilog_code += f'''
    initial begin
        $readmemh("{src_file}", mem);
    end

    always @ (posedge clk) begin
'''
        else:
            verilog_code += '''
    always @ (posedge clk) begin
        if (!rst_n) begin
            mem[address] <= {{DATA_WIDTH{{1'b0}}}};
        end
'''
        verilog_code += '''
        if (write & banksel) begin
            mem[address] <= wd;
        end
    end

    assign dataout = (read & banksel) ? mem[address] : {DATA_WIDTH{1'b0}};

endmodule
'''

        filename = os.path.join(path, f'sram_blackbox_{array_name}.sv')
        with open(filename, 'w', encoding='utf-8') as f:
            f.write(verilog_code)


def elaborate(sys: SysBuilder, **kwargs) -> str:
    """Elaborate the system into Verilog.

    Args:
        sys: The system to elaborate
        **kwargs: Configuration options including:
            - verilog: The simulator to use ("Verilator", "VCS", or None)
            - resource_base: Path to resources
            - override_dump: Whether to override existing files
            - sim_threshold: Simulation threshold
            - idle_threshold: Idle threshold
            - random: Whether to randomize execution
            - fifo_depth: Default FIFO depth

    Returns:
        Path to the generated Verilog files
    """

    path = kwargs.get('path', os.getcwd())
    path = Path(path) / f"{sys.name}_verilog"

    create_and_clean_dir(path)

    logs = generate_design(path / "design.py", sys)


    generate_testbench(path / "tb.py", sys, kwargs['sim_threshold'], logs)


    default_home = os.getenv('ASSASSYN_HOME', os.getcwd())
    resource_path = Path(default_home) / "python/assassyn/codegen/verilog"
    generate_sram_blackbox_files(sys, path,kwargs.get('resource_base'))
    files_to_copy = ["fifo.sv", "trigger_counter.sv"]
    for file_name in files_to_copy:
        source_file = resource_path / file_name

        if source_file.is_file():
            destination_file = path / file_name
            shutil.copy(source_file, destination_file)
        else:
            print(f"Warning: Resource file not found: {source_file}")

    return path
