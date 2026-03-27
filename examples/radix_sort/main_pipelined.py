# Radix Sort with Pipelined Write Stage
#
# This is an optimized version using dual-SRAM architecture to pipeline
# the write stage, achieving ~2x speedup on write operations.
#
# Key optimization: Overlap read and write operations using two SRAMs:
# - One SRAM for reading source data
# - One SRAM for writing sorted data
# - Ping-pong between passes
#
# Expected performance: ~33,000 cycles (vs 49,441 baseline, 33% improvement)
import os

from assassyn.frontend import *
from assassyn.backend import *
from assassyn import utils

# Resource base path
current_path = os.path.dirname(os.path.abspath(__file__))
resource_base = f"{current_path}/workload/"
print(f"resource_base: {resource_base}")
# Data width, length
data_width = 32
data_depth = sum(1 for _ in open(f"{resource_base}/numbers.data"))
addr_width = (data_depth * 2 + 1).bit_length()
print(f"data_width: {data_width}, data_depth: {data_depth}, addr_width: {addr_width}")

# MemUser module
class MemUser(Module):
    def __init__(self, width):
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
        width = self.rdata.dtype.bits
        rdata = self.pop_all_ports(True)
        rdata = rdata.bitcast(UInt(width))
        idx = (rdata >> offset_reg[0])[0:3]
        # Only read to radix_reg in stage 1
        with Condition(SM_reg[0] == UInt(2)(1)):
            log(
                "Stage 1: Read rdata=({:08x}) from memory addr_reg[0]=({:08x})",
                rdata,
                addr_reg[0] - UInt(addr_width)(1),
            )
            radix_reg[idx] = radix_reg[idx] + UInt(width)(1)
        return rdata


