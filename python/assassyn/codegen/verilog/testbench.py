
"""Testbench generation for Verilog simulation."""

from typing import List, Union
from pathlib import Path
from ...builder import SysBuilder

TEMPLATE = '''
import os
import glob
from pathlib import Path

import cocotb
from cocotb.triggers import Timer
from cocotb.runner import get_runner



@cocotb.test()
async def test_tb(dut):

    dut.clk.value = 1
    dut.rst.value = 1
    await Timer(500, units="ns")
    dut.clk.value = 0
    dut.rst.value = 0
    await Timer(500, units="ns")
    for cycle in range({}):
        dut.clk.value = 1
        await Timer(500, units="ns")
        dut.clk.value = 0
        await Timer(500, units="ns")
        {}
        if dut.global_finish.value == 1:
            break



def runner():
    sim = 'verilator'
    path = Path('./sv/hw')
    with open(path / 'filelist.f', 'r') as f:
        srcs = [path / i.strip() for i in f.readlines()]
    sram_blackbox_files = glob.glob('sram_blackbox_*.sv')
    srcs = srcs + sram_blackbox_files
    srcs = srcs + ['fifo.sv', 'trigger_counter.sv'{}]
    runner = get_runner(sim)
    runner.build(sources=srcs, hdl_toplevel='Top', always=True)
    runner.test(hdl_toplevel='Top', test_module='tb')

if __name__ == "__main__":
    runner()'''

def generate_testbench(fname: Union[str, Path], _sys: SysBuilder, sim_threshold: int,
                       dump_logger: List[str], external_files: List[str]):
    """Generate a testbench file for the given system."""
    with open(str(fname), "w", encoding='utf-8') as f:
        dump_logger = '\n        '.join(dump_logger)
        extra_sources = ''.join(f", '{name}'" for name in external_files)
        tb_dump = TEMPLATE.format(sim_threshold, dump_logger, extra_sources)
        f.write(tb_dump)
