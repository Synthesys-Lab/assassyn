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
        # p[0] = (span[0][0:15].bitcast(Int(16)) * Int(16)(FFT_SIZE+1)).bitcast(Int(32)) + odd[0]
        # log("p: {} = span_{}, odd_{}, state_{}", p[0], span[0], odd[0], state[0])

class Calculate_loop(Module):
    def __init__(self):
        super().__init__(
            ports={'In_full_flag': Port(Bits(1))},
        )

    @module.combinational
    def build(self, state: Array, rootindex: Array):
        In_full_flag = self.pop_all_ports(True)
        # state = RegArray(Int(32), 1)
        con = Bits(1)(0)
        con = state[0] < SRAM_USER.S9
        # full_flag = state[0] == Int(32)(I_MAX)
        
        state_value = Bits(4)(0)
        state_value = con.select((state[0].bitcast(UInt(4)) + UInt(4)(1)).bitcast(Bits(4)) , Bits(4)(0))
        
        
        con = (state[0] == SRAM_USER.S7) & (rootindex[0].bitcast(UInt(32)) == UInt(32)(0))
        state_value = con.select(SRAM_USER.S9, state_value)
        
        state[0] = state_value.bitcast(Bits(4))
        
        
        # state[0] = con.select((state[0].bitcast(Int(32)) + Int(32)(1)) , Int(32)(0)) #xxx
        
        # with Condition(rootindex[0] == Int(32)(0)): #cond
        #     state[0] = SRAM_USER.S9.bitcast(Int(32)) #yyy
        


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
        even_reg[0] = (user_state[0] == SRAM_USER.S1).select( rdata.bitcast(Bits(64)), even_reg[0]) # read
        odd_reg[0] = (user_state[0] == SRAM_USER.S2).select( rdata.bitcast(Bits(64)), odd_reg[0])
        twid_reg[0] = (user_state[0] == SRAM_USER.S7).select( rdata.bitcast(Bits(64)), twid_reg[0])

        log(" even_reg: {} | odd_reg: {} | twid_reg: {}",  even_reg[0], odd_reg[0], twid_reg[0])

