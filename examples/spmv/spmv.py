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

N_SIZE = 5
L_SIZE = 8
I_MAX = N_SIZE - 1
J_MAX = L_SIZE - 1
M_SIZE = N_SIZE * L_SIZE

class SRAM_USER:
    IDLE = Bits(3)(0)
    M1 =   Bits(3)(1)
    M2 =   Bits(3)(2)
    M3 =   Bits(3)(3)
    OUT =  Bits(3)(4)
    JUMP = Bits(3)(5)
    OUT_WAIT = Bits(3)(6)


class SRAM_Master(Module):
    def __init__(self):
        super().__init__(
            ports={ 'Start': Port(Bits(1))},
        ) 
        
    @module.combinational
    def build(self, i: Array, j: Array):
        user_state = RegArray(Bits(3), 1)
        Start = self.pop_all_ports(False)
        SRAM_Master_flag = RegArray(Bits(1), 1)
        log("user_state: {} ", user_state[0])
        SRAM_Master_flag[0] = (user_state[0] == SRAM_USER.M3).bitcast(Bits(1))
        with Condition((user_state[0] == SRAM_USER.M1)|(user_state[0] == SRAM_USER.M2)|(user_state[0] == SRAM_USER.M3)):
            with Condition(user_state[0] == SRAM_USER.M1):
                user_state[0] = SRAM_USER.M2

            with Condition(user_state[0] == SRAM_USER.M2):
                user_state[0] = SRAM_USER.M3

            with Condition(user_state[0] == SRAM_USER.M3):
                user_state[0] = (j[0]==Int(32)(J_MAX)).select(  SRAM_USER.OUT, SRAM_USER.JUMP)

        with Condition(user_state[0] == SRAM_USER.IDLE):
            with Condition(Start == Bits(1)(1)):
                user_state[0] = SRAM_USER.M1

        with Condition(user_state[0] == SRAM_USER.OUT):
            user_state[0] = SRAM_USER.JUMP

        with Condition(user_state[0] == SRAM_USER.JUMP):
            user_state[0] = SRAM_USER.IDLE

        p = Int(32)(0)
        p = (i[0][0:15].bitcast(Int(16)) * Int(16)(J_MAX+1)).bitcast(Int(32)) + j[0]
        log("p: {} = i_{}, j_{}", p, i[0], j[0])

        return SRAM_Master_flag

class External_loop(Module):
    def __init__(self):
        super().__init__(
            ports={'In_full_flag': Port(Bits(1))},
        )

    @module.combinational
    def build(self,sram_master: SRAM_Master,i:Array):
        In_full_flag = self.pop_all_ports(False)
        
        full_flag = RegArray(Bits(1), 1)


        with Condition((In_full_flag == Bits(1)(1))&(full_flag[0] == Bits(1)(0))):
            con = Bits(1)(0)
            con = i[0] < Int(32)(I_MAX)
            full_flag[0] = (i[0] == Int(32)(I_MAX)).bitcast(Bits(1))
            i[0] = con.select((i[0].bitcast(Int(32)) + Int(32)(1)) , Int(32)(0))
        
        log("outterloop----------In_full_flag: {} | i: {} ", In_full_flag,i[0])
        sram_master.async_called(Start = ~full_flag[0])

        

class Internal_loop(Module):
    def __init__(self):
        super().__init__(
            ports={ },
        ) 
        
    @module.combinational
    def build(self, outter_loop: External_loop ,sram_master_flag:Array,j:Array):
        
        
        con = Bits(1)(0)
        full_flag = Bits(1)(0)
        full_flag = (j[0] == (Int(32)(J_MAX))) & sram_master_flag[0]

        with Condition(sram_master_flag[0] == Bits(1)(1)):
            con = j[0] < Int(32)(J_MAX)
            j[0] = con.select((j[0].bitcast(Int(32)) + Int(32)(1)) , Int(32)(0))
        log("innerloop----------sram_master_flag: {} | j: {} |full_flag: {}", sram_master_flag[0],j[0],full_flag)
        outter_loop.async_called( In_full_flag = full_flag)
        




class Driver(Module):
    def __init__(self):
        super().__init__(
            ports={},
        )
 
    @module.combinational
    def build(self, inner_loop: Internal_loop ):
        inner_loop.async_called()
        

def test_spmv():
    sys =  SysBuilder('spmv')
    with sys:
        i = RegArray(Int(32), 1)
        j = RegArray(Int(32), 1)

        sram_master = SRAM_Master()
        sram_master_flag = sram_master.build(i,j)
        
        external_loop = External_loop()
        external_loop.build(sram_master,i)

        internal_loop = Internal_loop()
        internal_loop.build(external_loop,sram_master_flag,j)

        driver = Driver()
        driver.build(internal_loop)

    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=100,
        idle_threshold=100,
        
    )
    simulator_path, verilator_path = elaborate(sys, **conf)

    raw = utils.run_simulator(simulator_path)
    
    if verilator_path:
        raw = utils.run_verilator(verilator_path)

if __name__ == '__main__':
    test_spmv()