import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils
from assassyn.expr import Bind
from assassyn.backend import elaborate
import os
import shutil

from assassyn.frontend import *
from assassyn.backend import *
from assassyn import utils

I_MAX = 5
FFT_SIZE = 1024

ADDR_WIDTH = 12

class SRAM_USER:
    S0 = Bits(4)(0)
    S1 = Bits(4)(1)
    S2 = Bits(4)(2)
    S3 = Bits(4)(3)
    S4 = Bits(4)(4)
    S5 = Bits(4)(5)
    S6 = Bits(4)(6)
    S7 = Bits(4)(7)
    S8 = Bits(4)(8)
    S9 = Bits(4)(9)
    S10 = Bits(4)(10)

class Loop_user(Module):

    def __init__(self):
        ports={
            
        }
        super().__init__(
            ports=ports,
        )

    @module.combinational
    def build(self, span: Array, odd: Array, state: Array):
        p = RegArray(Int(32), 1)

class Calculate_loop(Module):
    def __init__(self):
        super().__init__(
            ports={'In_full_flag': Port(Bits(1))},
        )

    @module.combinational
    def build(self, state: Array, rootindex: Array):
        In_full_flag = self.pop_all_ports(True)
        con = Bits(1)(0)
        con = state[0] < SRAM_USER.S10
        
        state_value = Bits(4)(0)
        state_value = con.select((state[0].bitcast(UInt(4)) + UInt(4)(1)).bitcast(Bits(4)) , Bits(4)(0))
        
        
        con = (state[0] == SRAM_USER.S8) & (rootindex[0].bitcast(UInt(32)) == UInt(32)(0))
        state_value = con.select(SRAM_USER.S10, state_value)
        
        state[0] = state_value.bitcast(Bits(4))
        
        


class Memuser(Module):
    def __init__(self):
        super().__init__(
            ports={  'rdata': Port(Bits(64)) ,
                    #'user_sel': Port(Bits(3)) 
                    },
        ) 
        
    @module.combinational
    def build(self, user_state: Array, even_reg: Array,
              odd_reg: Array, twid_reg: Array):

        rdata = self.pop_all_ports(True)
        
        
        even_reg[0] = (user_state[0] == SRAM_USER.S2).select( rdata.bitcast(Bits(64)), even_reg[0]) # read
        odd_reg[0] = (user_state[0] == SRAM_USER.S3).select( rdata.bitcast(Bits(64)), odd_reg[0])
        twid_reg[0] = (user_state[0] == SRAM_USER.S9).select( rdata.bitcast(Bits(64)), twid_reg[0])

        log("        state: {} | even_reg: {} | odd_reg: {} | twid_reg: {}",  user_state[0], even_reg[0], odd_reg[0], twid_reg[0])

