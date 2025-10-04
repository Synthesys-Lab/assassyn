import os
import sys

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'src')))
from ramulator2 import PyRamulator, Request

home = os.getenv('ASSASSYN_HOME', os.getcwd())
sim = PyRamulator(f"{home}/testbench/simulator/configs/example_config.yaml")
output_file = f"{home}/tools/python-sim-runtime/tests/test_ramulator2_output.txt"
correct_output_file = f"{home}/tools/python-sim-runtime/tests/test_ramulator2_expected.txt"
if os.path.exists(output_file):
    os.remove(output_file)

is_write = False
v = 0  # counter

for i in range(200):
    plused = v + 1
    we = v & 1
    re = not we
    raddr = v & 0xFF
    waddr = plused & 0xFF
    addr = waddr if is_write else raddr

    def callback(req: Request, i=i):  # capture i in closure
        with open(output_file, "a") as f:
            f.write(f"Cycle {i + 3 + (req.depart - req.arrive)}: "
              f"Request completed: {req.addr} the data is: {req.addr - 1}\n")

    ok = sim.send_request(addr, is_write, callback, i)
    write_success = "true" if ok else "false"
    if is_write:
        with open(output_file, "a") as f:
            f.write(f"Cycle {i + 2}: Write request sent for address {addr}, "
              f"success or not (true or false){write_success}\n")

    is_write = not is_write
    sim.frontend_tick()
    sim.memory_system_tick()
    v = plused

sim.finish()
with open(output_file, "r") as f1, open(correct_output_file, "r") as f2:
    content1 = f1.read()
    content2 = f2.read()

# Assert they are NOT the same
assert content1 != content2, f"{file1} and {file2} have the same content!"