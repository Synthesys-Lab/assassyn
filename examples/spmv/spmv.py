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

N_SIZE = 494
L_SIZE = 10
I_MAX = N_SIZE - 1
J_MAX = L_SIZE - 1
M_SIZE = N_SIZE * L_SIZE
ADDR_WIDTH = (M_SIZE + M_SIZE + N_SIZE + N_SIZE).bit_length()
NZVAL_BASE = 0
COLS_BASE = M_SIZE
VEC_BASE = M_SIZE + M_SIZE
OUT_BASE = M_SIZE + M_SIZE + N_SIZE

class SRAM_USER:
    IDLE = Bits(3)(0)
    WAIT = Bits(3)(7)
    M1 =   Bits(3)(1)
    M2 =   Bits(3)(2)
    M3 =   Bits(3)(3)
    OUT =  Bits(3)(4)
    JUMP = Bits(3)(5)
    OUT_WAIT = Bits(3)(6)


class Memuser(Module):
    def __init__(self):
        super().__init__(
            ports={  'rdata': Port(Bits(32)) ,
                    #'user_sel': Port(Bits(3)) 
                    },
        ) 
        
    @module.combinational
    def build(self, i: Array, j: Array ,user_state: Array,cols_reg: Array,
              nzval_reg: Array,vec_reg: Array,address: Array, out: Array):

        cols_reg_addr  = Int(32)(0)
        nzval_reg_addr = Int(32)(0)
        vec_reg_addr   = Int(32)(0)

        rdata = self.pop_all_ports(True)
        cols_reg[0] = (user_state[0] == SRAM_USER.M1).select( rdata.bitcast(Int(32)), cols_reg[0])
        nzval_reg[0] = (user_state[0] == SRAM_USER.M2).select( rdata.bitcast(Int(32)), nzval_reg[0])
        vec_reg[0] = (user_state[0] == SRAM_USER.M3).select( rdata.bitcast(Int(32)), vec_reg[0])

        log(" cols_reg_addr: {} | user_state: {} ",  cols_reg_addr, user_state[0])
        



