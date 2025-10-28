from assassyn.frontend import *
from assassyn.test import run_test


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
            valid_temp = ~record[i][0].symbol
            valid_global = valid_global | valid_temp
            index = valid_temp.select(Bits(32)(i), index)
        with Condition(valid_global):
            for i in range(15):
                with Condition(index == Bits(32)(i)):
                    (record[i] & self)[0] <= entry.bundle(
                        symbol = ~record[i][0].symbol,
                        a = record[i][0].a,
                        b = index
                    ).value()
                    log("index {:05} ",index)

def build_system():
    record = [RegArray(entry,1) for _ in range(15)]
    driver = Driver()
    call = driver.build( record )

def check_raw(raw):
    """
    Validate the simulation output for the correct index log sequence.
    """
    print("Simulation output:")
    print(raw)

    lines = raw.split('\n')
    expected_indices = list(range(14, -1, -1))  # Expected indices: 14 to 0
    cycle = 1
    log_lines = (line for line in lines if "index" in line)
    for i, expected_index in enumerate(expected_indices):
        try:
            log_line = next(log_lines)
            actual_index = int(log_line.strip().split()[-1])
        except (IndexError, ValueError):
            raise ValueError(f"Line {i} is invalid or does not contain an index: {lines[i]}")

        if actual_index != expected_index:
            raise ValueError(f"Expected index: {expected_index}, but got: {actual_index} in line {i}: '{lines[i]}'")
    print("All indices match the expected output.")

def test_record():
    run_test(
        'record_large',
        build_system,
        check_raw,
        sim_threshold=200,
        idle_threshold=200,
        random=True,
        verilog=True
    )


if __name__ == '__main__':
    test_record()
