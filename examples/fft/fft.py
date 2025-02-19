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
FFT_SIZE = 16 # 最终1024

ADDR_WIDTH = 11

class SRAM_USER:
    S0 = Int(32)(0)
    S1 = Int(32)(1)
    S2 = Int(32)(2)
    S3 = Int(32)(3)
    S4 = Int(32)(4)
    S5 = Int(32)(5)
    S6 = Int(32)(6)
    S7 = Int(32)(7)
    S8 = Int(32)(8)
    S9 = Int(32)(9)

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
    def build(self, rootindex: Array):
        In_full_flag = self.pop_all_ports(True)
        state = RegArray(Int(32), 1)
        con = Bits(1)(0)
        con = state[0] < Int(32)(9)
        full_flag = state[0] == Int(32)(I_MAX)
        
        state_value = Int(32)(0)
        state_value = con.select((state[0].bitcast(Int(32)) + Int(32)(1)) , Int(32)(0))
        
        con = rootindex[0].bitcast(Int(32)) == Int(32)(0)
        state_value = con.select(Int(32)(9), state_value)
        
        state[0] = state_value.bitcast(Int(32))
        
        # state[0] = con.select((state[0].bitcast(Int(32)) + Int(32)(1)) , Int(32)(0)) #xxx
        
        # with Condition(rootindex[0] == Int(32)(0)): #cond
        #     state[0] = SRAM_USER.S9.bitcast(Int(32)) #yyy
        
        return state


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
        even_reg[0] = (user_state[0] == SRAM_USER.S1).select( rdata.bitcast(Int(64)), even_reg[0]) # read
        odd_reg[0] = (user_state[0] == SRAM_USER.S2).select( rdata.bitcast(Int(64)), odd_reg[0])
        twid_reg[0] = (user_state[0] == SRAM_USER.S7).select( rdata.bitcast(Int(64)), twid_reg[0])

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
        
        odd = RegArray(Int(32), 1, initializer=[0])
        span = RegArray(Int(32), 1, initializer=[FFT_SIZE >> 1])
        log = RegArray(Int(32), 1, initializer=[0])
        even = RegArray(Int(32), 1, initializer=[0])
        temp1 = RegArray(Int(32), 1, initializer=[0])
        temp2 = RegArray(Int(32), 1, initializer=[0])
        temp3 = RegArray(Int(32), 1, initializer=[0])
        temp4 = RegArray(Int(32), 1, initializer=[0])
        out1 = RegArray(Int(32), 1, initializer=[0])
        out2 = RegArray(Int(32), 1, initializer=[0])
        # rootindex = RegArray(Int(32), 1, initializer=[0])
        temp5 = RegArray(Int(32), 1, initializer=[0])
        temp6 = RegArray(Int(32), 1, initializer=[0])
        out3 = RegArray(Int(32), 1, initializer=[0])
        
        re = Bits(1)(0)
        we = Bits(1)(0)
        re = (state[0] == SRAM_USER.S1)|(state[0] == SRAM_USER.S2)|(state[0] == SRAM_USER.S7)
        we = (state[0] == SRAM_USER.S4)|(state[0] == SRAM_USER.S5)|(state[0] == SRAM_USER.S9)
        
        address_wire = Bits(ADDR_WIDTH)(0)
        
        log("xxx")
        
        # with Condition(state[0] == SRAM_USER.S9):
        #     with Condition(span[0] == Int(32)(0)):
        #         finish()
            
        #     con = Bits(1)(0)
        #     con = odd[0] == Int(32)(FFT_SIZE)
        #     odd[0] = con.select(Int(32)(0), (odd[0].bitcast(Int(32)) + Int(32)(1)))

        #     span[0] = con.select((span[0].bitcast(Int(32)) >> Int(32)(1)), span[0])
        #     log[0] = con.select((log[0].bitcast(Int(32)) + Int(32)(1)), log[0])
        
        with Condition(state[0] == SRAM_USER.S0):
            odd[0] = (odd[0].bitcast(Int(32)) | span[0].bitcast(Int(32))).bitcast(Int(32))
            even[0] = (odd[0].bitcast(Int(32)) ^ span[0].bitcast(Int(32))).bitcast(Int(32))
        # with Condition(state[0] == SRAM_USER.S1):
        #     address_wire = even[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH))
        # with Condition(state[0] == SRAM_USER.S2):
        #     address_wire = odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH))
        with Condition(state[0] == SRAM_USER.S3):
            temp1[0] = even_reg[0][32:63].bitcast(Int(32)) + odd_reg[0][32:63].bitcast(Int(32))
            temp2[0] = even_reg[0][32:63].bitcast(Int(32)) - odd_reg[0][32:63].bitcast(Int(32))
            temp3[0] = even_reg[0][0:31].bitcast(Int(32)) + odd_reg[0][0:31].bitcast(Int(32))
            temp4[0] = even_reg[0][0:31].bitcast(Int(32)) - odd_reg[0][0:31].bitcast(Int(32))
            out1[0] = concat(temp1[0].bitcast(Int(32)), temp3[0].bitcast(Int(32))).bitcast(Int(32)) # even
            out2[0] = concat(temp2[0].bitcast(Int(32)), temp4[0].bitcast(Int(32))).bitcast(Int(32)) # odd
        with Condition(state[0] == SRAM_USER.S4):
            # address_wire = even[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH))
            out[0] = out1[0].bitcast(Int(64))
        with Condition(state[0] == SRAM_USER.S5):
            # address_wire = odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH))
            out[0] = out2[0].bitcast(Int(64))
        
        with Condition(state[0] == SRAM_USER.S6):
            rootindex[0] = ((even[0].bitcast(Int(32)) << log[0].bitcast(Int(32))) & (Int(32)(FFT_SIZE - 1))).bitcast(Int(32))
            # with Condition(rootindex[0] == Int(32)(0)):
            #     state[0] = SRAM_USER.S8.bitcast(Int(32))
        # with Condition(state[0] == SRAM_USER.S7):
        #     address_wire = (rootindex[0][0:ADDR_WIDTH-1].bitcast(Int(32)) + Int(32)(1024)).bitcast(Bits(ADDR_WIDTH))
        with Condition(state[0] == SRAM_USER.S8):
            temp5[0] = (twid_reg[0][32:63].bitcast(Int(32)) * odd_reg[0][32:63].bitcast(Int(32)) - twid_reg[0][0:31].bitcast(Int(32)) * odd_reg[0][0:31].bitcast(Int(32))).bitcast(Int(32))
            temp6[0] = (twid_reg[0][32:63].bitcast(Int(32)) * odd_reg[0][0:31].bitcast(Int(32)) - twid_reg[0][0:31].bitcast(Int(32)) * odd_reg[0][32:63].bitcast(Int(32))).bitcast(Int(32))
            out3[0] = concat(temp5[0].bitcast(Int(32)), temp6[0].bitcast(Int(32))).bitcast(Int(32))
        with Condition(state[0] == SRAM_USER.S9):
            with Condition(rootindex[0] != Int(32)(0)):
                # address_wire = odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH))
                out[0] = out3[0].bitcast(Int(64))
            
            with Condition(span[0] == Int(32)(0)):
                finish()
            
            con = Bits(1)(0)
            con = odd[0] == Int(32)(FFT_SIZE)
            odd[0] = con.select(Int(32)(0), (odd[0].bitcast(Int(32)) + Int(32)(1)))
            span[0] = con.select((span[0].bitcast(Int(32)) >> Int(32)(1)), span[0])
            log[0] = con.select((log[0].bitcast(Int(32)) + Int(32)(1)), log[0])
        
        log("yyy")
        
        address_wire = state[0].case({
            SRAM_USER.S1: even[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S2: odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S4: even[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S5: odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S7: (rootindex[0][0:ADDR_WIDTH-1].bitcast(Int(32)) + Int(32)(1024)).bitcast(Bits(ADDR_WIDTH)),
            SRAM_USER.S9: odd[0][0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)),
            None: Bits(ADDR_WIDTH)(0)
        })
        sram = SRAM(64, 2**ADDR_WIDTH, init_file)
        sram.build(we, re, address_wire, out[0].bitcast(Bits(64)), memuser)
        with Condition(state[0] == SRAM_USER.S1 | state[0] == SRAM_USER.S2 | state[0] == SRAM_USER.S4 | state[0] == SRAM_USER.S5 | state[0] == SRAM_USER.S7 | state[0] == SRAM_USER.S9):
            sram.bound.async_called()
        
        # 'Array' object is not callable?
        # log("address_wire: {} even_reg: {} | odd_reg: {}", address_wire, even_reg[0], odd_reg[0])
        # log("we: {} | re: {} | out: {}", we, re, out[0].bitcast(Bits(64)))
        
        full_flag = Bits(1)(0)
        full_flag = odd[0] == (Int(32)(FFT_SIZE)-Int(32)(1))
        
        calculate_loop.async_called( In_full_flag = full_flag.bitcast(Bits(1)))
        
        log("zzz")
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
        even_reg  = RegArray(Int(64), 1)
        odd_reg = RegArray(Int(64), 1)
        twid_reg = RegArray(Int(64), 1)
        
        rootindex = RegArray(Int(32), 1, initializer=[0])
        
        calculate_loop = Calculate_loop()
        state = calculate_loop.build(rootindex)

        out = RegArray(Int(64), 1)

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
        sim_threshold=200,
        idle_threshold=200,
        resource_base= f'{utils.repo_path()}/examples/fft/data',
    )
    simulator_path, verilator_path = elaborate(sys, **conf)

    raw = utils.run_simulator(simulator_path)
    
    # if verilator_path:
    #     raw = utils.run_verilator(verilator_path)

if __name__ == '__main__':
    test_fft()