class External_loop(Module):
    def __init__(self):
        super().__init__(
            ports={ },
        ) 
        
    @module.combinational
    def build(self, calculate_loop: Calculate_loop, state: Array,
              memuser: Memuser, init_file, even_reg: Array, 
              odd_reg: Array, twid_reg: Array, rootindex: Array,
              out: Array):
        
        odd = RegArray(UInt(32), 1, initializer=[0])
        span = RegArray(UInt(32), 1, initializer=[FFT_SIZE >> 1])
        log0 = RegArray(UInt(32), 1, initializer=[0])
        even = RegArray(UInt(32), 1, initializer=[0])
        temp1 = RegArray(Bits(32), 1, initializer=[0])
        temp2 = RegArray(Bits(32), 1, initializer=[0])
        temp3 = RegArray(Bits(32), 1, initializer=[0])
        temp4 = RegArray(Bits(32), 1, initializer=[0])
        out1 = RegArray(Bits(64), 1, initializer=[0])
        out2 = RegArray(Bits(64), 1, initializer=[0])
        # rootindex = RegArray(Int(32), 1, initializer=[0])
        temp5 = RegArray(Bits(32), 1, initializer=[0])
        temp6 = RegArray(Bits(32), 1, initializer=[0])
        out3 = RegArray(Bits(64), 1, initializer=[0])
        
        re = Bits(1)(0)
        we = Bits(1)(0)
        re = (state[0] == SRAM_USER.S1)|(state[0] == SRAM_USER.S2)|(state[0] == SRAM_USER.S7)
        we = (state[0] == SRAM_USER.S4)|(state[0] == SRAM_USER.S5)|(state[0] == SRAM_USER.S9)
        
        address_wire = Bits(ADDR_WIDTH)(0)
        
        # with Condition(state[0] == SRAM_USER.S9):
        #     with Condition(span[0] == Int(32)(0)):
        #         finish()
            
        #     con = Bits(1)(0)
        #     con = odd[0] == Int(32)(FFT_SIZE)
        #     odd[0] = con.select(Int(32)(0), (odd[0].bitcast(Int(32)) + Int(32)(1)))

        #     span[0] = con.select((span[0].bitcast(Int(32)) >> Int(32)(1)), span[0])
        #     log[0] = con.select((log[0].bitcast(Int(32)) + Int(32)(1)), log[0])
        
        with Condition(state[0] == SRAM_USER.S0):
            log("state 0")
            odd[0] = (odd[0].bitcast(UInt(32)) | span[0].bitcast(UInt(32))).bitcast(UInt(32))
            even[0] = (odd[0].bitcast(UInt(32)) ^ span[0].bitcast(UInt(32))).bitcast(UInt(32))
        # with Condition(state[0] == SRAM_USER.S1):
        #     address_wire = even[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH))
        # with Condition(state[0] == SRAM_USER.S2):
        #     address_wire = odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH))
        with Condition(state[0] == SRAM_USER.S3):
            log("state 3")
            temp1 = Int(32)(0)
            temp1 = (even_reg[0][32:63].bitcast(Int(32)) + odd_reg[0][32:63].bitcast(Int(32))).bitcast(Int(32))
            temp2 = Int(32)(0)
            temp2 = (even_reg[0][32:63].bitcast(Int(32)) - odd_reg[0][32:63].bitcast(Int(32))).bitcast(Int(32))
            temp3 = Int(32)(0)
            temp3 = (even_reg[0][0:31].bitcast(Int(32)) + odd_reg[0][0:31].bitcast(Int(32))).bitcast(Int(32))
            temp4 = Int(32)(0)
            temp4 = (even_reg[0][0:31].bitcast(Int(32)) - odd_reg[0][0:31].bitcast(Int(32))).bitcast(Int(32))
            log("state 3.5")
            out1[0] = concat(temp1.bitcast(Bits(32)), temp3.bitcast(Bits(32))).bitcast(Bits(64)) # even
            out2[0] = concat(temp2.bitcast(Bits(32)), temp4.bitcast(Bits(32))).bitcast(Bits(64)) # odd
        with Condition(state[0] == SRAM_USER.S4):
            log("state 4")
            # address_wire = even[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH))
            out[0] = out1[0].bitcast(Bits(64))
        with Condition(state[0] == SRAM_USER.S5):
            log("state 5")
            # address_wire = odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH))
            out[0] = out2[0].bitcast(Bits(64))
        
        with Condition(state[0] == SRAM_USER.S6):
            log("state 6")
            rootindex[0] = ((even[0].bitcast(UInt(32)) << log0[0].bitcast(UInt(32))) & (UInt(32)(FFT_SIZE - 1))).bitcast(UInt(32))
            # with Condition(rootindex[0] == Int(32)(0)):
            #     state[0] = SRAM_USER.S8.bitcast(Int(32))
        # with Condition(state[0] == SRAM_USER.S7):
        #     address_wire = (rootindex[0][0:ADDR_WIDTH-1].bitcast(Int(32)) + Int(32)(1024)).bitcast(Bits(ADDR_WIDTH))
        with Condition(state[0] == SRAM_USER.S8):
            log("state 8")
            temp5 = Int(32)(0)
            temp5 = (twid_reg[0][32:63].bitcast(Int(32)) * odd_reg[0][32:63].bitcast(Int(32)) - twid_reg[0][0:31].bitcast(Int(32)) * odd_reg[0][0:31].bitcast(Int(32)))[0:31].bitcast(Int(32))
            temp6 = Int(32)(0)
            temp6 = (twid_reg[0][32:63].bitcast(Int(32)) * odd_reg[0][0:31].bitcast(Int(32)) - twid_reg[0][0:31].bitcast(Int(32)) * odd_reg[0][32:63].bitcast(Int(32)))[0:31].bitcast(Int(32))
            out3[0] = concat(temp5.bitcast(Bits(32)), temp6.bitcast(Bits(32))).bitcast(Bits(64))
        with Condition(state[0] == SRAM_USER.S9):
            log("state 9")
            with Condition(rootindex[0] != UInt(32)(0)):
                # address_wire = odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH))
                out[0] = out3[0].bitcast(Bits(64))
            
            with Condition(span[0] == UInt(32)(0)):
                finish()
            
            con = Bits(1)(0)
            con = odd[0] == (UInt(32)(FFT_SIZE) - UInt(32)(1))
            odd[0] = con.select(UInt(32)(0), (odd[0].bitcast(UInt(32)) + UInt(32)(1)))
            span[0] = con.select((span[0].bitcast(UInt(32)) >> UInt(32)(1)), span[0])
            log0[0] = con.select((log0[0].bitcast(UInt(32)) + UInt(32)(1)), log0[0])
        
        
        address_wire = state[0].case({
            SRAM_USER.S1: even[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S2: odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S4: even[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S5: odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S7: (rootindex[0][0:ADDR_WIDTH-1].bitcast(UInt(ADDR_WIDTH)) + UInt(ADDR_WIDTH)(1024)).bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S9: odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            None: Bits(ADDR_WIDTH)(0)
        })
        sram = SRAM(64, 2**ADDR_WIDTH, init_file)
        sram.build(we, re, address_wire, out[0].bitcast(Bits(64)), memuser)
        log("state: {}", state[0])
        with Condition((state[0] == SRAM_USER.S1) | (state[0] == SRAM_USER.S2) | (state[0] == SRAM_USER.S4) | (state[0] == SRAM_USER.S5) | (state[0] == SRAM_USER.S7) | (state[0] == SRAM_USER.S9)):
            sram.bound.async_called()
        
        log("span: {} odd: {} | log0: {}", span[0], odd[0], log0[0])
        log("address_wire: {} even_reg: {} | odd_reg: {}", address_wire, even_reg[0], odd_reg[0])
        log("we: {} | re: {} | out: {}", we, re, out[0].bitcast(Bits(64)))
        
        
        full_flag = Bits(1)(0)
        full_flag = odd[0] == (UInt(32)(FFT_SIZE)-UInt(32)(1))
        
        calculate_loop.async_called( In_full_flag = full_flag.bitcast(Bits(1)))
        
        return span, odd



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
        
        state = RegArray(Bits(4), 1)
        rootindex = RegArray(UInt(32), 1, initializer=[0])
        
        calculate_loop = Calculate_loop()
        calculate_loop.build(state, rootindex)

        out = RegArray(Bits(64), 1)

        memuser = Memuser()
        memuser.build(state,even_reg,odd_reg,twid_reg)
        
        external_loop = External_loop()
        span, odd = external_loop.build(calculate_loop, state, memuser, init_file, even_reg, odd_reg, twid_reg, rootindex, out)

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