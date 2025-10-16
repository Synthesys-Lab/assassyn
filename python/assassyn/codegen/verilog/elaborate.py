"""Elaborate Assassyn IR to Verilog."""

import os
import re
from pathlib import Path
import shutil
from .testbench import generate_testbench
from .design import generate_design
from ...ir.memory.sram import SRAM
from .utils import extract_sram_params

from ...builder import SysBuilder
from ...ir.module.external import ExternalSV

from ...utils import create_dir, repo_path


def generate_sram_blackbox_files(sys, path, resource_base=None):
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
            init_file = sram_info['init_file']
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


# pylint: disable=too-many-locals,too-many-branches
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
    path = Path(path) / "verilog"

    create_dir(path)

    external_sources = set()
    for module in sys.modules + sys.downstreams:
        if isinstance(module, ExternalSV) and getattr(module, 'file_path', None):
            external_sources.add(module.file_path)

    external_file_names = sorted({Path(file_name).name for file_name in external_sources})

    logs = generate_design(path / "design.py", sys)

    files_to_copy = ["fifo.sv", "trigger_counter.sv"]
    top_sv_path = path / "sv" / "hw" / "Top.sv"
    alias_resource_files = []  # (base_file, alias_module)

    if top_sv_path.exists():
        top_content = top_sv_path.read_text(encoding='utf-8')
        for resource_file in files_to_copy:
            base_module = Path(resource_file).stem
            pattern = rf"\b{base_module}_(\d+)\b"
            for suffix in set(re.findall(pattern, top_content)):
                alias_module = f"{base_module}_{suffix}"
                alias_resource_files.append((resource_file, alias_module))

    additional_files = sorted(
        set(external_file_names + [f"{alias}.sv" for _, alias in alias_resource_files])
    )

    generate_testbench(
        path / "tb.py",
        sys,
        kwargs['sim_threshold'],
        logs,
        additional_files
    )

    default_home = os.getenv('ASSASSYN_HOME', os.getcwd())
    resource_path = Path(default_home) / "python/assassyn/codegen/verilog"
    generate_sram_blackbox_files(sys, path, kwargs.get('resource_base'))
    for file_name in files_to_copy:
        source_file = resource_path / file_name

        if source_file.is_file():
            destination_file = path / file_name
            shutil.copy(source_file, destination_file)
        else:
            print(f"Warning: Resource file not found: {source_file}")

    # Create alias resources when CIRCT renames parameterised modules (e.g., fifo_1)
    for base_file, alias_module in alias_resource_files:
        source_file = resource_path / base_file
        if not source_file.is_file():
            print(f"Warning: Cannot create alias for missing resource: {source_file}")
            continue

        alias_path = path / f"{alias_module}.sv"
        if not alias_path.exists():
            content = source_file.read_text(encoding='utf-8')
            base_module = Path(base_file).stem
            content = content.replace(f"module {base_module}", f"module {alias_module}", 1)
            alias_path.write_text(content, encoding='utf-8')
            print(f"Copied {source_file} to {alias_path}")

    for file_name in external_sources:
        src_path = Path(file_name)
        if not src_path.is_absolute():
            src_path = Path(repo_path()) / file_name

        if src_path.is_file():
            destination_file = path / src_path.name
            shutil.copy(src_path, destination_file)
            print(f"Copied {src_path} to {destination_file}")
        else:
            print(f"Warning: External resource file not found: {src_path}")

    return path
