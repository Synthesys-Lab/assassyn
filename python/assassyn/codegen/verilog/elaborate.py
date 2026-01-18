"""Elaborate Assassyn IR to Verilog."""

import os
import re
from pathlib import Path
import shutil
from typing import List
from .testbench import generate_testbench
from .design import generate_design
from ...ir.memory.sram import SRAM
from ...ir.memory.base import MemoryBase
from .utils import extract_sram_params

from ...builder import SysBuilder
from ...utils import create_dir, repo_path
from ..simulator.external import collect_external_intrinsics
from ...utils import namify
from .array import ArrayMetadataRegistry


def _collect_external_sources(sys):
    """Gather SystemVerilog source files referenced by external intrinsics."""
    sources = set()
    for intrinsic in collect_external_intrinsics(sys):
        source = intrinsic.external_class.metadata().get('source')
        if source:
            sources.add(source)
    return sources


def _resolve_alias_resources(top_sv_path: Path, files_to_copy):
    """Infer CIRCT-generated aliases that need duplicate resource files."""
    if not top_sv_path.exists():
        return []

    alias_resource_files = []
    top_content = top_sv_path.read_text(encoding='utf-8')
    for resource_file in files_to_copy:
        base_module = Path(resource_file).stem
        pattern = rf"\b{base_module}_(\d+)\b"
        for suffix in set(re.findall(pattern, top_content)):
            alias_module = f"{base_module}_{suffix}"
            alias_resource_files.append((resource_file, alias_module))
    return alias_resource_files


def _copy_core_resources(resource_path: Path, destination: Path, files_to_copy):
    """Copy standard SV helper files used by the testbench."""
    for file_name in files_to_copy:
        source_file = resource_path / file_name
        if source_file.is_file():
            destination_file = destination / file_name
            shutil.copy(source_file, destination_file)
        else:
            print(f"Warning: Resource file not found: {source_file}")


def _copy_alias_resources(resource_path: Path, destination: Path, alias_resource_files):
    """Materialize alias modules emitted by CIRCT to keep resource names in sync."""
    for base_file, alias_module in alias_resource_files:
        source_file = resource_path / base_file
        if not source_file.is_file():
            print(f"Warning: Cannot create alias for missing resource: {source_file}")
            continue

        alias_path = destination / f"{alias_module}.sv"
        if alias_path.exists():
            continue

        content = source_file.read_text(encoding='utf-8')
        base_module = Path(base_file).stem
        alias_content = content.replace(f"module {base_module}", f"module {alias_module}", 1)
        alias_path.write_text(alias_content, encoding='utf-8')
        print(f"Copied {source_file} to {alias_path}")


def _copy_external_sources(external_sources, destination: Path):
    """Copy user-provided SystemVerilog sources into the elaboration output."""
    for file_name in external_sources:
        src_path = Path(file_name)
        if not src_path.is_absolute():
            src_path = Path(repo_path()) / file_name

        if src_path.is_file():
            destination_file = destination / src_path.name
            shutil.copy(src_path, destination_file)
            print(f"Copied {src_path} to {destination_file}")
        else:
            print(f"Warning: External resource file not found: {src_path}")


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


