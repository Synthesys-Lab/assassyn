
"""Testbench generation for Verilog simulation."""

from typing import List
from ...builder import SysBuilder

TEMPLATE = '''
import os
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


def runner():
    sim = 'verilator'
    path = Path('./sv/hw')
    with open(path / 'filelist.f', 'r') as f:
        srcs = [path / i.strip() for i in f.readlines()]
    srcs = srcs + ['fifo.sv', 'trigger_counter.sv']
    runner = get_runner(sim)
    runner.build(sources=srcs, hdl_toplevel='Top', always=True)
    runner.test(hdl_toplevel='Top', test_module='tb')

if __name__ == "__main__":
    runner()'''

def generate_testbench(fname: str, sys: SysBuilder, sim_threshold: int, dump_logger: List[str]):
    """Generate a testbench file for the given system."""
    with open(fname, "w", encoding='utf-8') as f:
        dump_logger = '\n        '.join(dump_logger)
        tb_dump = TEMPLATE.format(sim_threshold, dump_logger, sys.name)
        f.write(tb_dump)