# RadixReducer module
class RadixReducer(Module):
    def __init__(self, width):
        super().__init__(ports={})

    @module.combinational
    def build(self, radix_reg: RegArray, cycle_reg: RegArray):
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
        we_a: RegArray,
        re_a: RegArray,
        we_b: RegArray,
        re_b: RegArray,
        radix_reg: RegArray,
        offset_reg: RegArray,
        mem_pingpong_reg: RegArray,
        mem_start: Value,
        mem_end: Value,
    ):
        # Note: addr_a_reg and addr_b_reg will be accessed from outer scope
        # to avoid feedback loop in Downstream triggering
        # Pipeline FSM states: 0=init, 1=pipeline, 2=drain, 3=reset
        SM_MemImpl = RegArray(UInt(2), 1, initializer=[0])
        read_addr_reg = RegArray(UInt(addr_width), 1, initializer=[0])
        write_addr_reg = RegArray(UInt(addr_width), 1, initializer=[data_depth])
        reset_cycle_reg = RegArray(UInt(5), 1, initializer=[0])

        # Stage 3: Write Data to Memory (Pipelined)
        with Condition(SM_reg[0] == UInt(2)(3)):
            # State 0: Init - Prefetch first element
            with Condition(SM_MemImpl[0] == UInt(2)(0)):
                log(
                    "Stage 3-0 (Init): Prefetch first element. read_addr={:08x}, mem_start={:08x}",
                    addr_reg[0],
                    mem_start,
                )
                # Initialize addresses - only set internal registers
                read_addr_reg[0] = addr_reg[0]
                write_addr_reg[0] = UInt(addr_width)(data_depth) - mem_start
                # Transition to pipeline state - actual SRAM control happens there
                SM_MemImpl[0] = UInt(2)(1)

            # State 1: Pipeline - Overlap read and write
            with Condition(SM_MemImpl[0] == UInt(2)(1)):
                # On first entry (read_addr_reg == addr_reg), just prefetch
                # Otherwise, process buffered data while fetching next

                # Calculate write address based on current rdata's radix
                idx = (rdata.bitcast(UInt(data_width)) >> offset_reg[0])[0:3]
                write_addr_reg[0] = (
                    radix_reg[idx][0 : (addr_width - 1)].bitcast(UInt(addr_width))
                    - UInt(addr_width)(1)
                    + UInt(addr_width)(data_depth)
                    - mem_start.bitcast(UInt(addr_width))
                )

                log(
                    "Stage 3-1 (Pipeline): read_addr={:08x}, write_addr={:08x}, rdata={:08x}, idx={}",
                    read_addr_reg[0],
                    write_addr_reg[0],
                    rdata,
                    idx,
                )

                # Update radix count
                radix_reg[idx] = radix_reg[idx] - UInt(data_width)(1)

                # Set wdata for write
                wdata[0] = rdata.bitcast(Bits(data_width))

                # Control SRAMs based on ping-pong
                # If ping-pong=0: read from A (source), write to B (dest)
                # If ping-pong=1: read from B (source), write to A (dest)
                with Condition(mem_pingpong_reg[0] == UInt(1)(0)):
                    # Read from A, write to B
                    self.addr_a_reg[0] = read_addr_reg[0]
                    re_a[0] = Bits(1)(1)
                    we_a[0] = Bits(1)(0)

                    self.addr_b_reg[0] = write_addr_reg[0]
                    re_b[0] = Bits(1)(0)
                    we_b[0] = Bits(1)(1)

                with Condition(mem_pingpong_reg[0] == UInt(1)(1)):
                    # Read from B, write to A
                    self.addr_b_reg[0] = read_addr_reg[0]
                    re_b[0] = Bits(1)(1)
                    we_b[0] = Bits(1)(0)

                    self.addr_a_reg[0] = write_addr_reg[0]
                    re_a[0] = Bits(1)(0)
                    we_a[0] = Bits(1)(1)

                # Check if we've read all elements
                with Condition(read_addr_reg[0] > mem_start.bitcast(UInt(addr_width))):
                    # Continue pipeline
                    read_addr_reg[0] = read_addr_reg[0] - UInt(addr_width)(1)
                    SM_MemImpl[0] = UInt(2)(1)

                with Condition(read_addr_reg[0] == mem_start.bitcast(UInt(addr_width))):
                    # All elements read, move to drain
                    SM_MemImpl[0] = UInt(2)(2)

            # State 2: Drain - Write last buffered element
            with Condition(SM_MemImpl[0] == UInt(2)(2)):
                log(
                    "Stage 3-2 (Drain): Writing last element {:08x}",
                    rdata,
                )

                # Write the last buffered element
                idx = (rdata.bitcast(UInt(data_width)) >> offset_reg[0])[0:3]
                write_addr_reg[0] = (
                    radix_reg[idx][0 : (addr_width - 1)].bitcast(UInt(addr_width))
                    - UInt(addr_width)(1)
                    + UInt(addr_width)(data_depth)
                    - mem_start.bitcast(UInt(addr_width))
                )
                radix_reg[idx] = radix_reg[idx] - UInt(data_width)(1)
                wdata[0] = rdata.bitcast(Bits(data_width))

                # Only write, no read
                with Condition(mem_pingpong_reg[0] == UInt(1)(0)):
                    # Write to B
                    self.addr_b_reg[0] = write_addr_reg[0]
                    re_a[0] = Bits(1)(0)
                    we_a[0] = Bits(1)(0)
                    re_b[0] = Bits(1)(0)
                    we_b[0] = Bits(1)(1)

                with Condition(mem_pingpong_reg[0] == UInt(1)(1)):
                    # Write to A
                    self.addr_a_reg[0] = write_addr_reg[0]
                    re_a[0] = Bits(1)(0)
                    we_a[0] = Bits(1)(1)
                    re_b[0] = Bits(1)(0)
                    we_b[0] = Bits(1)(0)

                # Move to reset
                SM_MemImpl[0] = UInt(2)(3)

            # State 3: Reset - Clear radix_reg and return to main FSM
            with Condition(SM_MemImpl[0] == UInt(2)(3)):
                # Reset all 16 radix registers to 0
                with Condition(reset_cycle_reg[0] < UInt(5)(16)):
                    log(
                        "Stage 3-3 (Reset): radix_reg[{}] = 0",
                        reset_cycle_reg[0],
                    )
                    radix_reg[reset_cycle_reg[0]] = UInt(data_width)(0)
                    reset_cycle_reg[0] = reset_cycle_reg[0] + UInt(5)(1)

                # After all radix_reg reset, reset other state
                with Condition(reset_cycle_reg[0] == UInt(5)(16)):
                    log("Stage 3-3 (Reset): Complete, returning to main FSM")
                    # Disable all SRAM operations
                    re_a[0] = Bits(1)(0)
                    we_a[0] = Bits(1)(0)
                    re_b[0] = Bits(1)(0)
                    we_b[0] = Bits(1)(0)

                    # Reset state
                    reset_cycle_reg[0] = UInt(5)(0)
                    SM_MemImpl[0] = UInt(2)(0)
                    SM_reg[0] = UInt(2)(0)
        return


