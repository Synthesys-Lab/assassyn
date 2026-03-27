"""Radix Sort with FSM-based State Machine Implementation.

This module implements a hardware radix sort using Assassyn's FSM abstraction for
cleaner state machine design. The algorithm sorts 32-bit integers by processing
4 bits at a time (radix-16), requiring 8 passes through the data.

Architecture Overview:
---------------------
The implementation uses two coordinated finite state machines:

1. Main FSM (Driver): Controls the overall sorting process
   - reset: Initialize for next radix digit (4 bits)
   - read: Read data from memory into radix histogram
   - prefix: Compute prefix sum for bucket boundaries
   - write: Write sorted data back to memory (delegated to MemImpl)

2. MemImpl FSM: Handles the write-back phase
   - init: Set up read/write address pointers
   - read: Read next element from source buffer
   - write: Write element to destination based on radix bucket
   - reset: Clear radix counters for next pass

Key Design Features:
-------------------
- Ping-pong buffering: Uses two memory regions to avoid overwriting source data
- Radix-16 counting: Processes 4 bits per pass (16 possible values)
- In-place prefix sum: Computes bucket boundaries for stable sorting
- Declarative FSM: State transitions defined in tables for clarity

Memory Organization:
-------------------
- Memory is divided into two halves for ping-pong buffering
- Each pass reads from one half and writes to the other
- After 8 passes (32 bits / 4 bits), data is fully sorted
"""
import os

from assassyn.frontend import *
from assassyn.backend import *
from assassyn import utils
from assassyn.ir.module import fsm

# Resource base path
current_path = os.path.dirname(os.path.abspath(__file__))
resource_base = f"{current_path}/workload/"
print(f"resource_base: {resource_base}")
# Data width, length
data_width = 32
data_depth = sum(1 for _ in open(f"{resource_base}/numbers.data"))
addr_width = (data_depth * 2 + 1).bit_length()
print(f"data_width: {data_width}, data_depth: {data_depth}, addr_width: {addr_width}")


# MemUser module - 不变
class MemUser(Module):
    """Memory user module that processes read data from SRAM.

    This module receives data from SRAM and updates the radix histogram
    during the read phase. It extracts the relevant 4-bit radix value
    from each data element and increments the corresponding bucket counter.

    The module only operates during Stage 1 (read state) of the main FSM.
    """

    def __init__(self, width):
        """Initialize MemUser with data width.

        Args:
            width: Bit width of data elements (typically 32)
        """
        super().__init__(ports={"rdata": Port(Bits(width))}, no_arbiter=True)

    @module.combinational
    def build(
        self,
        SM_reg: RegArray,
        radix_reg: RegArray,
        offset_reg: RegArray,
        addr_reg: RegArray,
        mem_pingpong_reg: RegArray,
    ):
        """Build the MemUser combinational logic.

        Args:
            SM_reg: Main state machine register (4 states: reset/read/prefix/write)
            radix_reg: Radix histogram array (16 buckets for 4-bit radix)
            offset_reg: Current bit offset being processed (0, 4, 8, ..., 28)
            addr_reg: Current memory address being accessed
            mem_pingpong_reg: Ping-pong buffer selector (0 or 1)

        Returns:
            rdata: Processed read data value
        """
        width = self.rdata.dtype.bits
        rdata = self.pop_all_ports(True)
        rdata = rdata.bitcast(UInt(width))
        # Extract 4-bit radix index from current bit position
        # [0:3] extracts bits 0-3 (4 bits total, values 0-15)
        idx = (rdata >> offset_reg[0])[0:3]
        # Only read to radix_reg in stage 1 (read state)
        # Increment the bucket counter for this radix value
        with Condition(SM_reg[0] == Bits(2)(1)):
            log(
                "Stage 1: Read rdata=({:08x}) from memory addr_reg[0]=({:08x})",
                rdata,
                addr_reg[0] - UInt(addr_width)(1),
            )
            radix_reg[idx] = radix_reg[idx] + UInt(width)(1)
        return rdata


