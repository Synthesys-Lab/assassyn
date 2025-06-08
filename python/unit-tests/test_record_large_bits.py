from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
import assassyn


entry = Record(
    symbol = Bits(1),
    a = Bits(32),
    b = Bits(32)   
)

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self,record:Array):
        index = Bits(32)(0)
        valid_temp = Bits(1)(0)
        valid_global = Bits(1)(0)

        for i in range(15):
            valid_temp = ~record[i].symbol
            valid_global = valid_global | valid_temp
            index = valid_temp.select(Bits(32)(i), index)
        
        with Condition(valid_global):
            record[index] = entry.bundle(
                symbol = ~record[index].symbol, 
                a = record[index].a,
                b = index
            ).value()
            log("index {:05} ",index)

def check_raw(raw):
    """
    Validate the simulation output for the correct index log sequence.
    """
    print("Simulation output:")
    print(raw)

    lines = raw.split('\n')
    expected_indices = list(range(14, -1, -1))  # Expected indices: 14 to 0
    cycle = 1
    for i, expected_index in enumerate(expected_indices):
        try:
            actual_index = int(lines[i].strip().split()[-1])
        except (IndexError, ValueError):
            raise ValueError(f"Line {i} is invalid or does not contain an index: {lines[i]}")

        if actual_index != expected_index:
            raise ValueError(f"Expected index: {expected_index}, but got: {actual_index} in line {i}: '{lines[i]}'")
    print("All indices match the expected output.")

def test_record():
    sys = SysBuilder('record_large')
    with sys:
        record = RegArray(entry,15,attr=[Array.FULLY_PARTITIONED])
        driver = Driver()
        call = driver.build( record )

    print(sys)

    config = assassyn.backend.config(
            verilog=utils.has_verilator(),
            sim_threshold=200,
            idle_threshold=200,
            random=True)

    simulator_path, verilator_path = elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    check_raw(raw)
    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check_raw(raw)


if __name__ == '__main__':
    test_record()