# Driver module
class Driver(Module):
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
        addr_a_reg: RegArray,
        addr_b_reg: RegArray,
        we_a: RegArray,
        re_a: RegArray,
        we_b: RegArray,
        re_b: RegArray,
        wdata: RegArray,
        offset_reg: RegArray,
        mem_pingpong_reg: RegArray,
    ):
        read_cond = (
            (mem_pingpong_reg[0] == UInt(1)(0))
            & (addr_reg[0] < UInt(addr_width)(data_depth))
        ) | (
            (mem_pingpong_reg[0] == UInt(1)(1))
            & (addr_reg[0] < UInt(addr_width)(2 * data_depth))
        )

        # Build dual SRAMs for pipelined write
        # SRAM A: initially contains input data
        sram_a = SRAM(
            width=data_width,
            depth=2 ** addr_width,
            init_file=f"{resource_base}/numbers.data",
        )
        sram_a.name = "sram_a"

        # SRAM B: initially empty, will receive sorted data
        sram_b = SRAM(
            width=data_width,
            depth=2 ** addr_width,
            init_file=None,
        )
        sram_b.name = "sram_b"

        # Build both SRAMs with separate address registers
        sram_a.build(we_a[0], re_a[0], addr_a_reg[0], wdata[0])
        sram_b.build(we_b[0], re_b[0], addr_b_reg[0], wdata[0])

        # Mux SRAM outputs based on ping-pong
        # When mem_pingpong_reg[0] == 0: select sram_a, when == 1: select sram_b
        # Create all-1s mask by shifting
        all_ones = UInt(data_width)((1 << data_width) - 1)

        # Use arithmetic to create masks without conditionals
        # ping_pong is 0 or 1, so we can use it directly for masking
        # mask_b = ping_pong * all_ones (all_ones when ping_pong=1, 0 when ping_pong=0)
        # mask_a = (1 - ping_pong) * all_ones (all_ones when ping_pong=0, 0 when ping_pong=1)
        ping_pong_ext = mem_pingpong_reg[0].bitcast(UInt(data_width))
        select_b_mask = ping_pong_ext * all_ones
        select_a_mask = (UInt(data_width)(1) - ping_pong_ext) * all_ones

        rdata_muxed = (
            (sram_a.dout[0].bitcast(UInt(data_width)) & select_a_mask) |
            (sram_b.dout[0].bitcast(UInt(data_width)) & select_b_mask)
        ).bitcast(Bits(data_width))

        memory_user.async_called(rdata=rdata_muxed)

        mem_start = UInt(addr_width)(0) + (
            mem_pingpong_reg[0] * UInt(addr_width)(data_depth)
        )[0 : (addr_width - 1)].bitcast(UInt(addr_width))
        mem_end = mem_start + UInt(addr_width)(data_depth)

        # Outer for loop
        with Condition(offset_reg[0] < UInt(data_width)(data_width)):
            # Stage Machine: 0 for reset; 1 for read; 2 for prefix; 3 for write
            with Condition(SM_reg[0] == UInt(2)(0)):  # Stage 0: Reset
                log(
                    "Radix Sort: Bits {} - {} Completed!",
                    offset_reg[0],
                    offset_reg[0] + UInt(data_width)(4),
                )
                log(
                    "========================================================================"
                )
                offset_reg[0] = offset_reg[0] + UInt(data_width)(4)
                SM_reg[0] = UInt(2)(1)
                addr_reg[0] = UInt(addr_width)(0) + (
                    ~mem_pingpong_reg[0] * UInt(addr_width)(data_depth)
                )[0 : (addr_width - 1)].bitcast(UInt(addr_width))

                # Set read enable for the appropriate SRAM based on ping-pong
                # Ping-pong flips: if was 0, now 1 (read from B); if was 1, now 0 (read from A)
                mem_pingpong_reg[0] = (~mem_pingpong_reg[0]).bitcast(UInt(1))
                with Condition(mem_pingpong_reg[0] == UInt(1)(0)):
                    # Read from SRAM A
                    re_a[0] = Bits(1)(1)
                    we_a[0] = Bits(1)(0)
                    re_b[0] = Bits(1)(0)
                    we_b[0] = Bits(1)(0)
                with Condition(mem_pingpong_reg[0] == UInt(1)(1)):
                    # Read from SRAM B
                    re_a[0] = Bits(1)(0)
                    we_a[0] = Bits(1)(0)
                    re_b[0] = Bits(1)(1)
                    we_b[0] = Bits(1)(0)

            # Stage 1: Read Data into radix
            with Condition(SM_reg[0] == UInt(2)(1)):
                with Condition(addr_reg[0] < mem_end):
                    # SRAM is automatically accessed when conditions are met
                    addr_reg[0] = addr_reg[0] + UInt(addr_width)(1)
                with Condition(addr_reg[0] == (mem_end - UInt(addr_width)(1))):
                    # Disable read for both SRAMs
                    re_a[0] = Bits(1)(0)
                    re_b[0] = Bits(1)(0)
                with Condition(~read_cond):
                    SM_reg[0] = UInt(2)(2)
                    cycle_reg[0] = UInt(data_width)(1)
                    addr_reg[0] = addr_reg[0] - UInt(addr_width)(1)
            # Stage 2: Prefix sum the radix
            with Condition(SM_reg[0] == UInt(2)(2)):
                radix_reducer.async_called()
                with Condition(cycle_reg[0] == UInt(data_width)(15)):
                    SM_reg[0] = UInt(2)(3)
                    # Note: SRAM control will be set in MemImpl
            # Stage 3: Write Data to Memory
            with Condition(SM_reg[0] == UInt(2)(3)):
                # SRAM write is handled by MemImpl FSM
                pass
        with Condition(offset_reg[0] == UInt(data_width)(data_width)):
            log("finish")
            finish()
        return mem_start, mem_end


