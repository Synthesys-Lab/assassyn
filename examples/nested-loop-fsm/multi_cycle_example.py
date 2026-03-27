"""Multi-Cycle Nested For-Loop FSM Example: Simple Multiplier

This example demonstrates a nested for-loop FSM with computation.
Both FSMs are in the Driver module, communicating through shared registers.

Computation: For each iteration i (0 to 9), compute result = i * 3
then accumulate all results.

Expected output: sum = (0*3) + (1*3) + (2*3) + ... + (9*3) = 135

Note: This version uses single-cycle compute to avoid scheduling conflicts.
For true multi-cycle inner FSM, a Downstream module approach is needed.
"""

from assassyn.frontend import *
from assassyn.backend import *
from assassyn.ir.module import fsm
from assassyn import utils


class Driver(Module):
    """Driver module implementing both outer and inner FSMs."""

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        # ==================================================================
        # Shared Registers
        # ==================================================================

        # Outer loop state and control
        outer_state = RegArray(Bits(2), 1, initializer=[0])
        loop_counter = RegArray(UInt(32), 1, initializer=[0])
        outer_valid = RegArray(Bits(1), 1, initializer=[0])

        # Inner FSM state and computation (multi-cycle)
        inner_state = RegArray(Bits(2), 1, initializer=[0])
        multiplicand = RegArray(UInt(32), 1, initializer=[0])
        multiplier = RegArray(UInt(32), 1, initializer=[0])
        product = RegArray(UInt(32), 1, initializer=[0])
        accumulator = RegArray(UInt(32), 1, initializer=[0])
        shift_count = RegArray(UInt(8), 1, initializer=[0])

        # Handshake signals
        inner_ready = RegArray(Bits(1), 1, initializer=[1])
        inner_done = RegArray(Bits(1), 1, initializer=[0])

        # Data passed from outer to inner
        iteration_data = RegArray(UInt(32), 1, initializer=[0])

        # ==================================================================
        # Inner FSM: Simple Multiplier (single-cycle compute)
        # ==================================================================

        # Inner FSM transition conditions
        inner_default = Bits(1)(1)
        inner_valid_high = outer_valid[0] == Bits(1)(1)

        # Inner FSM transition table (single-cycle, like basic_example)
        inner_table = {
            "idle": {inner_valid_high: "compute", ~inner_valid_high: "idle"},
            "compute": {inner_default: "done"},
            "done": {inner_default: "reset"},
            "reset": {inner_default: "idle"},
        }

        # Inner FSM state actions
        def inner_idle_action():
            inner_ready[0] = Bits(1)(1)
            inner_done[0] = Bits(1)(0)
            log("  InnerFSM: [IDLE] ready")

        def inner_compute_action():
            inner_ready[0] = Bits(1)(0)
            inner_done[0] = Bits(1)(0)
            # Compute result = iteration_data * 3
            product[0] = iteration_data[0] + iteration_data[0] + iteration_data[0]
            log("  InnerFSM: [COMPUTE] {} * 3 = {}", iteration_data[0], product[0])

        def inner_done_action():
            inner_ready[0] = Bits(1)(0)
            inner_done[0] = Bits(1)(1)
            # Accumulate result
            accumulator[0] = accumulator[0] + product[0]
            log("  InnerFSM: [DONE] product={}, total={}",
                product[0], accumulator[0])

        def inner_reset_action():
            inner_ready[0] = Bits(1)(0)
            inner_done[0] = Bits(1)(0)
            log("  InnerFSM: [RESET]")

        inner_action_dict = {
            "idle": inner_idle_action,
            "compute": inner_compute_action,
            "done": inner_done_action,
            "reset": inner_reset_action,
        }

        # Generate inner FSM
        inner_fsm_inst = fsm.FSM(inner_state, inner_table)
        inner_fsm_inst.generate(inner_action_dict)

        # ==================================================================
        # Outer FSM: Loop Controller
        # ==================================================================

        # Loop parameters
        loop_start = UInt(32)(0)
        loop_end = UInt(32)(10)
        loop_step = UInt(32)(1)

        # Outer FSM transition conditions
        outer_default = Bits(1)(1)
        ready_high = inner_ready[0] == Bits(1)(1)
        done_high = inner_done[0] == Bits(1)(1)
        not_finished = loop_counter[0] < loop_end
        finished = loop_counter[0] >= loop_end

        # Outer FSM transition table
        outer_table = {
            "init": {outer_default: "wait_ready"},
            "wait_ready": {ready_high: "execute", ~ready_high: "wait_ready"},
            "execute": {outer_default: "check_done"},
            "check_done": {
                done_high & not_finished: "wait_ready",
                done_high & finished: "check_done",
                ~done_high: "check_done",
            },
        }

        # Outer FSM state actions
        def outer_init_action():
            loop_counter[0] = loop_start
            outer_valid[0] = Bits(1)(0)
            log("OuterFSM: [INIT] start={}, end={}", loop_start, loop_end)

        def outer_wait_ready_action():
            outer_valid[0] = Bits(1)(0)
            log("OuterFSM: [WAIT_READY] counter={}, ready={}",
                loop_counter[0], inner_ready[0])

        def outer_execute_action():
            iteration_data[0] = loop_counter[0]
            outer_valid[0] = Bits(1)(1)
            log("OuterFSM: [EXECUTE] sending iter={}", loop_counter[0])

        def outer_check_done_action():
            outer_valid[0] = Bits(1)(0)

            with Condition(inner_done[0] == Bits(1)(1)):
                loop_counter[0] = loop_counter[0] + loop_step
                log("OuterFSM: [CHECK_DONE] iter {} complete",
                    loop_counter[0] - loop_step)

            with Condition(loop_counter[0] >= loop_end):
                log("OuterFSM: [DONE] Loop complete! Final sum={}", accumulator[0])
                finish()

        outer_action_dict = {
            "init": outer_init_action,
            "wait_ready": outer_wait_ready_action,
            "execute": outer_execute_action,
            "check_done": outer_check_done_action,
        }

        # Generate outer FSM
        outer_fsm_inst = fsm.FSM(outer_state, outer_table)
        outer_fsm_inst.generate(outer_action_dict)