# RadixReducer module - 不变
class RadixReducer(Module):
    """Radix reducer module that computes prefix sum on histogram.

    This module performs an in-place prefix sum (cumulative sum) on the
    radix histogram array. The prefix sum converts bucket counts into
    bucket boundary positions, which are used to determine where each
    element should be placed in the sorted output.

    Example:
        Input histogram:  [3, 1, 2, 0, ...] (counts per bucket)
        Output prefix sum: [0, 3, 4, 6, ...] (starting positions)

    The module operates during Stage 2 (prefix state) of the main FSM
    and takes 16 cycles to process all 16 buckets.
    """

    def __init__(self, width):
        """Initialize RadixReducer with data width.

        Args:
            width: Bit width of counters (typically 32)
        """
        super().__init__(ports={})

    @module.combinational
    def build(self, radix_reg: RegArray, cycle_reg: RegArray):
        """Build the RadixReducer combinational logic.

        Args:
            radix_reg: Radix histogram array (16 elements, will be modified in-place)
            cycle_reg: Cycle counter for prefix sum iteration (0-15)
        """
        # Prefix sum: each bucket adds the previous bucket's value
        # Runs for 16 cycles (one per bucket)
        # Prefix sum
        with Condition(cycle_reg[0] < UInt(data_width)(16)):
            cycle_index = cycle_reg[0][0:3].bitcast(UInt(4))
            radix_reg[cycle_index] = (
                radix_reg[cycle_index] + radix_reg[cycle_index - UInt(4)(1)]
            )
            log(
                "Stage 2: radix_reg[{}]: {:08x}; cycle_index: {:04x};cycle_reg[0]: {:08x}",
                cycle_reg[0] - UInt(data_width)(1),
                radix_reg[cycle_reg[0] - UInt(data_width)(1)],
                cycle_index,
                cycle_reg[0],
            )
            cycle_reg[0] = cycle_reg[0] + UInt(data_width)(1)
        return


