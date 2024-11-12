import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils

pred_data = Record({ (32, 32): ('validity', Bits), (0, 31): ('payload', Bits) })

state_bits = 8
state_init_a = UInt(state_bits)(1)
state_init_b = UInt(state_bits)(2)
state_sort   = UInt(state_bits)(3)
state_read   = UInt(state_bits)(4)
state_idle   = UInt(state_bits)(5)

n = 2048
addr_bits = n.bit_length()
addr_type = UInt(addr_bits)

class RegisterWriter(Module):

    def __init__(self):
        super().__init__(ports={'rdata': Port(Bits(32))})
        self.name = 'reg_writer'

    @module.combinational
    def build(self):
        with Condition(self.rdata.valid()):
            self.rdata.pop()



class Sorter(Module):

    def __init__(self):
        super().__init__(ports={})
        self.name = 'sort'

    @module.combinational
    def build(self, block_size, block_start, from_ptr, to_ptr, writer):
        state = RegArray(UInt(state_bits), 1, initializer=[1])
        k = RegArray(addr_type, 1, initializer=[0])
        pred = RegArray(UInt(1), 1, initializer=[0])
        idx = RegArray(addr_type, 2, initializer=[0, 0])

        with Condition(state[0] == state_init_a):
            log("[sort.init] for block.size: {}, block.start: {}, and 1st element", block_size[0], block_start[0])
            state[0] = state_init_b
            k[0] = addr_type(0)
            idx[0] = addr_type(0)
            # TODO(@were): write to reg[0], by memory re=1, lineno=(block.start + 0 + from[0]).

        with Condition(state[0] == state_init_b):
            state[0] = state_idle
            log("[sort.init] 2nd element")
            idx[1] = addr_type(0)
            # TODO(@were): write to reg[1], by memory re=1, lineno=(block.start + (block.size / 2) + from[0]).

        # Idle for a cycle to wait the memory write data to reg[b].
        with Condition(state[0] == state_idle):
            state[0] = state_sort

        with Condition(state[0] == state_sort):
            # TODO(@were): Replace "0" with comparison later.
            pred[0] = UInt(1)(0)
            state[0] = state_read
            k[0] = k[0] + addr_type(1)
            log("[loop.k++ ] {}", k[0])
            # TODO(@were): memory we=1, lineno=(block.start + k[0] + to[0]).

        with Condition(state[0] == state_read):
            # TODO(@were): memory re=(index[pred[0]] < (block.size / 2)), lineno=(block.start + index[pred[0]] + (block.size / 2) * pred[0] + from[0]).
            log("[sort.fill] refill the popped element")
            with Condition(k[0] < block_size[0]):
                state[0] = state_sort
            with Condition(k[0] == block_size[0]):
                new_start = block_start[0] + block_size[0]
                block_start[0] = new_start
                state[0] = state_init_a
                log("[loop.next] block.start: {}", new_start)

        half_block = block_size[0] >> addr_type(1)

        we = state[0] == state_sort

        re = state[0].case({
            state_init_a: UInt(1)(1),
            state_init_b: UInt(1)(1),
            state_sort: UInt(1)(0),
            state_read: idx[pred[0]] < half_block,
            None: UInt(1)(0)
        })

        addr = state[0].case({
            state_init_a: block_start[0] + from_ptr[0],
            state_init_b: block_start[0] + half_block + from_ptr[0],
            state_sort: block_start[0] + k[0] + to_ptr[0],
            state_read: block_start[0] + idx[pred[0]] + pred[0].select(half_block, addr_type(0)) + from_ptr[0],
            None: addr_type(0)
        })

        log('[loop.sram] addr: {}', addr)

        wdata = Bits(32)(0)

        sram = SRAM(32, n * 2, 'init.hex')
        sram.build(we, re, addr, wdata, writer)
        with Condition(re):
            sram.bound.async_called()


class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, sorter, block_size, block_start, from_ptr, to_ptr):

        with Condition(block_start[0] == addr_type(n)):
            with Condition(block_size[0] == addr_type(n)):
                finish()
            block_size[0] = block_size[0] << addr_type(1)
            block_start[0] = addr_type(0)
            from_ptr[0] = to_ptr[0]
            to_ptr[0] = from_ptr[0]
            log("[loop.2x  ] block.size, reset block.start, swap from/to")


        sorter.async_called()

def test_sort():
    sys = SysBuilder('merge_sort')
    with sys:

        block_size = RegArray(addr_type, 1, initializer=[2])
        block_start = RegArray(addr_type, 1, initializer=[0])
        from_ptr = RegArray(addr_type, 1, initializer=[0])
        to_ptr = RegArray(addr_type, 1, initializer=[n // 2])

        writer = RegisterWriter()
        writer.build()

        sorter = Sorter()
        sorter.build(block_size, block_start, from_ptr, to_ptr, writer)

        driver = Driver()
        driver.build(sorter, block_size, block_start, from_ptr, to_ptr)


    config = backend.config(
            sim_threshold=100000,
            idle_threshold=100000,
            resource_base=f'{utils.repo_path()}/examples/merge-sort/input',
            verilog=utils.has_verilator())

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    # print(raw)
    # check(raw)

    # if utils.has_verilator():
    #     raw = utils.run_verilator(verilator_path)
    #     check(raw)

if __name__ == "__main__":
    test_sort()