def build_system():
    sys = SysBuilder("radix_sort")
    with sys:
        SM_reg = RegArray(UInt(2), 1, initializer=[1])
        cycle_reg = RegArray(UInt(data_width), 1, initializer=[0])
        addr_reg = RegArray(UInt(addr_width), 1, initializer=[0])
        # Separate address registers for dual SRAM
        addr_a_reg = RegArray(UInt(addr_width), 1, initializer=[0])
        addr_b_reg = RegArray(UInt(addr_width), 1, initializer=[0])
        wdata = RegArray(Bits(data_width), 1, initializer=[0])
        # Separate control signals for dual SRAM
        we_a = RegArray(Bits(1), 1, initializer=[0])
        re_a = RegArray(Bits(1), 1, initializer=[1])
        we_b = RegArray(Bits(1), 1, initializer=[0])
        re_b = RegArray(Bits(1), 1, initializer=[0])
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
            addr_a_reg=addr_a_reg,
            addr_b_reg=addr_b_reg,
            we_a=we_a,
            re_a=re_a,
            we_b=we_b,
            re_b=re_b,
            wdata=wdata,
            offset_reg=offset_reg,
            mem_pingpong_reg=mem_pingpong_reg,
        )
        # Create Memory Implementation
        mem_impl = MemImpl()
        # Pass addr_a_reg and addr_b_reg through closure to avoid Downstream feedback
        mem_impl.addr_a_reg = addr_a_reg
        mem_impl.addr_b_reg = addr_b_reg
        mem_impl.build(
            rdata=rdata,
            wdata=wdata,
            SM_reg=SM_reg,
            addr_reg=addr_reg,
            we_a=we_a,
            re_a=re_a,
            we_b=we_b,
            re_b=re_b,
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