class MemImpl(Downstream):
    """Memory implementation with FSM for write-back operations.

    This downstream module handles Stage 3 (write) of the main FSM using its own
    nested 4-state FSM. It reads sorted elements from the source buffer, determines
    their destination positions using the prefix-summed radix histogram, and writes
    them to the destination buffer.

    FSM States:
    ----------
    - init (0): Initialize read/write address pointers
    - read (1): Set up read operation for next element
    - write (2): Write element to destination, update radix counters
    - reset (3): Clear all radix counters for next pass

    The module implements a read-modify-write pattern:
    1. Read an element from source (cycle N)
    2. Compute destination address from radix histogram (cycle N+1)
    3. Write to destination and decrement counter (cycle N+1)
    4. Repeat until all elements processed

    After processing all elements, the radix counters are reset to zero
    and control returns to the main FSM's reset state.
    """

    def __init__(self):
        super().__init__()
        self.name = "MemImpl"

    @downstream.combinational
    def build(
        self,
        rdata: Value,
        wdata: RegArray,
        SM_reg: RegArray,
        addr_reg: RegArray,
        we: RegArray,
        re: RegArray,
        radix_reg: RegArray,
        offset_reg: RegArray,
        mem_pingpong_reg: RegArray,
        mem_start: Value,
        mem_end: Value,
    ):
        SM_MemImpl = RegArray(Bits(2), 1, initializer=[0])
        read_addr_reg = RegArray(UInt(addr_width), 1, initializer=[0])
        write_addr_reg = RegArray(UInt(addr_width), 1, initializer=[data_depth])
        stop_reg = RegArray(UInt(1), 1, initializer=[0])
        reset_cycle_reg = RegArray(UInt(4), 1, initializer=[0])

        # Stage 3: Write Data to Memory (only execute when main SM is in stage 3)
        with Condition(SM_reg[0] == Bits(2)(3)):
            # Define FSM transition conditions
            default = Bits(1)(1)
            not_stopped = stop_reg[0] == UInt(1)(0)
            is_stopped = stop_reg[0] == UInt(1)(1)
            reset_not_done = reset_cycle_reg[0] < UInt(4)(15)
            reset_done = reset_cycle_reg[0] == UInt(4)(15)

            # FSM transition table for MemImpl
            # States: init(0) -> read(1) -> write(2) -> (read or reset) -> reset(3) -> init
            memimpl_table = {
                "init": {default: "read"},
                "read": {default: "write"},
                "write": {not_stopped: "read", is_stopped: "reset"},
                "reset": {reset_done: "init", reset_not_done: "reset"},
            }

            # Define state-specific actions
            def init_action():
                """Initialize read/write addresses."""
                log(
                    "Stage 3-0: Initialization Cycle: Copy addr_reg[0]={:08x} to read_addr_reg[0]; mem_start={:08x}; mem_end={:08x}.",
                    addr_reg[0],
                    mem_start,
                    mem_end,
                )
                read_addr_reg[0] = addr_reg[0]
                write_addr_reg[0] = UInt(addr_width)(data_depth) - mem_start

            def read_action():
                """Read cycle: set up read from memory."""
                log("Stage 3-1: Reading from mem_addr ({}).", addr_reg[0])
                re[0] = Bits(1)(0)
                we[0] = Bits(1)(1)
                addr_reg[0] = write_addr_reg[0]
                with Condition(read_addr_reg[0] > mem_start.bitcast(UInt(addr_width))):
                    read_addr_reg[0] = read_addr_reg[0] - UInt(addr_width)(1)

            def write_action():
                """Write cycle: write data to memory and update radix."""
                log(
                    "Stage 3-2: Writing wdata ({:08x}) to mem_addr ({}); wdata <= rdata ({:08x}).",
                    wdata[0],
                    addr_reg[0],
                    rdata,
                )
                idx = (rdata >> offset_reg[0])[0:3]
                wdata[0] = rdata.bitcast(Bits(data_width))
                write_addr_reg[0] = (
                    radix_reg[idx][0 : (addr_width - 1)].bitcast(UInt(addr_width))
                    - UInt(addr_width)(1)
                    + UInt(addr_width)(data_depth)
                    - mem_start.bitcast(UInt(addr_width))
                )
                radix_reg[idx] = radix_reg[idx] - UInt(data_width)(1)

                # Check if we should stop
                with Condition(read_addr_reg[0] == mem_start.bitcast(UInt(addr_width))):
                    stop_reg[0] = UInt(1)(1)

                # Prepare for next iteration or stop
                with Condition(stop_reg[0] == UInt(1)(0)):  # Continue
                    addr_reg[0] = read_addr_reg[0]
                    re[0] = Bits(1)(1)
                    we[0] = Bits(1)(0)
                with Condition(stop_reg[0] == UInt(1)(1)):  # Stop
                    addr_reg[0] = (
                        radix_reg[idx][0 : (addr_width - 1)].bitcast(UInt(addr_width))
                        - UInt(addr_width)(1)
                        + UInt(addr_width)(data_depth)
                        - mem_start.bitcast(UInt(addr_width))
                    )

            def reset_action():
                """Reset all radix registers and state."""
                with Condition(reset_cycle_reg[0] == UInt(4)(0)):
                    log(
                        "Stage 3-3: Writing wdata ({:08x}) to mem_addr ({});",
                        wdata[0],
                        addr_reg[0],
                    )
                    re[0] = Bits(1)(0)
                    we[0] = Bits(1)(0)

                with Condition(reset_cycle_reg[0] <= UInt(4)(14)):
                    log(
                        "Stage 3-3: Reset radix_reg[{}] to {:08x}.",
                        reset_cycle_reg[0],
                        UInt(data_width)(0),
                    )
                    radix_reg[reset_cycle_reg[0]] = UInt(data_width)(0)
                    reset_cycle_reg[0] = reset_cycle_reg[0] + UInt(4)(1)

                with Condition(reset_cycle_reg[0] == UInt(4)(15)):
                    log(
                        "Stage 3-3: Reset radix_reg[{}] to {:08x}.",
                        reset_cycle_reg[0],
                        UInt(data_width)(0),
                    )
                    log(
                        "Stage 3-3: Reset other registers: reset_cycle_reg[0]=0; SM_MemImpl[0]=0; SM_reg[0]=0; read_addr_reg[0]=0; write_addr_reg[0]=data_depth; stop_reg[0]=0;"
                    )
                    radix_reg[reset_cycle_reg[0]] = UInt(data_width)(0)
                    reset_cycle_reg[0] = UInt(4)(0)
                    SM_reg[0] = Bits(2)(0)  # Return to reset state
                    stop_reg[0] = UInt(1)(0)

            # Create action dictionary
            memimpl_actions = {
                "init": init_action,
                "read": read_action,
                "write": write_action,
                "reset": reset_action,
            }

            # Generate FSM
            memimpl_fsm = fsm.FSM(SM_MemImpl, memimpl_table)
            memimpl_fsm.generate(memimpl_actions)

        return