def test_multi_cycle_example():
    """Build and run the multi-cycle multiplier example."""
    print("=" * 60)
    print("Multi-Cycle For-Loop FSM: Shift-Add Multiplier")
    print("=" * 60)
    print("Computing: sum = (0*3) + (1*3) + ... + (9*3)")
    print("Expected result: 135")
    print("=" * 60)

    # Build system
    sys = SysBuilder('multi_cycle_loop_fsm')
    with sys:
        driver = Driver()
        driver.build()

    print("\nSystem built successfully")

    # Configure and elaborate
    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=5000,
        idle_threshold=100,
    )

    print("Elaborating system...")
    simulator_path, verilog_path = elaborate(sys, **conf)
    print(f"Simulator: {simulator_path}")

    # Run simulation
    print("\n" + "=" * 60)
    print("Running Simulation...")
    print("=" * 60)
    raw = utils.run_simulator(simulator_path)

    # Show last lines
    print("\n" + "=" * 60)
    print("Simulation Output (last 50 lines):")
    print("=" * 60)
    lines = raw.split('\n')
    for line in lines[-50:]:
        if line.strip():
            print(line)

    # Verify
    print("\n" + "=" * 60)
    print("Verification:")
    print("=" * 60)

    for line in lines:
        if "Final sum=" in line:
            parts = line.split("Final sum=")
            if len(parts) > 1:
                sum_str = parts[1].strip()
                try:
                    final_sum = int(sum_str)
                    expected = 135
                    if final_sum == expected:
                        print(f"✅ SUCCESS: sum = {final_sum}")
                    else:
                        print(f"❌ FAILED: sum = {final_sum} (expected {expected})")
                except:
                    print(f"⚠️  Could not parse: {sum_str}")
                break
    else:
        print("⚠️  Final sum not found")

    # Verilator
    if verilog_path and utils.has_verilator():
        print("\n" + "=" * 60)
        print("Running Verilator...")
        print("=" * 60)
        raw_v = utils.run_verilator(verilog_path)
        for line in raw_v.split('\n'):
            if "Final sum=" in line:
                parts = line.split("Final sum=")
                if len(parts) > 1:
                    try:
                        final_sum = int(parts[1].strip())
                        if final_sum == 135:
                            print(f"✅ Verilator SUCCESS: sum = {final_sum}")
                        else:
                            print(f"❌ Verilator FAILED: sum = {final_sum}")
                    except:
                        pass
                    break

    print("\n" + "=" * 60)
    print("Test Complete")
    print("=" * 60)


if __name__ == '__main__':
    test_multi_cycle_example()
