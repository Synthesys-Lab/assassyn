"""Test for cross-module ArrayRead exposure.

This test verifies that ArrayRead expressions used as external values in other
modules are properly exposed as output ports during Verilog code generation.

The bug this test catches: When an ArrayRead from one module is used as an
external value in another module (especially downstream modules), the analysis
pass must detect this cross-module usage and record the expression for exposure.
Without this, the generated Verilog code will try to access a non-existent
expose_* attribute, causing an AttributeError.
"""

from assassyn.frontend import *
from assassyn.test import run_test


class Producer(Module):
    """Module that reads from an array and returns the value."""

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, data_array: Array):
        # This ArrayRead will be used as an external in the Consumer module
        value = data_array[0]
        return value


class Consumer(Downstream):
    """Downstream module that uses the ArrayRead from Producer as an external."""

    def __init__(self):
        super().__init__()

    @downstream.combinational
    def build(self, external_value: Value,pass_valid:Value):
        # Use the external value (which is an ArrayRead from Producer)
        result = external_value.optional(UInt(32)(0))
        incremented = result + UInt(32)(1)
        log("Consumer received: {}, incremented: {}", result, incremented)

        with Condition(pass_valid.valid()):
            log("Pass valid")


class Driver(Module):
    """Driver module that coordinates the system."""

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, producer: Producer, data_array: Array ):
        # Initialize the data array
        (data_array & self)[0] <= UInt(32)(42)

        # Initialize the counter
        counter = RegArray(UInt(32), 1)
        current = counter[0]
        next_val = current + UInt(32)(1)
        (counter & self)[0] <=  next_val
        with Condition(counter[0] < UInt(32)(2)):
            pass_valid = counter[0]
        # Call the producer
        producer.async_called()
        return pass_valid


def build_system():
    """Build the test system with cross-module ArrayRead usage."""
    # Create shared array
    data_array = RegArray(UInt(32), 1)

    # Create producer that reads from array
    producer = Producer()
    array_read_value = producer.build(data_array)

    # Create consumer that uses the ArrayRead as external
    consumer = Consumer()

    # Create driver
    driver = Driver()
    pass_valid = driver.build(producer, data_array)

    consumer.build(array_read_value,pass_valid)


def check(raw):
    """Verify the output contains expected log messages."""
    found_logs = 0
    valid_logs = 0
    for line in raw.split('\n'):
        if "Pass valid" in line:
            valid_logs += 1
            if valid_logs > 2:
                raise AssertionError("Received more 'Pass valid' logs than expected")
        if 'Consumer received:' in line:
            found_logs += 1
            # Extract the values from the log
            parts = line.split()
            if len(parts) >= 4:
                received = parts[-3].rstrip(',')
                incremented = parts[-1]
                # Verify the increment operation
                assert int(incremented) == int(received) + 1, \
                    f"Expected {int(received) + 1}, got {incremented}"

    # Ensure we got some output
    assert found_logs > 0, "No consumer logs found in output"


def test_array_read_external():
    """Run the test for cross-module ArrayRead exposure."""
    run_test('array_read_external', build_system, check, sim_threshold=10)


if __name__ == '__main__':
    test_array_read_external()