# Driver module with FSM
class Driver(Module):
    """Driver module that orchestrates the main radix sort FSM.

    This module implements the top-level control flow for radix sort using
    a 4-state FSM. It coordinates memory access, radix histogram building,
    prefix sum computation, and the write-back phase.

    Main FSM States:
    ---------------
    - reset (0): Initialize for next 4-bit digit pass
      * Increment bit offset (0→4→8→...→28)
      * Toggle ping-pong buffer
      * Set up memory read

    - read (1): Build radix histogram
      * Read all elements from current buffer
      * Extract 4-bit radix at current offset
      * Increment bucket counters (via MemUser)
      * Transition when all elements read

    - prefix (2): Compute prefix sum
      * Convert bucket counts to positions
      * Takes 16 cycles (one per bucket)
      * Transition when prefix sum complete

    - write (3): Write sorted data
      * Delegated to MemImpl FSM
      * MemImpl reads, sorts, and writes elements
      * Returns to reset for next pass

    Ping-pong Buffering:
    -------------------
    Memory is split into two halves. Each pass:
    1. Reads from one half (source)
    2. Writes to other half (destination)
    3. Next pass swaps source/destination

    After 8 passes (32 bits / 4 bits), data is fully sorted.
    """

    def __init__(self):
        super().__init__(ports={}, no_arbiter=True)

    @module.combinational
    def build(
        self,
        memory_user: Module,
        radix_reducer: Module,
        cycle_reg: RegArray,
        radix_reg: RegArray,
        SM_reg: RegArray,
        addr_reg: RegArray,
        we: RegArray,
        re: RegArray,
        wdata: RegArray,
        offset_reg: RegArray,
        mem_pingpong_reg: RegArray,
    ):
        """Build the Driver module with main FSM logic.

        Args:
            memory_user: MemUser module for processing read data
            radix_reducer: RadixReducer module for prefix sum
            cycle_reg: Cycle counter for prefix sum (0-15)
            radix_reg: Radix histogram array (16 buckets)
            SM_reg: Main state machine register (2 bits for 4 states)
            addr_reg: Current memory address
            we: Write enable signal
            re: Read enable signal
            wdata: Write data buffer
            offset_reg: Current bit offset (0, 4, 8, ..., 28)
            mem_pingpong_reg: Ping-pong buffer selector (0 or 1)

        Returns:
            Tuple of (mem_start, mem_end) for current buffer region
        """
        # Determine if we're still reading based on address and buffer
        read_cond = (
            (mem_pingpong_reg[0] == UInt(1)(0))
            & (addr_reg[0] < UInt(addr_width)(data_depth))
        ) | (
            (mem_pingpong_reg[0] == UInt(1)(1))
            & (addr_reg[0] < UInt(addr_width)(2 * data_depth))
        )

        # Build Memory
        numbers_mem = SRAM(
            width=data_width,
            depth=2**addr_width,
            init_file=f"{resource_base}/numbers.data",
        )
        numbers_mem.name = "numbers_mem"
        numbers_mem.build(we[0], re[0], addr_reg[0], wdata[0])

        # Connect SRAM output to MemUser input
        memory_user.async_called(rdata=numbers_mem.dout[0])

        mem_start = UInt(addr_width)(0) + (
            mem_pingpong_reg[0] * UInt(addr_width)(data_depth)
        )[0 : (addr_width - 1)].bitcast(UInt(addr_width))
        mem_end = mem_start + UInt(addr_width)(data_depth)

        # Outer loop: only run when offset < data_width
        with Condition(offset_reg[0] < UInt(data_width)(data_width)):
            # Define FSM transition conditions
            default = Bits(1)(1)
            read_not_done = read_cond
            read_done = ~read_cond
            prefix_not_done = cycle_reg[0] < UInt(data_width)(15)
            prefix_done = cycle_reg[0] == UInt(data_width)(15)

            # Main FSM transition table
            # States: reset(0) -> read(1) -> prefix(2) -> write(3) -> reset
            main_table = {
                "reset": {default: "read"},
                "read": {read_done: "prefix", read_not_done: "read"},
                "prefix": {prefix_done: "write", prefix_not_done: "prefix"},
                "write": {default: "write"},  # Transitions back to reset in MemImpl
            }

            # Define state-specific actions
            def reset_action():
                """Initialize for next radix digit."""
                log(
                    "Radix Sort: Bits {} - {} Completed!",
                    offset_reg[0],
                    offset_reg[0] + UInt(data_width)(4),
                )
                log(
                    "========================================================================"
                )
                offset_reg[0] = offset_reg[0] + UInt(data_width)(4)
                addr_reg[0] = UInt(addr_width)(0) + (
                    ~mem_pingpong_reg[0] * UInt(addr_width)(data_depth)
                )[0 : (addr_width - 1)].bitcast(UInt(addr_width))
                re[0] = Bits(1)(1)
                we[0] = Bits(1)(0)
                mem_pingpong_reg[0] = (~mem_pingpong_reg[0]).bitcast(UInt(1))

            def read_action():
                """Read data from memory into radix registers."""
                with Condition(addr_reg[0] < mem_end):
                    # SRAM is automatically accessed when conditions are met
                    addr_reg[0] = addr_reg[0] + UInt(addr_width)(1)

                with Condition(addr_reg[0] == (mem_end - UInt(addr_width)(1))):
                    re[0] = Bits(1)(0)

                with Condition(~read_cond):
                    cycle_reg[0] = UInt(data_width)(1)
                    addr_reg[0] = addr_reg[0] - UInt(addr_width)(1)

            def prefix_action():
                """Perform prefix sum on radix array."""
                radix_reducer.async_called()
                with Condition(cycle_reg[0] == UInt(data_width)(15)):
                    re[0] = Bits(1)(1)
                    we[0] = Bits(1)(0)

            def write_action():
                """Write sorted data back to memory."""
                # SRAM write is handled by MemImpl FSM
                pass

            # Create action dictionary
            main_actions = {
                "reset": reset_action,
                "read": read_action,
                "prefix": prefix_action,
                "write": write_action,
            }

            # Generate main FSM
            main_fsm = fsm.FSM(SM_reg, main_table)
            main_fsm.generate(main_actions)

        with Condition(offset_reg[0] == UInt(data_width)(data_width)):
            log("finish")
            finish()

        return mem_start, mem_end


