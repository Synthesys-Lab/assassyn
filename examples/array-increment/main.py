#!/usr/bin/env python3
"""
Indirect Array Increment: for i in 0..100: a[b[i]]++
Demonstrates dynamic indexing into RegArrays in Assassyn.
"""

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
import assassyn
import os

ARRAY_SIZE = 128
LOOP_COUNT = 100  
INDEX_BITS = 7



class ArrayIncrementFSM(Module):
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, user_state: Array, i: Array, b: Array, a: Array):
        default_cond = Bits(1)(1)
        loop_done = i[0] >= UInt(32)(LOOP_COUNT)

        t_table = {
            "IDLE":   {default_cond: "LOOP"},
            "LOOP":   {loop_done: "DONE", ~loop_done: "LOOP"},
            "DONE":   {default_cond: "DONE"},
            "UNUSED": {default_cond: "UNUSED"},
        }

        def loop_body():
            current_i = i[0]
            idx = b[current_i]
            val = a[idx]
            new_val = val + UInt(32)(1)
            a[idx] = new_val
            i[0] = current_i + UInt(32)(1)
            log("i={}, b[i]={}, a[b[i]]={}", current_i, idx, new_val)

        def done_body():
            log("DONE")
            finish()

        body_table = {"LOOP": loop_body, "DONE": done_body}

        my_fsm = fsm.FSM(user_state, t_table)
        my_fsm.generate(body_table)


class Driver(Module):
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, array_fsm: ArrayIncrementFSM):
        array_fsm.async_called()


def main():
    resource_dir = os.path.join(os.path.dirname(__file__), 'resources')
    hex_file = os.path.join(resource_dir, 'b_indices.hex')

    with open(hex_file, 'r') as f:
        b_init = [int(line.strip(), 16) for line in f if line.strip()]

    if len(b_init) < ARRAY_SIZE:
        b_init.extend([0] * (ARRAY_SIZE - len(b_init)))
    b_init = b_init[:ARRAY_SIZE] 

    sys = SysBuilder('indirect_array_increment')

    with sys:
        user_state = RegArray(Bits(2), 1, initializer=[0])
        i = RegArray(UInt(32), 1, initializer=[0])
        b = RegArray(UInt(INDEX_BITS), ARRAY_SIZE, initializer=b_init)
        a = RegArray(UInt(32), ARRAY_SIZE, initializer=[0] * ARRAY_SIZE)

        array_fsm = ArrayIncrementFSM()
        array_fsm.build(user_state, i, b, a)

        driver = Driver()
        driver.build(array_fsm)

        sys.expose_on_top(a, kind='Output')

    config = assassyn.backend.config(
        verilog=utils.has_verilator(),
        sim_threshold=LOOP_COUNT + 10,
        idle_threshold=LOOP_COUNT + 50,
        random=False
    )

    simulator_path, verilator_path = elaborate(sys, **config)
    raw = utils.run_simulator(simulator_path)

    expected_a = {}
    for line in raw.split('\n'):
        if 'i=' in line and 'b[i]=' in line:
            parts = line.split()
            b_val = int(parts[-2].split('=')[1].rstrip(','))
            a_val = int(parts[-1].split('=')[1])
            expected_a[b_val] = expected_a.get(b_val, 0) + 1
            assert a_val == expected_a[b_val]

    assert len([l for l in raw.split('\n') if 'i=' in l and 'b[i]=' in l]) == LOOP_COUNT + 1

    print("\nAccumulation results:")
    for line in raw.split('\n'):
        if 'i=' in line and 'b[i]=' in line:
            print(line)

    if verilator_path:
        utils.run_verilator(verilator_path)

    print("SUCCESS")


if __name__ == "__main__":
    main()