class SRAM_Master(Module):
    def __init__(self):
        super().__init__(
            ports={ 'Start': Port(Bits(1)) },
        ) 
        
    @module.combinational
    def build(self, i: Array, j: Array , init_file , memuser: Memuser , addr: Array ,
              cols_reg: Array, nzval_reg: Array,vec_reg: Array, out: Array , user_state: Array):

        sum = RegArray(Int(32), 1)

        Start = self.pop_all_ports(False)
        SRAM_Master_flag = RegArray(Bits(1), 1)
        log("user_state: {} ", user_state[0])
        SRAM_Master_flag[0] = (user_state[0] == SRAM_USER.M3).bitcast(Bits(1))

        re = Bits(1)(0)
        we = Bits(1)(0)
        re = (user_state[0] == SRAM_USER.M1)|(user_state[0] == SRAM_USER.M2)|(user_state[0] == SRAM_USER.WAIT)
        we = (user_state[0] == SRAM_USER.M3)&(j[0] == Int(32)(J_MAX))

        p = Int(32)(0)
        p = (i[0][0:15].bitcast(Int(16)) * Int(16)(J_MAX+1)).bitcast(Int(32)) + j[0]
        address_wire = Bits(ADDR_WIDTH)(0)

        cols_reg_addr = p + Int(32)(COLS_BASE)
        nzval_reg_addr = p + Int(32)(NZVAL_BASE)
        vec_reg_addr = cols_reg[0] + Int(32)(VEC_BASE)
        out_addr = p + Int(32)(OUT_BASE)

        address_wire = (user_state[0] == SRAM_USER.WAIT).select( cols_reg_addr[0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)), address_wire)
        address_wire = (user_state[0] == SRAM_USER.M1).select( nzval_reg_addr[0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)), address_wire)
        address_wire = (user_state[0] == SRAM_USER.M2).select( vec_reg_addr[0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)), address_wire)
        address_wire = (user_state[0] == SRAM_USER.OUT).select( out_addr[0:ADDR_WIDTH-1].bitcast(Bits(ADDR_WIDTH)), address_wire)

        with Condition((user_state[0] == SRAM_USER.M1)|(user_state[0] == SRAM_USER.M2)|(user_state[0] == SRAM_USER.M3)|(user_state[0] == SRAM_USER.WAIT)):
            with Condition(user_state[0] == SRAM_USER.M1):
                user_state[0] = SRAM_USER.M2

            with Condition(user_state[0] == SRAM_USER.M2):
                user_state[0] = SRAM_USER.M3

            with Condition(user_state[0] == SRAM_USER.M3):
                user_state[0] = (j[0]==Int(32)(J_MAX)).select(  SRAM_USER.OUT, SRAM_USER.JUMP)
                

            with Condition(user_state[0] == SRAM_USER.WAIT):
                with Condition(Start == Bits(1)(1)):
                    user_state[0] = SRAM_USER.M1
                

        with Condition(user_state[0] == SRAM_USER.IDLE):
            with Condition(Start == Bits(1)(1)):
                user_state[0] = SRAM_USER.WAIT

        with Condition(user_state[0] == SRAM_USER.OUT):
            user_state[0] = SRAM_USER.OUT_WAIT
            sum[0] = sum[0] + nzval_reg[0][0:15].bitcast(Int(16)) * vec_reg[0][0:15].bitcast(Int(16))
            
        with Condition(user_state[0] == SRAM_USER.JUMP):
            user_state[0] = SRAM_USER.WAIT
            sum[0] = sum[0] + nzval_reg[0][0:15].bitcast(Int(16)) * vec_reg[0][0:15].bitcast(Int(16))
            log("sum: {} = nzval_reg[0] * vec_reg[0] = {} * {}", sum[0], nzval_reg[0][0:15].bitcast(Int(16)), vec_reg[0][0:15].bitcast(Int(16)))
        
        with Condition(user_state[0] == SRAM_USER.OUT_WAIT):
            user_state[0] = SRAM_USER.WAIT
            out[0] = sum[0]
            sum[0] = Int(32)(0)
            log("sum-clear: {} = 0", sum[0])

        


        sram = SRAM(32, 2**ADDR_WIDTH , init_file)
        sram.build(we, re, address_wire, out[0].bitcast(Bits(32)), memuser)
        sram.bound.async_called()
        
        log("address_wire: {} cols_reg: {} | nzval_reg: {} | vec_reg: {}", address_wire, cols_reg[0], nzval_reg[0], vec_reg[0])
        log("we: {} | re: {} | addr: {} | out: {}", we, re, addr[0], out[0].bitcast(Bits(32)))

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
        
        log("outterloop----------In_full_flag: {} | i: {} |Stop-signal: {}", In_full_flag,i[0],full_flag[0])

        finish_flag = In_full_flag & (i[0] == Int(32)(I_MAX)) & (~ full_flag[0])
        with Condition(finish_flag):
            log("finish")
            finish()

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
    init_file = 'ellpack_data_reformatted.data'
    with sys:
        i = RegArray(Int(32), 1)
        j = RegArray(Int(32), 1)

        user_state = RegArray(Bits(3), 1)

        cols_reg = RegArray(Int(32), 1)
        nzval_reg = RegArray(Int(32), 1)
        vec_reg = RegArray(Int(32), 1)

        addr = RegArray( Bits(ADDR_WIDTH) , 1)

        out = RegArray(Int(32), 1)

        memuser = Memuser()
        memuser.build(i,j,user_state,cols_reg,nzval_reg,vec_reg,addr,out)

        sram_master = SRAM_Master()
        sram_master_flag = sram_master.build(i,j,init_file,memuser,addr,cols_reg , nzval_reg,vec_reg,out , user_state)
        
        external_loop = External_loop()
        external_loop.build(sram_master,i)

        internal_loop = Internal_loop()
        internal_loop.build(external_loop,sram_master_flag,j)

        driver = Driver()
        driver.build(internal_loop)

        sys.expose_on_top(i, kind='Output')
        sys.expose_on_top(j, kind='Output')
        sys.expose_on_top(user_state, kind='Output')
        sys.expose_on_top(out, kind='Output')

    
    
    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=30000,
        idle_threshold=30000,
        resource_base= f'{utils.repo_path()}/examples/spmv/data',
        
    )
    simulator_path, verilator_path = elaborate(sys, **conf)

    raw = utils.run_simulator(simulator_path)
    
    if verilator_path:
        raw = utils.run_verilator(verilator_path)

if __name__ == '__main__':
    test_spmv()