def build_system():
    sys = SysBuilder("radix_sort_fsm")
    with sys:
        # State machine uses 2 bits for 4 states (reset=0, read=1, prefix=2, write=3)
        SM_reg = RegArray(Bits(2), 1, initializer=[1])  # Start at read state
        cycle_reg = RegArray(UInt(data_width), 1, initializer=[0])
        addr_reg = RegArray(UInt(addr_width), 1, initializer=[0])
        wdata = RegArray(Bits(data_width), 1, initializer=[0])
        we = RegArray(Bits(1), 1, initializer=[0])
        re = RegArray(Bits(1), 1, initializer=[1])
        radix_reg = RegArray(UInt(data_width), 16, initializer=[0] * 16)
        offset_reg = RegArray(UInt(data_width), 1, initializer=[0])
        mem_pingpong_reg = RegArray(UInt(1), 1, initializer=[0])

        # Create Memory User
        memory_user = MemUser(width=data_width)
        rdata = memory_user.build(
            SM_reg=SM_reg,
            radix_reg=radix_reg,
            offset_reg=offset_reg,
            addr_reg=addr_reg,
            mem_pingpong_reg=mem_pingpong_reg,
        )

        # Create Radix Reducer
        radix_reducer = RadixReducer(width=data_width)
        radix_reducer.build(radix_reg, cycle_reg=cycle_reg)

        # Create driver
        driver = Driver()
        mem_start, mem_end = driver.build(
            memory_user,
            radix_reducer,
            cycle_reg=cycle_reg,
            radix_reg=radix_reg,
            SM_reg=SM_reg,
            addr_reg=addr_reg,
            we=we,
            re=re,
            wdata=wdata,
            offset_reg=offset_reg,
            mem_pingpong_reg=mem_pingpong_reg,
        )

        # Create Memory Implementation
        mem_impl = MemImpl()
        mem_impl.build(
            rdata=rdata,
            wdata=wdata,
            SM_reg=SM_reg,
            addr_reg=addr_reg,
            we=we,
            re=re,
            radix_reg=radix_reg,
            offset_reg=offset_reg,
            mem_pingpong_reg=mem_pingpong_reg,
            mem_start=mem_start,
            mem_end=mem_end,
        )

        sys.expose_on_top(radix_reg, kind="Output")

    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=100000,
        idle_threshold=10,
        resource_base="",
        fifo_depth=1,
    )

    simulator_path, verilog_path = elaborate(sys, **conf)
    return sys, simulator_path, verilog_path


if __name__ == "__main__":
    sys, simulator_path, verilog_path = build_system()
    print("System built successfully!")
    utils.run_simulator(simulator_path)
    print("Simulation check completed!")
    if utils.has_verilator():
        raw = utils.run_verilator(verilog_path)