def _sv_literal_for_initializer(value: int, width: int) -> str:
    value &= (1 << width) - 1 if width > 0 else 0
    hex_digits = max(1, (width + 3) // 4)
    return f"{width}'h{value:0{hex_digits}x}"


# pylint: disable=too-many-locals,too-many-branches,too-many-statements
def generate_register_file_sv_files(sys: SysBuilder, path: Path) -> List[str]:
    """Generate synthesizable multi-port register-file SV modules for non-payload arrays.

    These files back the external module wrappers emitted by `build_register_file`.
    """

    registry = ArrayMetadataRegistry()
    registry.collect(sys)

    generated: List[str] = []
    for arr in sys.arrays:
        owner = arr.owner
        if isinstance(owner, MemoryBase) and arr.is_payload(owner):
            continue

        metadata = registry.metadata_for(arr)
        if metadata is None:
            num_write_ports = len(arr.get_write_ports())
            num_read_ports = 0
        else:
            num_write_ports = len(metadata.write_ports)
            num_read_ports = len(metadata.read_order)

        module_name = namify(arr.name)
        depth = int(arr.size)
        data_width = int(arr.scalar_ty.bits)
        addr_width = int(arr.index_bits) if int(arr.index_bits) > 0 else 1
        include_read_index = int(arr.index_bits) > 0

        initializer = getattr(arr, "initializer", None)

        lines: List[str] = []
        lines.append("// Auto-generated by assassyn.codegen.verilog (external regfile)")
        lines.append(f"module {module_name}(")
        port_lines: List[str] = []
        port_lines.append("  input  logic        clk,")
        port_lines.append("  input  logic        rst,")
        for w_idx in range(num_write_ports):
            port_lines.append(f"  input  logic        w_port{w_idx},")
            port_lines.append(f"  input  logic [{addr_width - 1}:0]  widx_port{w_idx},")
            port_lines.append(f"  input  logic [{data_width - 1}:0] wdata_port{w_idx},")
        if include_read_index:
            for r_idx in range(num_read_ports):
                port_lines.append(f"  input  logic [{addr_width - 1}:0]  ridx_port{r_idx},")
        for r_idx in range(num_read_ports):
            comma = "," if r_idx != num_read_ports - 1 else ""
            port_lines.append(
                f"  output logic [{data_width - 1}:0] rdata_port{r_idx}{comma}"
            )
        if num_read_ports == 0:
            # Trailing comma cleanup: drop from last write port.
            if port_lines[-1].endswith(","):
                port_lines[-1] = port_lines[-1].rstrip(",")
        lines.extend(port_lines)
        lines.append(");")
        lines.append("")
        lines.append(f"  logic [{data_width - 1}:0] mem [0:{depth - 1}];")
        lines.append("  integer i;")
        lines.append("")
        lines.append("  always_ff @(posedge clk) begin")
        lines.append("    if (rst) begin")
        if initializer is None:
            lines.append(f"      for (i = 0; i < {depth}; i = i + 1) begin")
            lines.append("        mem[i] <= '0;")
            lines.append("      end")
        else:
            if len(initializer) != depth:
                msg = (
                    f"Initializer length {len(initializer)} does not match depth {depth} "
                    f"for {module_name}"
                )
                raise ValueError(msg)
            for idx, value in enumerate(initializer):
                lines.append(
                    f"      mem[{idx}] <= {_sv_literal_for_initializer(int(value), data_width)};"
                )
        lines.append("    end else begin")
        for w_idx in range(num_write_ports - 1, -1, -1):
            lines.append(f"      if (w_port{w_idx}) begin")
            lines.append(f"        mem[widx_port{w_idx}] <= wdata_port{w_idx};")
            lines.append("      end")
        lines.append("    end")
        lines.append("  end")
        lines.append("")
        for r_idx in range(num_read_ports):
            if include_read_index:
                lines.append(f"  assign rdata_port{r_idx} = mem[ridx_port{r_idx}];")
            else:
                lines.append(f"  assign rdata_port{r_idx} = mem[0];")
        lines.append("endmodule")
        lines.append("")

        out_path = path / f"{module_name}.sv"
        out_path.write_text("\n".join(lines), encoding="utf-8")
        generated.append(out_path.name)

    return generated


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

    external_sources = _collect_external_sources(sys)
    external_file_names = sorted({Path(file_name).name for file_name in external_sources})

    logs = generate_design(
        path / "design.py",
        sys,
        default_fifo_depth=kwargs.get('fifo_depth', 2),
    )

    files_to_copy = ["fifo.sv", "trigger_counter.sv"]
    top_sv_path = path / "sv" / "hw" / "Top.sv"
    alias_resource_files = _resolve_alias_resources(top_sv_path, files_to_copy)

    regfile_files = generate_register_file_sv_files(sys, path)
    additional_files = sorted(
        set(
            external_file_names
            + regfile_files
            + [f"{alias}.sv" for _, alias in alias_resource_files]
        )
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
    _copy_core_resources(resource_path, path, files_to_copy)
    _copy_alias_resources(resource_path, path, alias_resource_files)
    _copy_external_sources(external_sources, path)

    return path