class External_loop(Module):
    def __init__(self):
        super().__init__(
            ports={ },
        ) 
        self.odd = RegArray(UInt(32), 1, initializer=[0])
        self.span = RegArray(UInt(32), 1, initializer=[FFT_SIZE >> 1])
        self.log0 = RegArray(UInt(32), 1, initializer=[0])
        self.even = RegArray(UInt(32), 1, initializer=[0])
        self.out1 = RegArray(Bits(64), 1, initializer=[0])
        self.out2 = RegArray(Bits(64), 1, initializer=[0])
        self.out3 = RegArray(Bits(64), 1, initializer=[0])
        
    @module.combinational
    def build(self, calculate_loop: Calculate_loop, state: Array,
              memuser: Memuser, init_file, even_reg: Array, 
              odd_reg: Array, twid_reg: Array, rootindex: Array,
              out: Array):
        
        re = Bits(1)(0)
        we = Bits(1)(0)
        re = (state[0] == SRAM_USER.S1)|(state[0] == SRAM_USER.S2)|(state[0] == SRAM_USER.S8)
        we = (state[0] == SRAM_USER.S5)|(state[0] == SRAM_USER.S6)|(state[0] == SRAM_USER.S10)
        
        address_wire = Bits(ADDR_WIDTH)(0)
        
        
        with Condition(state[0] == SRAM_USER.S0):
            odd_temp = UInt(32)(0)
            odd_temp = (self.odd[0].bitcast(UInt(32)) | self.span[0].bitcast(UInt(32))).bitcast(UInt(32))
            self.odd[0] = odd_temp
            self.even[0] = (odd_temp.bitcast(UInt(32)) ^ self.span[0].bitcast(UInt(32))).bitcast(UInt(32))
        with Condition(state[0] == SRAM_USER.S4):
            temp1 = Int(32)(0)
            temp1 = (even_reg[0][32:63].bitcast(Int(32)) + odd_reg[0][32:63].bitcast(Int(32))).bitcast(Int(32))
            temp2 = Int(32)(0)
            temp2 = (even_reg[0][32:63].bitcast(Int(32)) - odd_reg[0][32:63].bitcast(Int(32))).bitcast(Int(32))
            temp3 = Int(32)(0)
            temp3 = (even_reg[0][0:31].bitcast(Int(32)) + odd_reg[0][0:31].bitcast(Int(32))).bitcast(Int(32))
            temp4 = Int(32)(0)
            temp4 = (even_reg[0][0:31].bitcast(Int(32)) - odd_reg[0][0:31].bitcast(Int(32))).bitcast(Int(32))
            self.out1[0] = concat(temp1.bitcast(Bits(32)), temp3.bitcast(Bits(32))).bitcast(Bits(64)) # even
            self.out2[0] = concat(temp2.bitcast(Bits(32)), temp4.bitcast(Bits(32))).bitcast(Bits(64)) # odd
        
        with Condition(state[0] == SRAM_USER.S7):
            rootindex[0] = ((self.even[0].bitcast(UInt(32)) << self.log0[0].bitcast(UInt(32))) & (UInt(32)(FFT_SIZE - 1))).bitcast(UInt(32))
        with Condition(state[0] == SRAM_USER.S9):
            temp5 = Int(32)(0)
            temp5 = (twid_reg[0][32:63].bitcast(Int(32)) * odd_reg[0][32:63].bitcast(Int(32)) - twid_reg[0][0:31].bitcast(Int(32)) * odd_reg[0][0:31].bitcast(Int(32)))[0:31].bitcast(Int(32))
            temp6 = Int(32)(0)
            temp6 = (twid_reg[0][32:63].bitcast(Int(32)) * odd_reg[0][0:31].bitcast(Int(32)) - twid_reg[0][0:31].bitcast(Int(32)) * odd_reg[0][32:63].bitcast(Int(32)))[0:31].bitcast(Int(32))
            self.out3[0] = concat(temp5.bitcast(Bits(32)), temp6.bitcast(Bits(32))).bitcast(Bits(64))
        with Condition(state[0] == SRAM_USER.S10):
            
            with Condition(self.span[0] == UInt(32)(0)):
                log("finish")
                finish()
            
            con = Bits(1)(0)
            con = self.odd[0] == (UInt(32)(FFT_SIZE - 1))
            self.odd[0] = con.select(UInt(32)(0), (self.odd[0].bitcast(UInt(32)) + UInt(32)(1)))
            self.span[0] = con.select((self.span[0].bitcast(UInt(32)) >> UInt(32)(1)), self.span[0])
            self.log0[0] = con.select((self.log0[0].bitcast(UInt(32)) + UInt(32)(1)), self.log0[0])
        
        
        address_wire = state[0].case({
            SRAM_USER.S1: self.even[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S2: self.odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S5: self.even[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S6: self.odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S8: (rootindex[0][0:ADDR_WIDTH-1].bitcast(UInt(ADDR_WIDTH)) + UInt(ADDR_WIDTH)(1024)).bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S10: self.odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            None: Bits(ADDR_WIDTH)(0)
        })
        
        out[0] = state[0].case({
            SRAM_USER.S5: self.out1[0].bitcast(Bits(64)),
            SRAM_USER.S6: self.out2[0].bitcast(Bits(64)),
            SRAM_USER.S10: self.out3[0].bitcast(Bits(64)),
            None: out[0]
        })
        
        sram = SRAM(64, 2**ADDR_WIDTH, init_file)
        sram.build(we, re, address_wire, out[0].bitcast(Bits(64)), memuser)
        sram.bound.async_called()
        
        log("state: {}", state[0])
        log("span: {} odd: {} | even: {} | log0: {}", self.span[0], self.odd[0], self.even[0], self.log0[0])
        log("address_wire: {} ｜ even_reg: {} | odd_reg: {}", address_wire, even_reg[0], odd_reg[0])
        log("we: {} | re: {} | out: {}", we, re, out[0].bitcast(Bits(64)))
        
        
        full_flag = Bits(1)(0)
        full_flag = self.odd[0] == (UInt(32)(FFT_SIZE)-UInt(32)(1))
        
        calculate_loop.async_called( In_full_flag = full_flag.bitcast(Bits(1)))
        
        return self.span, self.odd



class Driver(Module):
    def __init__(self):
        super().__init__(
            ports={},
        )

    @module.combinational
    def build(self, inner_loop: External_loop, user: Loop_user):
        inner_loop.async_called()
        user.async_called()

def test_fft():
    sys =  SysBuilder('fft')
    init_file = 'fft_data.data'
    with sys:
        even_reg  = RegArray(Bits(64), 1)
        odd_reg = RegArray(Bits(64), 1)
        twid_reg = RegArray(Bits(64), 1)
        
        state = RegArray(Bits(4), 1, initializer=[0])
        rootindex = RegArray(UInt(32), 1, initializer=[0])
        
        calculate_loop = Calculate_loop()
        calculate_loop.build(state, rootindex)

        out = RegArray(Bits(64), 1)

        memuser = Memuser()
        memuser.build(state,even_reg,odd_reg,twid_reg)
        
        external_loop = External_loop()
        span, odd = external_loop.build(calculate_loop, state, memuser, init_file, even_reg, odd_reg, twid_reg, rootindex, out)
        sys.expose_on_top(external_loop.odd)
        sys.expose_on_top(external_loop.span)
        sys.expose_on_top(external_loop.log0)
        sys.expose_on_top(external_loop.even)
        sys.expose_on_top(external_loop.out1)
        sys.expose_on_top(external_loop.out2)
        sys.expose_on_top(external_loop.out3)

        loop_user = Loop_user()
        loop_user.build(span, odd, state)

        driver = Driver()
        driver.build(external_loop, loop_user)
    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=100000,
        idle_threshold=200,
        resource_base= f'{utils.repo_path()}/examples/fft/data',
    )
    simulator_path, verilator_path = elaborate(sys, **conf)

    raw = utils.run_simulator(simulator_path)
    
    if verilator_path:
        raw = utils.run_verilator(verilator_path)

if __name__ == '__main__':
    test_fft()